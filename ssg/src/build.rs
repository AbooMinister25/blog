use std::path::Path;

use crate::{entry::Entry, post::Post};
use color_eyre::{eyre::ContextCompat, Result};
use ignore::{DirEntry, Walk};
use rusqlite::Connection;
use tera::Tera;

// Walk over all site entries and build them.
#[tracing::instrument]
pub fn build(conn: &Connection, tera: &Tera) -> Result<()> {
    // Collect all entries
    let entries = Walk::new("contents/")
        .filter_map(Result::ok)
        .map(DirEntry::into_path)
        .map(|p| to_entry(&p))
        .collect::<Result<Vec<_>>>()?;

    // Build all entries
    entries
        .iter()
        .map(|e| e.build(conn, tera))
        .collect::<Result<Vec<_>>>()?;

    Ok(())
}

#[tracing::instrument]
fn to_entry(path: &Path) -> Result<impl Entry> {
    // If the path isn't a directory and isn't our index page, treat it as a post.
    let entry = if path.is_file()
        && path.file_name().context(
            "Everything here should have a file name, if it doesn't it your fault deal with it.",
        )? != "index.md"
    {
        Post {
            path: path.to_owned(),
        }
    } else {
        todo!()
    };

    Ok(entry)
}
