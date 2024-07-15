use std::fmt::Debug;
use std::fs;
use std::path::{Path, PathBuf};

use color_eyre::Result;
use ignore::Walk;
use rusqlite::Connection;
use sql::{get_hashes, insert_entry, update_entry_hash};
use tracing::{info, trace};

/// Represent an entry - any item that is to be processed by the static site generator.
#[derive(Debug)]
pub struct Entry {
    pub path: PathBuf,
    pub raw_content: Vec<u8>,
    pub hash: String,
    pub new: bool,
}

impl Entry {
    pub fn new(path: PathBuf, raw_content: Vec<u8>, hash: String, new: bool) -> Self {
        Self {
            path,
            raw_content,
            hash,
            new,
        }
    }
}

/// Recursively traverse the files in the given path, read them, hash them, and then
/// filter out only the ones that have changed or have been newly created since the last
/// run of the program.
#[tracing::instrument]
pub fn discover_entries<T: AsRef<Path> + Debug>(
    conn: &Connection,
    path: T,
    special_pages: &[String],
) -> Result<Vec<Entry>> {
    let mut ret = Vec::new();

    trace!("Discovering entries at {:?}", path);

    // TODO: Refactor this when introducing parallel stuff, it isn't ideal right now
    let entries = read_entries(&path)?;
    info!("Discovered {:?} entries", entries.len());

    let hashes = entries
        .iter()
        .map(|(_, s)| format!("{:016x}", seahash::hash(s)))
        .collect::<Vec<String>>();

    for ((path, content), hash) in entries.into_iter().zip(hashes) {
        let hashes = get_hashes(conn, &path)?;

        if hashes.is_empty() {
            // A new file was created.
            ret.push(Entry::new(path, content, hash, true));
        } else if hashes[0] != hash {
            // Existing file was changed.
            ret.push(Entry::new(path, content, hash, false));
        } else if special_pages.iter().any(|ending| path.ends_with(ending)) {
            ret.push(Entry::new(path, content, hash, false));
        }
    }

    info!("Building {:?} entries", ret.len());

    Ok(ret)
}

#[tracing::instrument]
fn read_entries<T: AsRef<Path> + Debug>(path: T) -> Result<Vec<(PathBuf, Vec<u8>)>> {
    let mut ret = Vec::new();

    for entry in Walk::new(path.as_ref())
        .filter_map(Result::ok)
        .filter(|e| !e.path().is_dir())
    {
        trace!("Reading entry at {:?}", entry.path());
        let content = fs::read(entry.path())?;
        ret.push((entry.into_path(), content))
    }

    Ok(ret)
}
