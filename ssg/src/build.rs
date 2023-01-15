use std::path::Path;

use crate::{entry::Entry, post::Post, stylesheet::Stylesheet};
use color_eyre::Result;
use ignore::{DirEntry, Walk};
use rusqlite::Connection;
use tera::Tera;
use tracing::info;

// Walk over all site entries and build them.
#[tracing::instrument(skip(tera))]
pub fn build(conn: &Connection, tera: &Tera) -> Result<()> {
    let current_dir = Path::new(".");
    let content_dir = current_dir.join("contents");
    let sass_dir = current_dir.join("sass");

    // Build the entries
    build_entries::<Post>(conn, tera, &content_dir)?;
    build_entries::<Stylesheet>(conn, tera, &sass_dir)?;

    info!("Built entries");

    Ok(())
}

#[tracing::instrument]
fn build_entries<T: Entry>(conn: &Connection, tera: &Tera, path: &Path) -> Result<()> {
    // Discover entries
    let entries = Walk::new(path)
        .filter_map(Result::ok)
        .map(DirEntry::into_path)
        .filter(|p| !p.is_dir())
        .map(|p| T::from_file(p))
        .collect::<Vec<_>>();

    // Build entries
    entries
        .iter()
        .map(|e| e.build(conn, tera))
        .collect::<Result<Vec<_>>>()?;

    Ok(())
}
