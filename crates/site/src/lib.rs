#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_errors_doc)]

use std::fmt::Debug;
use std::path::{Path, PathBuf};

use color_eyre::Result;
use content::Post;
use rusqlite::Connection;

#[derive(Debug)]
pub struct Context {
    pub conn: Connection,
}

impl Context {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
}

/// Represents the static site, and hold all its pages.
#[derive(Debug)]
pub struct Site {
    pub ctx: Context,
    pub site_path: PathBuf,
    pub posts: Vec<Post>,
}

impl Site {
    pub fn new<P: AsRef<Path> + Debug>(conn: Connection, path: P) -> Result<Self> {
        let entries = content::discover_entries(&conn, &path)?;
        let posts = entries.into_iter().map(Post::from).collect::<Vec<Post>>();

        let ctx = Context::new(conn);

        Ok(Self {
            ctx,
            site_path: path.as_ref().to_path_buf(),
            posts,
        })
    }
}
