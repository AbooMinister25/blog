#![warn(clippy::pedantic, clippy::nursery)]

use std::{
    fmt::Debug,
    fs,
    path::{Path, PathBuf},
};

use color_eyre::{eyre::ContextCompat, Result};
use entry::Entry;
use tracing::trace;
use utils::fs::ensure_directory;

/// Represents a static asset. For the most part, they're copied over
/// to the resulting static site as-is. Their hashes are stored in
/// the database so that existing and unchanged files aren't repeatedly
/// copied over - the same as all other entries.
#[derive(Debug)]
pub struct StaticFile {
    pub path: PathBuf,
    pub hash: String,
    pub new: bool,
}

impl StaticFile {
    #[tracing::instrument]
    pub fn new(path: PathBuf, hash: String, new: bool) -> Self {
        Self { path, hash, new }
    }

    #[tracing::instrument]
    pub fn render<T: AsRef<Path> + Debug>(&self, output_directory: T) -> Result<()> {
        ensure_directory(
            output_directory
                .as_ref()
                .join("static/")
                .join(self.directory()?),
        )?;

        trace!("Rendering static file at {:?}", self.path);

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

        trace!("Rendered static file to {:?}", out_path);

        Ok(())
    }

    #[tracing::instrument]
    fn directory(&self) -> Result<PathBuf> {
        let parent = self.path.parent().unwrap(); // All static entries will have a parent
        Ok(if parent.ends_with("static") {
            PathBuf::from(".")
        } else {
            parent
                .components()
                .skip_while(|c| {
                    let p = AsRef::<Path>::as_ref(c);
                    !p.ends_with("static")
                })
                .skip(1)
                .collect::<PathBuf>()
        })
    }
}

impl From<Entry> for StaticFile {
    fn from(value: Entry) -> Self {
        Self::new(value.path, value.hash, value.new)
    }
}
