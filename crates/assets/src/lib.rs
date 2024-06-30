#![warn(clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::missing_panics_doc,
    clippy::missing_const_for_fn,
    clippy::must_use_candidate
)]

mod embed_fonts;

use std::{
    ffi::OsStr,
    fmt::Debug,
    fs,
    path::{Path, PathBuf},
};

use color_eyre::{eyre::ContextCompat, Result};
use embed_fonts::embed_font;
use entry::Entry;
use rsass::{compile_scss_path, output};
use tracing::trace;
use utils::fs::ensure_directory;

/// Represents a resource that is typically passed through an asset pipeline.
/// This can include images, stylesheets, or javascript.
#[derive(Debug)]
pub struct Asset {
    path: PathBuf,
    raw_content: Vec<u8>,
}

impl Asset {
    #[tracing::instrument]
    pub fn new(path: PathBuf, raw_content: Vec<u8>) -> Self {
        Self { path, raw_content }
    }

    #[tracing::instrument]
    pub fn render<T: AsRef<Path> + Debug>(&self, output_directory: T) -> Result<()> {
        let directory = self.directory()?;

        ensure_directory(output_directory.as_ref().join("assets/").join(&directory))?;

        trace!("Rendering asset at {:?}", self.path);

        let filename = self
            .path
            .file_stem()
            .context("Invalid filename")?
            .to_str()
            .context("Filename not valid unicode")?;

        let out_dir = output_directory.as_ref().join("assets/").join(directory);
        let out_path = self.preprocess_and_write(out_dir, filename)?;

        trace!("Rendered asset file to {:?}", out_path);

        Ok(())
    }

    #[tracing::instrument]
    fn preprocess_and_write<T: AsRef<Path> + Debug>(
        &self,
        out_dir: T,
        filename: &str,
    ) -> Result<PathBuf> {
        Ok(match self.path.extension().and_then(OsStr::to_str) {
            Some(ext @ "scss") => {
                let out_path = out_dir.as_ref().join(format!("{filename}.css"));

                let format = output::Format {
                    style: output::Style::Compressed,
                    ..Default::default()
                };

                let css = compile_scss_path(&self.path, format)?;
                fs::write(&out_path, css)?;

                out_path
            }
            Some(ext @ "js") => {
                todo!()
            }
            Some(ext) => {
                let out_path = out_dir.as_ref().join(format!("{filename}.{ext}"));
                fs::copy(&self.path, &out_path)?;

                out_path
            }
            None => {
                let out_path = out_dir.as_ref().join(filename);
                fs::copy(&self.path, &out_path)?;

                out_path
            }
        })
    }

    #[tracing::instrument]
    fn postprocess<T: AsRef<Path> + Debug>(&self, out_path: T) -> Result<()> {
        if let Some(e) = self.path.extension() {
            if e == "svg" {
                embed_font(&out_path)?;
            }
        }

        Ok(())
    }

    #[tracing::instrument]
    fn directory(&self) -> Result<PathBuf> {
        let parent = self.path.parent().unwrap(); // All asset entries will have a parent
        Ok(if parent.ends_with("assets") {
            PathBuf::from(".")
        } else {
            parent
                .components()
                .skip_while(|c| {
                    let p = AsRef::<Path>::as_ref(c);
                    !p.ends_with("assets")
                })
                .skip(1)
                .collect::<PathBuf>()
        })
    }
}

impl From<Entry> for Asset {
    fn from(value: Entry) -> Self {
        Self::new(value.path, value.raw_content)
    }
}
