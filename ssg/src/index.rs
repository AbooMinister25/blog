use crate::entry::BuildStatus;
use crate::entry::Entry;
use crate::markdown::Document;
use crate::utils::ensure_directory;
use crate::DATE_FORMAT;
use color_eyre::eyre::{ContextCompat, Result};
use rusqlite::Connection;
use std::path::Path;
use std::{fs, path::PathBuf};
use tera::{Context, Tera};
use tracing::debug;

// The index page found at `/`. Represented by an `index.md` in the `contents/` folder.
#[derive(Debug)]
pub struct Post {
    pub path: PathBuf,
}

impl Entry for Post {
    #[tracing::instrument]
    fn from_file(path: PathBuf) -> Self {
        Self { path }
    }

    #[tracing::instrument]
    fn build_status(&self, _: &Connection) -> Result<BuildStatus> {
        // No incremental building for index.html
        Ok(BuildStatus::New(String::new()))
    }

    #[tracing::instrument]
    fn hash(&self) -> Result<String> {
        // TODO: Refactor this to use an Option instead
        Ok(String::new())
    }

    #[tracing::instrument]
    fn build(&self, conn: &Connection, tera: &Tera) -> Result<()> {
        ensure_directory(Path::new("public/"))?;
        debug!("Building index.md");

        let parsed_document = Document::from_file(&self.path)?;
        render_index(tera, parsed_document)?;
        debug!("Built index");

        Ok(())
    }
}

#[tracing::instrument]
fn render_index(tera: &Tera, document: Document) -> Result<()> {
    // Create the file
    let file = fs::File::create("public/index.html")?;

    // Insert context for the template

    Ok(())
}
