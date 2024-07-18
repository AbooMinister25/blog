use std::fmt::Debug;
use std::fs;
use std::path::{Path, PathBuf};

use color_eyre::Result;
use ignore::Walk;
use tracing::{info, trace};

use crate::context::Context;
use crate::sql::get_hashes;

/// Represent an entry - any item that is to be processed by the static site generator.
#[derive(Debug, PartialEq, Eq)]
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
pub fn discover_entries(ctx: &Context) -> Result<Vec<Entry>> {
    let mut ret = Vec::new();

    trace!("Discovering entries at {:?}", ctx.config.root);

    // TODO: Refactor this when introducing parallel processing, it isn't ideal right now
    let entries = read_entries(&ctx.config.root)?;
    info!("Discovered {:?} entries", entries.len());

    let hashes = entries
        .iter()
        .map(|(_, s)| format!("{:016x}", seahash::hash(s)))
        .collect::<Vec<String>>();

    for ((path, content), hash) in entries.into_iter().zip(hashes) {
        let hashes = get_hashes(&ctx.conn, &path)?;

        if hashes.is_empty() {
            // A new file was created.
            ret.push(Entry::new(path, content, hash, true));
        } else if hashes[0] != hash {
            // Existing file was changed.
            ret.push(Entry::new(path, content, hash, false));
        } else if ctx
            .config
            .special_pages
            .iter()
            .any(|ending| path.ends_with(ending))
        {
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
        ret.push((entry.into_path(), content));
    }

    Ok(ret)
}

#[cfg(test)]
mod tests {
    use crate::{config::Config, sql};

    use super::*;
    use markdown::MarkdownRenderer;
    use r2d2::{Pool, PooledConnection};
    use r2d2_sqlite::SqliteConnectionManager;
    use std::fs;
    use tempfile::tempdir;
    use tera::Tera;

    fn setup_db() -> Result<PooledConnection<SqliteConnectionManager>> {
        let manager = SqliteConnectionManager::memory();
        let pool = Pool::new(manager)?;
        let conn = pool.get()?;

        conn.execute(
            "
            CREATE TABLE IF NOT EXISTS entries (
                entry_id INTEGER PRIMARY KEY,
                path VARCHAR NOT NULL,
                hash TEXT NOT NULL
            )
        ",
            (),
        )?;

        Ok(conn)
    }

    #[test]
    fn test_read_entries() -> Result<()> {
        let tmp_dir = tempdir()?;

        for i in 1..=4 {
            let file_path = tmp_dir.path().join(format!("{i}.md"));
            fs::write(file_path, format!("Post {i}"))?;
        }

        let entries = read_entries(tmp_dir.path())?
            .into_iter()
            .map(|(p, c)| (p, String::from_utf8(c).unwrap()))
            .collect::<Vec<(PathBuf, String)>>();

        assert_eq!(
            entries,
            vec![
                (tmp_dir.path().join("4.md"), "Post 4".to_string()),
                (tmp_dir.path().join("3.md"), "Post 3".to_string()),
                (tmp_dir.path().join("2.md"), "Post 2".to_string()),
                (tmp_dir.path().join("1.md"), "Post 1".to_string()),
            ]
        );

        Ok(())
    }

    #[test]
    fn test_discover_entries() -> Result<()> {
        let conn = setup_db()?;
        let tmp_dir = tempdir()?;
        std::fs::create_dir(tmp_dir.path().join("themes/"))?;

        for i in 1..=4 {
            let file_path = tmp_dir.path().join(format!("{i}.md"));
            fs::write(file_path, format!("Post {i}"))?;
        }

        let context = Context::new(
            conn,
            Tera::default(),
            MarkdownRenderer::new(tmp_dir.path(), "Solarized (light)")?,
            Config {
                root: tmp_dir.path().to_owned(),
                ..Default::default()
            },
        );
        let entries = discover_entries(&context)?;

        assert_eq!(
            entries,
            vec![
                Entry::new(
                    tmp_dir.path().join("4.md"),
                    b"Post 4".to_vec(),
                    format!("{:016x}", seahash::hash(b"Post 4".as_ref())),
                    true
                ),
                Entry::new(
                    tmp_dir.path().join("3.md"),
                    b"Post 3".to_vec(),
                    format!("{:016x}", seahash::hash(b"Post 3".as_ref())),
                    true
                ),
                Entry::new(
                    tmp_dir.path().join("2.md"),
                    b"Post 2".to_vec(),
                    format!("{:016x}", seahash::hash(b"Post 2".as_ref())),
                    true
                ),
                Entry::new(
                    tmp_dir.path().join("1.md"),
                    b"Post 1".to_vec(),
                    format!("{:016x}", seahash::hash(b"Post 1".as_ref())),
                    true
                ),
            ]
        );

        for entry in entries {
            sql::insert_entry(&context.conn, &entry.path, &entry.hash)?;
        }

        let entries = discover_entries(&context)?;

        assert!(entries.is_empty());

        fs::write(tmp_dir.path().join("4.md"), "changed")?;
        let entries = discover_entries(&context)?;
        assert_eq!(
            entries,
            vec![Entry::new(
                tmp_dir.path().join("4.md"),
                b"changed".to_vec(),
                format!("{:016x}", seahash::hash(b"changed".as_ref())),
                false
            )]
        );

        Ok(())
    }
}
