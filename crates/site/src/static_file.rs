use std::{
    fmt::Debug,
    fs,
    path::{Path, PathBuf},
};

use color_eyre::{eyre::ContextCompat, Result};
use tracing::trace;

use crate::utils::fs::ensure_directory;
use crate::{context::Context, output::Output};

/// Represents a static asset. For the most part, they're copied over
/// to the resulting static site as-is. Their hashes are stored in
/// the database so that existing and unchanged files aren't repeatedly
/// copied over - the same as all other entries.
#[derive(Debug)]
pub struct StaticFile {
    pub path: PathBuf,
    pub out_path: PathBuf,
    pub hash: String,
    pub fresh: bool,
}

impl StaticFile {
    #[tracing::instrument(level = tracing::Level::DEBUG)]
    pub fn new<P: AsRef<Path> + Debug>(
        ctx: &Context,
        path: P,
        hash: String,
        fresh: bool,
    ) -> Result<Self> {
        trace!("Processing static asset at {path:?}");

        let out_path = out_path(&path, &ctx.config.output_path)?;
        ensure_directory(out_path.parent().context("Path should havae a parent")?)?;
        trace!("Processed static file at {path:?}");

        Ok(Self {
            path: path.as_ref().to_owned(),
            out_path,
            hash,
            fresh,
        })
    }
}

impl Output for StaticFile {
    fn write(&self, _: &Context) -> Result<()> {
        trace!(
            "Writing static file at {:?} to disk at {:?}",
            self.path,
            self.out_path
        );
        fs::copy(&self.path, &self.out_path)?;
        trace!(
            "Wrote static file at {:?} to disk at {:?}",
            self.path,
            self.out_path
        );

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
    let parent = path.as_ref().parent().unwrap(); // All static entries will have a parent.
    let directory = if parent.ends_with("static") {
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
    };

    let filename = path.as_ref().file_name().context("Invalid filename")?;
    let out_dir = output_path.as_ref().join("static/").join(directory);

    Ok(out_dir.join(filename))
}
