#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]

pub mod filesystem;
pub mod summary;

use color_eyre::eyre::Result;
use rusqlite::Connection;
use std::path::PathBuf;
use tera::Tera;

pub const DATE_FORMAT: &str = "%b %e, %Y";

/// Whether an entry has been newly added, if it existed but was changed, or if existed and remained unchanged.
#[derive(Debug)]
pub enum BuildStatus {
    New(String),
    Changed(String),
    Unchanged,
}

// Describes common behavior for an entry.
//
// An entry is the main unit the static site generator works with. It can be a markdown file, stylesheet, or some other static asset.
pub trait Entry {
    fn from_file(path: PathBuf) -> Self;
    fn build_status(&self, conn: &Connection) -> Result<BuildStatus>;
    fn hash(&self) -> Result<String>;
    fn build(&self, conn: &Connection, tera: &Tera, status: BuildStatus) -> Result<()>;
}
