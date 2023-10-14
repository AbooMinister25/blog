#![warn(clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::missing_panics_doc,
    clippy::missing_const_for_fn,
    clippy::must_use_candidate
)]

use std::fmt::Debug;
use std::fs;
use std::path::{Path, PathBuf};

use color_eyre::{eyre::ContextCompat, Result};
use entry::Entry;
use rsass::{compile_scss_path, output};
use tracing::trace;
use utils::fs::ensure_directory;

/// A stylesheet
#[derive(Debug)]
pub struct Stylesheet {
    pub path: PathBuf,
    pub raw_content: String,
}

impl Stylesheet {
    #[tracing::instrument]
    pub fn new(path: PathBuf, content: String) -> Self {
        Self {
            path,
            raw_content: content,
        }
    }

    #[tracing::instrument]
    pub fn render<T: AsRef<Path> + Debug>(&self, output_directory: T) -> Result<()> {
        ensure_directory(output_directory.as_ref().join("styles/"))?;

        trace!("Rendering stylesheet at {:?}", self.path);

        let filename = self
            .path
            .file_stem()
            .context("Invalid filename")?
            .to_str()
            .context("Filename not valid unicode")?;
        let out_path = output_directory
            .as_ref()
            .join("styles/")
            .join(format!("{filename}.css"));

        trace!("Rendering stylesheet to {:?}", out_path);

        let format = output::Format {
            style: output::Style::Compressed,
            ..Default::default()
        };

        // Compile and write the CSS to disk
        let css = compile_scss_path(&self.path, format)?;
        fs::write(out_path, css)?;

        trace!("Rendered stylesheet");

        Ok(())
    }
}

impl From<Entry> for Stylesheet {
    fn from(value: Entry) -> Self {
        Self::new(value.path, value.content)
    }
}
