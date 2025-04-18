use std::sync::Arc;

use git2::{Oid, Repository, Signature};
use schemou::legos::ShortIdStr;
use tokio::sync::Mutex;

use crate::erout;
use registrie::*;

#[derive(Clone)]
pub struct DB {
    git: Arc<Mutex<Repository>>,
}

impl DB {
    // This function blocks on fs operations
    // That's fine as it's called once at the very start
    pub fn get_or_create(path: &str) -> Self {
        let git = Arc::new(Mutex::new(
            Repository::open_bare(path).unwrap_or_else(|_| DB::init_repo(path).unwrap()),
        ));

        tracing::info!("openned git database repo");
        Self { git }
    }

    pub async fn new_record(&self, username: ShortIdStr, pubkey: Box<[u8]>) -> Oid {
        new_record(self.git.clone(), username, pubkey)
            .await
            .expect("Git database not accessible")
    }

    fn init_repo(path: &str) -> Result<Repository, git2::Error> {
        tracing::info!("initializing new git database repo");
        let repo = Repository::init_bare(path).expect("OS");
        {
            let sig = Signature::now(AUTHOR, AUTHOR)?;
            let tree = repo.find_tree(repo.treebuilder(None)?.write()?)?;
            let commit = repo.find_commit(erout!(repo.commit(
                None,
                &sig,
                &sig,
                "Initial Commit",
                &tree,
                &[]
            )))?;

            erout!(repo.branch(DEFAULT_BRANCH, &commit, false));
        }
        Ok(repo)
    }
}

#[cfg(test)]
mod db_tests {
    use base64::{prelude::BASE64_STANDARD, Engine};
    use registrie::lookup_record;
    use schemou::legos::ShortIdStr;
    use tokio::task::spawn_blocking;

    use super::DB;
    use std::fs;

    #[tokio::test]
    async fn new_record() {
        let path = rand::random::<u64>().to_string();
        let db = {
            let path = path.clone();
            spawn_blocking(move || DB::get_or_create(&path))
                .await
                .unwrap()
        };

        let username = ShortIdStr::new("duskyelf").unwrap();
        let pubkey: Box<[u8]> = [1, 2, 3, 13, 42].into();

        db.new_record(username.clone(), pubkey.clone()).await;

        let record = lookup_record(db.git, username.clone())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(*username, record.username);
        assert_eq!(
            pubkey,
            BASE64_STANDARD.decode(record.pubkey).unwrap().into()
        );

        fs::remove_dir_all(path).unwrap();
    }
}
