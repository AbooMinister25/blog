use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub url: String,
    pub root: PathBuf,
    pub output_path: PathBuf,
    pub log_folder: PathBuf,
    pub development: bool,
    pub special_pages: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            url: String::from("http://0.0.0.0:8000/"),
            root: Path::new("blog/").to_owned(),
            output_path: Path::new("public/").to_owned(),
            log_folder: Path::new("log/").to_owned(),
            development: false,
            special_pages: vec![
                "index.md".to_string(),
                "search.md".to_string(),
                "about.md".to_string(),
            ],
        }
    }
}
