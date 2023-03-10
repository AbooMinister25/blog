#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::module_name_repetitions)]

use color_eyre::Result;
use content::{post::Post, series::Series};
use entry::Entry;
use ignore::{DirEntry, Walk};
use rusqlite::Connection;
use sass::Stylesheet;
use std::path::Path;
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
    build_series(conn, tera)?;

    info!("Built entries");

    Ok(())
}

#[tracing::instrument]
fn build_entries<T: Entry>(conn: &Connection, tera: &Tera, path: &Path) -> Result<()> {
    // Discover entries
    let entries = Walk::new(path)
        .filter_map(Result::ok)
        .map(DirEntry::into_path)
        .filter(|p| {
            !p.is_dir()
                && p.file_name()
                    .expect("File name should never terminate in ..")
                    != "index.md"
        })
        .map(T::from_file);

    // Build entries
    entries
        .map(|e| e.build(conn, tera))
        .collect::<Result<Vec<_>>>()?;

    Ok(())
}

#[tracing::instrument]
fn build_series(conn: &Connection, tera: &Tera) -> Result<()> {
    // Discover entries
    let series = Walk::new("contents/")
        .filter_map(Result::ok)
        .map(DirEntry::into_path)
        .filter(|p| {
            !p.is_dir()
                && p.file_name()
                    .expect("File name should never terminate in ..")
                    == "index.md"
        })
        .map(Series::from_file);

    // Build entries
    series
        .map(|e| e.build(conn, tera))
        .collect::<Result<Vec<_>>>()?;

    Ok(())
}
