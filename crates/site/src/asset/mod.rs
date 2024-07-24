mod embed_fonts;
mod esbuild;

use std::{
    ffi::OsStr,
    fmt::Debug,
    fs,
    path::{Path, PathBuf},
};

use color_eyre::{eyre::ContextCompat, Result};
use rsass::{compile_scss_path, output};
use tracing::trace;

use crate::{
    asset::embed_fonts::embed_font, asset::esbuild::bundle_js, context::Context, output::Output,
    utils::fs::ensure_directory,
};

// Represents a resource that is typically passed through an asset pipeline.
/// This can include images, stylesheets, or javascript.
#[derive(Debug)]
pub struct Asset {
    pub path: PathBuf,
    pub out_path: PathBuf,
    raw_content: Vec<u8>,
    content: Vec<u8>,
    hash: String,
    fresh: bool,
}

impl Asset {
    #[tracing::instrument(level = tracing::Level::DEBUG)]
    pub fn new<P: AsRef<Path> + Debug>(
        ctx: &Context,
        path: P,
        raw_content: Vec<u8>,
        hash: String,
        fresh: bool,
    ) -> Result<Self> {
        trace!("Processing asset at {path:?}");

        let out_path = out_path(&path, &ctx.config.output_path)?;
        ensure_directory(out_path.parent().context("Path should have parent")?)?;

        let (content, out_path) = preprocess(&path, out_path)?;
        trace!("Processed asset at {path:?}");

        Ok(Self {
            path: path.as_ref().to_owned(),
            out_path,
            raw_content,
            content,
            hash,
            fresh,
        })
    }

    #[tracing::instrument(level = tracing::Level::DEBUG)]
    fn postprocess(&self) -> Result<()> {
        if let Some(e) = self.path.extension() {
            if e == "svg" {
                embed_font(&self.out_path)?;
            }
        }

        Ok(())
    }
}

impl Output for Asset {
    #[tracing::instrument(level = tracing::Level::DEBUG)]
    fn write(&self, ctx: &Context) -> Result<()> {
        trace!(
            "Writing asset at {:?} to disk at {:?}",
            self.path,
            self.out_path
        );
        fs::write(&self.out_path, &self.content)?;
        trace!(
            "Wrote asset at {:?} to disk at {:?}",
            self.path,
            self.out_path
        );

        trace!("Postprocessing asset at {:?}", self.out_path);
        self.postprocess()?;
        trace!("Postprocessed asset at {:?}", self.out_path);

        Ok(())
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn hash(&self) -> &str {
        &self.hash
    }

    fn fresh(&self) -> bool {
        self.fresh
    }
}

fn out_path<P: AsRef<Path>, T: AsRef<Path>>(path: P, output_path: T) -> Result<PathBuf> {
    let parent = path.as_ref().parent().unwrap(); // All assets will have a parent.
    let directory = if parent.ends_with("assets") {
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
    };

    let filename = path
        .as_ref()
        .file_stem()
        .context("Asset should have a filename")?;

    let out_dir = output_path.as_ref().join("assets/").join(directory);
    Ok(out_dir.join(filename))
}

#[tracing::instrument(level = tracing::Level::DEBUG)]
fn preprocess<P: AsRef<Path> + Debug, T: AsRef<Path> + Debug>(
    path: P,
    output_path: T,
) -> Result<(Vec<u8>, PathBuf)> {
    let mut op = output_path.as_ref().to_owned();

    Ok((
        match path.as_ref().extension().and_then(OsStr::to_str) {
            Some("scss") => {
                op.set_extension("css");
                let format = output::Format {
                    style: output::Style::Compressed,
                    ..Default::default()
                };

                compile_scss_path(path.as_ref(), format)?
            }
            Some("js") => {
                op.set_extension("js");
                bundle_js(path)?
            }
            Some(ext) => {
                op.set_extension(ext);
                fs::read(path)?
            }
            None => fs::read(path)?,
        },
        op,
    ))
}
