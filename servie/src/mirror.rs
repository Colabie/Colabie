use registrie::{lookup_record, Record, DEFAULT_BRANCH};
use schemou::legos::ShortIdStr;

use std::sync::Arc;

use git2::{build::RepoBuilder, Error, Repository};
use tokio::{sync::Mutex, task::spawn_blocking};

#[derive(Clone)]
pub struct Mirror {
    git: Arc<Mutex<Repository>>,
}

impl Mirror {
    // This function blocks on io operations
    // That's fine as it's called once at the very start
    pub async fn open_or_create(url: String, path: String) -> Result<Self, Error> {
        let mirror = if let Ok(repo) = Repository::open_bare(&path) {
            let mirror = Self {
                git: Arc::new(Mutex::new(repo)),
            };

            mirror.fetch_db().await?;
            Ok::<_, Error>(mirror)
        } else {
            tracing::info!("cloning registrie");
            Ok(Self {
                git: Arc::new(Mutex::new(
                    RepoBuilder::new()
                        .bare(true)
                        .clone(&url, std::path::Path::new(&path))?,
                )),
            })
        };

        mirror
    }

    pub async fn fetch_db(&self) -> Result<(), Error> {
        tracing::info!("fetching registrie");
        let handle = tokio::runtime::Handle::current();
        let repo = self.clone();
        spawn_blocking(move || {
            handle
                .block_on(repo.git.lock())
                .find_remote("origin")?
                .fetch(&[DEFAULT_BRANCH], None, None)?;

            let (merge_analysis, _) = {
                let repo = handle.block_on(repo.git.lock());

                let annotated_commit =
                    repo.reference_to_annotated_commit(&repo.find_reference("FETCH_HEAD")?)?;
                repo.merge_analysis(&[&annotated_commit])?
            };

            if merge_analysis.is_up_to_date() {
                return Ok(());
            }

            if merge_analysis.is_fast_forward() {
                let repo = handle.block_on(repo.git.lock());
                repo.find_branch(DEFAULT_BRANCH, git2::BranchType::Local)
                    .expect("Default Branch")
                    .into_reference()
                    .set_target(
                        repo.reference_to_annotated_commit(&repo.find_reference("FETCH_HEAD")?)?
                            .id(),
                        "Fetch Mirror",
                    )?;
            } else {
                return Err(Error::from_str("Fast-forward only!"));
            }

            Ok(())
        })
        .await
        .unwrap()
    }

    pub async fn lookup_record(self, username: ShortIdStr) -> Option<Record> {
        lookup_record(self.git, username)
            .await
            .expect("Git database not accessible")
    }
}
