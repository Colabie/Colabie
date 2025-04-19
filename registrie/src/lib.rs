use schemou::legos::ShortIdStr;

use std::sync::Arc;

use base64::prelude::*;
use git2::{build, Error, FileMode, Oid, Repository, Signature};
use tokio::{sync::Mutex, task::spawn_blocking};

pub use nanoserde::{DeRon, SerRon};

pub const AUTHOR: &str = "registrie";
pub const DEFAULT_BRANCH: &str = "main";

#[derive(DeRon, SerRon)]
pub struct Record {
    pub username: String,
    pub pubkey: String,
}

pub fn record_path(username: &ShortIdStr) -> String {
    if username.len() > 3 {
        format!(
            "{}/{}/{}",
            &username[0..2],
            &username[2..4],
            username.as_str()
        )
    } else {
        format!(
            "{}/{}/{}",
            &username[0..2],
            &username[2..3],
            username.as_str()
        )
    }
}

pub async fn new_record(
    git: Arc<Mutex<Repository>>,
    username: ShortIdStr,
    pubkey: Box<[u8]>,
) -> Result<Oid, Error> {
    let record = Record {
        username: username.to_string(),
        pubkey: BASE64_STANDARD.encode(pubkey),
    };

    // As git2 operations are blocking, we wrap those with `spawn_blocking()`
    // but then to keep track of the db lock from the async enviorment, ie. `tokio::sync::Mutex`
    // the blocking task waits `git.blocking_lock()` for the mutex lock

    spawn_blocking(move || {
        let data = record.serialize_ron();
        let blob = git.blocking_lock().blob(data.as_bytes())?;

        // The above lock gets freed and retaken below, as writting a blob is independent
        // So here some other task could use the db

        let repo = git.blocking_lock();
        let sig = Signature::now(AUTHOR, AUTHOR)?;

        let reference = repo
            .find_branch(DEFAULT_BRANCH, git2::BranchType::Local)?
            .into_reference();

        let last_commit = reference
            .peel_to_commit()
            .expect("Unreachable: no commit on reference");

        let tree = repo.find_tree(
            build::TreeUpdateBuilder::new()
                .upsert(record_path(&username), blob, FileMode::Blob)
                .create_updated(&repo, &last_commit.tree()?)?,
        )?;

        // TODO: Sign registrie's git commits
        // labels: enhancement
        // Issue URL: https://github.com/Colabie/Colabie/issues/7
        repo.commit(
            reference.name(),
            &sig,
            &sig,
            &format!("Register: {}", record.username),
            &tree,
            &[&last_commit],
        )
    })
    .await
    .unwrap()
}

pub async fn lookup_record(
    git: Arc<Mutex<Repository>>,
    username: ShortIdStr,
) -> Result<Option<Record>, Error> {
    // As git2 operations are blocking, we wrap those with `spawn_blocking()`
    // but then to keep track of the db lock from the async enviorment, ie. `tokio::sync::Mutex`
    // the blocking task waits `git.blocking_lock()` for the mutex lock

    spawn_blocking(move || {
        let path: String = record_path(&username);
        let path = std::path::Path::new(&path);
        let repo = git.blocking_lock();

        let tree_entry = {
            let tree = repo
                .find_branch(DEFAULT_BRANCH, git2::BranchType::Local)?
                .into_reference()
                .peel_to_commit()
                .expect("Unreachable: no commit on reference")
                .tree()?;

            match tree.get_path(path) {
                Ok(tree_entry) => tree_entry,
                Err(_) => return Ok(None),
            }
        };

        let record = {
            let blob = tree_entry
                .to_object(&repo)?
                .into_blob()
                .expect("Unreachable: path is not a blob");

            let raw_record =
                std::str::from_utf8(blob.content()).expect("Unreachable: non-utf8 record");

            Record::deserialize_ron(raw_record).expect("Unreachable: unparsable record")
        };

        Ok::<_, Error>(Some(record))
    })
    .await
    .unwrap()
}
