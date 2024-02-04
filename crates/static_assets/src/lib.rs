#![warn(clippy::pedantic, clippy::nursery)]

mod embed_fonts;

use std::{
    fmt::Debug,
    fs,
    path::{Path, PathBuf},
};

use color_eyre::{eyre::ContextCompat, Result};
use embed_fonts::embed_font;
use entry::Entry;
use tracing::trace;
use utils::fs::ensure_directory;

/// Represents a static asset. For the most part, they're copied over
/// to the resulting static site as-is. Their hashes are stored in
/// the database so that existing and unchanged files aren't repeatedly
/// copied over - the same as all other entries.
#[derive(Debug)]
pub struct StaticAsset {
    pub path: PathBuf,
}

impl StaticAsset {
    #[tracing::instrument]
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    #[tracing::instrument]
    pub fn render<T: AsRef<Path> + Debug>(&self, output_directory: T) -> Result<()> {
        ensure_directory(
            output_directory
                .as_ref()
                .join("static/")
                .join(self.directory()?),
        )?;

        trace!("Rendering static asset at {:?}", self.path);

        let filename = self
            .path
            .file_name()
            .context("Invalid filename")?
            .to_str()
            .context("Filename not valid unicode")?;
        let out_path = output_directory
            .as_ref()
            .join("static/")
            .join(self.directory()?)
            .join(filename);

        fs::copy(&self.path, &out_path)?;

        if let Some(e) = self.path.extension() {
            if e == "svg" {
                embed_font(&out_path)?;
            }
        }

        trace!("Rendered static asset at {:?}", out_path);

        Ok(())
    }

    #[tracing::instrument]
    fn directory(&self) -> Result<&str> {
        let parent = self.path.parent().unwrap(); // All static entries will have a parent
        Ok(if parent == Path::new("/static") {
            "."
        } else {
            parent
                .file_name()
                .context("Path should have a filename")?
                .to_str()
                .context("Path should be valid UTF-8")?
        })
    }
}

impl From<Entry> for StaticAsset {
    fn from(value: Entry) -> Self {
        Self::new(value.path)
    }
}
