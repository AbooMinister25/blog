use std::path::Path;

use crate::{entry::Entry, post::Post, stylesheet::Stylesheet};
use color_eyre::{eyre::ContextCompat, Result};
use ignore::{DirEntry, Walk};
use rusqlite::Connection;
use std::fs;
use tera::Tera;
use tracing::info;

// Walk over all site entries and build them.
#[tracing::instrument(skip(tera))]
pub fn build(conn: &Connection, tera: &Tera) -> Result<()> {
    // Create directories
    create_directories()?;
    info!("Created directories");

    // Collect all entries
    let entries = Walk::new("site/")
        .filter_map(Result::ok)
        .map(DirEntry::into_path)
        .filter(|path| !path.is_dir())
        .map(|p| to_entry(&p))
        .collect::<Result<Vec<_>>>()?;
    info!("Discovered {} entries", entries.len());

    // Build all entries
    entries
        .iter()
        .map(|e| e.build(conn, tera))
        .collect::<Result<Vec<_>>>()?;
    info!("Built entries");

    Ok(())
}

#[tracing::instrument]
fn to_entry(path: &Path) -> Result<Box<dyn Entry>> {
    let filename = path.file_name().context("Path should have a filename")?;
    // Directories have been filtered out
    let extension = path.extension().context("File should have an extension")?;

    // If the path isn't a directory and isn't our index page, treat it as a post.
    if extension == "md" && filename != "index.md" {
        Ok(Box::new(Post {
            path: path.to_owned(),
        }))
    } else if extension == "scss" {
        Ok(Box::new(Stylesheet {
            path: path.to_owned(),
        }))
    } else {
        todo!()
    }
}

#[tracing::instrument]
fn create_directories() -> Result<()> {
    fs::create_dir_all("dist")?;
    fs::create_dir_all("dist/styles")?;
    fs::create_dir_all("dist/public")?;

    Ok(())
}
