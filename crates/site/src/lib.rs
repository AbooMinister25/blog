#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::must_use_candidate)]

use color_eyre::Result;
use content::BlogContent;
use entry::BuildStatus;
use entry::Entry;
use ignore::{DirEntry, Walk};
use rusqlite::Connection;
use sass::Stylesheet;
use static_assets::StaticAsset;
use std::path::{Path, PathBuf};
use tera::Tera;
use tracing::debug;

#[derive(Debug)]
pub struct Site {
    path: PathBuf,
    tera: Tera,
    conn: Connection,
}

impl Site {
    pub fn new(path: PathBuf, tera: Tera, conn: Connection) -> Self {
        Self { path, tera, conn }
    }

    #[tracing::instrument]
    pub fn build(&self) -> Result<()> {
        let content_dir = self.path.join("contents");
        let sass_dir = self.path.join("sass");
        let static_dir = self.path.join("static");

        self.process_entry::<BlogContent>(&content_dir)?;
        self.process_entry::<Stylesheet>(&sass_dir)?;
        self.process_entry::<StaticAsset>(&static_dir)?;

        Ok(())
    }

    #[tracing::instrument]
    fn discover_entries<T: Entry>(&self, path: &Path) -> Vec<T> {
        // Discover entries
        debug!("Discovering entries at {:?}", path);
        Walk::new(path)
            .filter_map(Result::ok)
            .map(DirEntry::into_path)
            .filter(|p| !p.is_dir())
            .map(T::from_file)
            .collect()
    }

    #[tracing::instrument]
    fn process_entry<T: Entry>(&self, directory: &Path) -> Result<()> {
        debug!("Processing entries at {:?}", directory);
        let entries = self.discover_entries::<T>(directory);
        let build_statuses = entries
            .iter()
            .map(|p| p.build_status(&self.conn))
            .collect::<Result<Vec<BuildStatus>>>()?;

        build_statuses
            .into_iter()
            .zip(entries)
            .map(|(status, post)| post.build(&self.conn, &self.tera, status))
            .collect::<Result<Vec<()>>>()?;

        Ok(())
    }
}
