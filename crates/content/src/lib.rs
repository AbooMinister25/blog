#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_const_for_fn)]

use std::{
    default::Default,
    fmt::Debug,
    fs, io,
    path::{Path, PathBuf},
};

use color_eyre::Result;
use ignore::{DirEntry, Walk};
use rusqlite::Connection;
use sql::{get_hashes, insert_post, update_hash};
use tracing::trace;

#[derive(Debug)]
pub struct Entry {
    pub path: PathBuf,
    pub content: String,
    pub hash: String,
}

#[derive(Debug, Default)]
pub struct Post {
    pub path: PathBuf,
    pub raw_content: String,
    pub content: String,
}

impl Post {
    pub fn new(path: PathBuf, content: String) -> Self {
        Self {
            path,
            raw_content: content,
            ..Default::default()
        }
    }
}

impl Entry {
    pub fn new(path: PathBuf, content: String, hash: String) -> Self {
        Self {
            path,
            content,
            hash,
        }
    }
}

impl From<Entry> for Post {
    fn from(value: Entry) -> Self {
        Self::new(value.path, value.content)
    }
}

/// Traverse files in content folder, read in their content, hash them, and then filter to get
/// only the ones that have changed, or have been newly created, since the last run of the program.
#[tracing::instrument]
pub fn discover_entries<P: AsRef<Path> + Debug>(conn: &Connection, path: P) -> Result<Vec<Entry>> {
    let mut ret = Vec::new();

    let content_path = path.as_ref().join("contents");
    trace!("Discovering posts at {:?}", content_path);

    // TODO: Refactor this when introducing parallel stuff, it aint ideal right now

    let entries = Walk::new(content_path)
        .filter_map(Result::ok)
        .map(DirEntry::into_path)
        .filter(|p| !p.is_dir())
        .collect::<Vec<PathBuf>>();

    let content = entries
        .iter()
        .map(fs::read_to_string)
        .collect::<Result<Vec<String>, io::Error>>()?;

    let hashes = content
        .iter()
        .map(|s| format!("{:016x}", seahash::hash(s.as_bytes())))
        .collect::<Vec<String>>();

    for ((path, content), hash) in entries.into_iter().zip(content).zip(hashes) {
        let hashes = get_hashes(conn, &path)?;

        if hashes.is_empty() {
            // A new file was created.
            insert_post(conn, &path, &hash)?;
            ret.push(Entry::new(path, content, hash));
        } else if hashes[0] != hash {
            // Existing file was changed.
            update_hash(conn, &path, &hash)?;
            ret.push(Entry::new(path, content, hash));
        }
    }

    Ok(ret)
}
