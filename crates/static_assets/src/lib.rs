use color_eyre::eyre::{ContextCompat, Result};
use entry::filesystem::ensure_directory;
use entry::{BuildStatus, Entry};
use rusqlite::Connection;
use sql::insert_asset;
use std::path::Path;
use std::{fs, path::PathBuf};
use tera::Tera;
use tracing::debug;

// Represents a static asset, stored in the `static/` directory.
// For the most part, they're copied over to the resulting site
// as-is. Their hashes are stored in the sqlite database so time
// isn't wasted with copying existing and unchanged files over.
#[derive(Debug)]
pub struct StaticAsset {
    pub path: PathBuf,
}

impl StaticAsset {
    fn directory(&self) -> Result<&str> {
        let parent = self.path.parent().unwrap(); // All static entries will have a parent directory
        return Ok(if parent == Path::new("/static") {
            ""
        } else {
            parent
                .file_name()
                .context("Path shouldn't terminate in ..")?
                .to_str()
                .context("File name should be valid UTF-8")?
        });
    }

    fn copy_asset(&self) -> Result<()> {
        debug!(
            "Copying asset at {}",
            self.path.to_str().context("Path should be valid unicode")?
        );

        let path = Path::new("public/static").join(self.directory()?).join(
            self.path
                .file_name()
                .context("File should have a name")?
                .to_str()
                .context("Filename should be valid unicode")?,
        );

        fs::copy(&self.path, path)?;
        Ok(())
    }
}

impl Entry for StaticAsset {
    #[tracing::instrument]
    fn from_file(path: PathBuf) -> Self {
        Self { path }
    }

    #[tracing::instrument]
    fn build_status(&self, conn: &Connection) -> Result<BuildStatus> {
        let asset_hash = self.hash()?;

        let mut stmt = conn.prepare("SELECT hash FROM static_assets WHERE path = :path")?;
        let path_str = self
            .path
            .to_str()
            .context("Error while converting path to string")?;

        // Get the hashes found for this path
        let hashes_iter = stmt.query_map(&[(":path", path_str)], |row| row.get(0))?;
        let mut hashes: Vec<String> = Vec::new();
        for hash in hashes_iter {
            hashes.push(hash?);
        }

        // If the hashes are empty, a new file was created. If it was different from the new
        // hash, then the file contents changed. Otherwise the file was not changed.
        let build = if hashes.is_empty() {
            BuildStatus::New(asset_hash)
        } else if hashes[0] != asset_hash {
            BuildStatus::Changed(asset_hash)
        } else {
            BuildStatus::Unchanged
        };

        Ok(build)
    }

    #[tracing::instrument]
    fn hash(&self) -> Result<String> {
        let raw_asset = fs::read(&self.path)?;
        // Hash asset, format as string
        Ok(format!("{:016x}", seahash::hash(&raw_asset)))
    }

    #[tracing::instrument(skip(_tera))]
    fn build(&self, conn: &Connection, _tera: &Tera, status: BuildStatus) -> Result<()> {
        ensure_directory(Path::new("public/static").join(self.directory()?))?;

        let status = self.build_status(conn)?;
        match status {
            BuildStatus::New(asset_hash) => {
                insert_asset(conn, &self.path, &asset_hash)?;
                self.copy_asset()?;
            }
            BuildStatus::Changed(asset_hash) => {
                conn.execute(
                    "UPDATE static_assets SET hash = (:hash) WHERE path = (:path)",
                    &[
                        (":hash", &asset_hash),
                        (
                            ":path",
                            &self
                                .path
                                .to_str()
                                .context("Path should be valid unicode")?
                                .to_string(),
                        ),
                    ],
                )?;
                self.copy_asset()?
            }
            _ => (), // Don't do anything if the file was unchanged
        }

        Ok(())
    }
}
