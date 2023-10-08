#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_errors_doc)]

use std::fmt::Debug;
use std::path::{Path, PathBuf};

use color_eyre::Result;
use content::Post;
use markdown::MarkdownRenderer;
use rusqlite::Connection;
use tera::Tera;
use tracing::info;

#[derive(Debug)]
pub struct Context {
    pub conn: Connection,
    pub markdown_renderer: MarkdownRenderer,
    pub tera: Tera,
}

impl Context {
    pub fn new(conn: Connection, markdown_renderer: MarkdownRenderer, tera: Tera) -> Self {
        Self {
            conn,
            markdown_renderer,
            tera,
        }
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
    #[tracing::instrument]
    pub fn new<P: AsRef<Path> + Debug>(conn: Connection, path: P) -> Result<Self> {
        let entries = content::discover_entries(&conn, &path)?;
        let posts = entries.into_iter().map(Post::from).collect::<Vec<Post>>();

        let renderer = MarkdownRenderer::new()?;
        info!("Loaded templates");
        let tera = Tera::new("templates/**/*.tera")?;
        let ctx = Context::new(conn, renderer, tera);

        Ok(Self {
            ctx,
            site_path: path.as_ref().to_path_buf(),
            posts,
        })
    }

    #[tracing::instrument]
    pub fn build(&mut self) -> Result<()> {
        for post in &mut self.posts {
            post.render(&self.ctx.tera, &self.ctx.markdown_renderer, "public/")?;
        }

        Ok(())
    }
}
