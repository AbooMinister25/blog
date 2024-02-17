use std::fmt::Debug;
use std::fs;
use std::path::{Path, PathBuf};

use color_eyre::Result;
use ignore::Walk;
use rusqlite::Connection;
use sql::{get_hashes, insert_entry, update_entry_hash};
use tracing::{info, trace};

#[salsa::jar(db = Db)]
pub struct Jar(Entry);

pub trait Db: salsa::DbWithJar<Jar> {}
impl<DB> Db for DB where DB: ?Sized + salsa::DbWithJar<Jar> {}

/// Represent an entry - any item that is to be processed by the static site generator.
#[salsa::input]
#[derive(Debug)]
pub struct Entry {
    pub path: PathBuf,
    #[return_ref]
    pub raw_content: Vec<u8>,
    #[return_ref]
    pub hash: String,
}

/// Recursively traverse the files in the given path, read them, hash them, and then
/// filter out only the ones that have changed or have been newly created since the last
/// run of the program.
#[tracing::instrument(skip(db))]
pub fn discover_entries<T: AsRef<Path> + Debug>(
    db: &dyn Db,
    conn: &Connection,
    path: T,
) -> Result<Vec<Entry>> {
    let mut ret = Vec::new();

    trace!("Discovering entries at {:?}", path);

    // TODO: Refactor this when introducing parallel stuff, it aint ideal right now
    let entries = read_entries(conn, &path)?;
    info!("Discovered {:?} entries", entries.len());

    let hashes = entries
        .iter()
        .map(|(_, s)| format!("{:016x}", seahash::hash(s)))
        .collect::<Vec<String>>();

    for ((path, content), hash) in entries.into_iter().zip(hashes) {
        let hashes = get_hashes(conn, &path)?;

        if hashes.is_empty() {
            // A new file was created.
            insert_entry(conn, &path, &hash)?;
            ret.push(Entry::new(db, path, content, hash));
        } else if hashes[0] != hash {
            // Existing file was changed.
            update_entry_hash(conn, &path, &hash)?;
            ret.push(Entry::new(db, path, content, hash));
        }
    }

    info!("Building {:?} entries", ret.len());

    Ok(ret)
}

#[tracing::instrument]
fn read_entries<T: AsRef<Path> + Debug>(
    conn: &Connection,
    path: T,
) -> Result<Vec<(PathBuf, Vec<u8>)>> {
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
