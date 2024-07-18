use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Configuration values for the site.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub url: String,
    pub root: PathBuf,
    pub output_path: PathBuf,
    pub log_path: PathBuf,
    pub development: bool,
    pub special_pages: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            url: String::from("http://0.0.0.0:8000/"),
            root: Path::new("blog/").to_owned(),
            output_path: Path::new("public/").to_owned(),
            log_path: Path::new("log/").to_owned(),
            development: false,
            special_pages: vec![
                "index.md".to_string(),
                "search.md".to_string(),
                "about.md".to_string(),
                "500.md".to_string(),
                "404.md".to_string(),
            ],
        }
    }
}
