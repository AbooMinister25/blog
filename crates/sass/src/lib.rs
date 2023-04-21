#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::module_name_repetitions)]

use color_eyre::eyre::{ContextCompat, Result};
use entry::filesystem::ensure_directory;
use entry::{BuildStatus, Entry};
use rsass::{compile_scss_path, output};
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};
use tera::Tera;
use tracing::debug;

// A stylesheet
#[derive(Debug)]
pub struct Stylesheet {
    pub path: PathBuf,
}

impl Entry for Stylesheet {
    #[tracing::instrument]
    fn from_file(path: PathBuf) -> Self {
        Self { path }
    }

    #[tracing::instrument]
    fn build_status(&self, _: &Connection) -> Result<BuildStatus> {
        // No incremental building for stylesheets
        Ok(BuildStatus::New(String::new()))
    }

    #[tracing::instrument]
    fn hash(&self) -> Result<String> {
        // TODO: Refactor this to use an Option instead
        Ok(String::new())
    }

    #[tracing::instrument]
    fn build(&self, conn: &Connection, _: &Tera, _: BuildStatus) -> Result<()> {
        ensure_directory(Path::new("public/styles"))?;
        debug!(
            "Building stylesheet at {}",
            self.path.to_str().context("Path should be valid unicode")?
        );

        let filename = self
            .path
            .file_stem()
            .context("Invalid filename")?
            .to_str()
            .context("Filename not valid unicode")?;

        let format = output::Format {
            style: output::Style::Compressed,
            ..Default::default()
        };

        // Compile and write the CSS to disk
        let css = compile_scss_path(&self.path, format)?;
        fs::File::create(format!("public/styles/{filename}.css"))?;
        fs::write(format!("public/styles/{filename}.css"), css)?;

        debug!("Built stylesheet");

        Ok(())
    }
}
