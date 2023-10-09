#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_panics_doc)]

use std::path::PathBuf;

// A stylesheet
#[derive(Debug)]
pub struct Stylesheet {
    pub path: PathBuf,
}
