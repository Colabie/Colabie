use registrie::{lookup_record, Record, DEFAULT_BRANCH};
use schemou::legos::ShortIdStr;

use std::{fs, sync::Arc};

use git2::{build::RepoBuilder, Error, Repository};
use tokio::{sync::Mutex, task::spawn_blocking};

#[derive(Clone)]
pub struct Mirror {
    git: Arc<Mutex<Repository>>,
}

impl Mirror {
    // This function blocks on io operations
    // That's fine as it's called once at the very start
    pub async fn open_or_create() -> Result<Self, Error> {
        let path = std::env::var("MIRROR_PATH").expect("MIRROR_PATH environment variable not set");
        let url = {
            let upstream_url_env =
                std::env::var("UPSTREAM_URL").expect("UPSTREAM_URL environment variable not set");

            if upstream_url_env.starts_with("https://") {
                upstream_url_env
            } else {
                let path =
                    fs::canonicalize(&upstream_url_env).expect("Failed to canonicalize local path");
                format!("file://{}", path.to_str().expect("Invalid UTF-8 in path"))
            }
        };

        if let Ok(repo) = Repository::open_bare(&path) {
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
        }
    }

    pub async fn fetch_db(&self) -> Result<(), Error> {
        tracing::info!("fetching registrie");
        let repo = self.clone();
        spawn_blocking(move || {
            let repo = repo.git.blocking_lock();

            let (merge_analysis, _) = {
                let annotated_commit =
                    repo.reference_to_annotated_commit(&repo.find_reference("FETCH_HEAD")?)?;
                repo.merge_analysis(&[&annotated_commit])?
            };

            if merge_analysis.is_up_to_date() {
                return Ok(());
            }

            if merge_analysis.is_fast_forward() {
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
