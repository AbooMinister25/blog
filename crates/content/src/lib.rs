use std::{
    fmt::Debug,
    fs,
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

#[derive(Debug)]
pub struct Post {
    pub path: PathBuf,
    pub content: String,
}

impl Post {
    pub fn new(path: PathBuf, content: String) -> Self {
        Self { path, content }
    }
}

impl Entry {
    pub fn from_file(path: PathBuf) -> Result<Self> {
        let content = fs::read_to_string(&path)?;
        let hash = format!("{:016x}", seahash::hash(content.as_bytes()));

        Ok(Self {
            path,
            content,
            hash,
        })
    }
}

impl From<Entry> for Post {
    fn from(value: Entry) -> Self {
        Self {
            path: value.path,
            content: value.content,
        }
    }
}

/// Traverses the files in the current folder, reads in their contents, hashes them, and yields `Entry`s.
#[tracing::instrument]
pub fn discover_entries<P: AsRef<Path> + Debug>(path: P) -> Result<Vec<Entry>> {
    let content_path = path.as_ref().join("contents");
    trace!("Discovering entries at {:?}", content_path);

    Walk::new(content_path)
        .filter_map(Result::ok)
        .map(DirEntry::into_path)
        .filter(|p| !p.is_dir())
        .map(Entry::from_file)
        .collect()
}

/// Filter entries to get only the ones that have changed, or have been newly created, since the last run of the program.
#[tracing::instrument]
pub fn to_build(conn: &Connection, entries: Vec<Entry>) -> Result<Vec<Entry>> {
    let mut ret = Vec::new();

    for entry in entries {
        let hashes = get_hashes(conn, &entry.path)?;

        if hashes.is_empty() {
            // A new file was created.
            insert_post(conn, &entry.path, &entry.hash)?;
            ret.push(entry)
        } else if hashes[0] != entry.hash {
            // Existing file was changed.
            update_hash(conn, &entry.path, &entry.hash)?;
            ret.push(entry)
        }
    }

    Ok(ret)
}
