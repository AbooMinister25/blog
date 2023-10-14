#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_errors_doc)]

use std::fmt::Debug;
use std::path::{Path, PathBuf};

use color_eyre::Result;
use content::Post;
use entry::discover_entries;
use markdown::MarkdownRenderer;
use rusqlite::Connection;
use sass::Stylesheet;
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
    pub root: PathBuf,
    pub posts: Vec<Post>,
    pub stylesheets: Vec<Stylesheet>,
}

impl Site {
    #[tracing::instrument]
    pub fn new<P: AsRef<Path> + Debug>(conn: Connection, path: P) -> Result<Self> {
        let renderer = MarkdownRenderer::new()?;

        info!("Loaded templates");
        let tera = Tera::new("templates/**/*.tera")?;
        let ctx = Context::new(conn, renderer, tera);

        Ok(Self {
            ctx,
            root: path.as_ref().to_path_buf(),
            posts: Vec::new(),
            stylesheets: Vec::new(),
        })
    }

    #[tracing::instrument]
    pub fn discover(&mut self) -> Result<()> {
        info!("Discovering entries");
        let entries = discover_entries(&self.ctx.conn, &self.root)?;

        let mut posts = Vec::new();
        let mut stylesheets = Vec::new();

        for entry in entries {
            if entry.path.extension().is_some_and(|e| e == ".md") {
                posts.push(Post::from(entry));
            } else if entry.path.extension().is_some_and(|e| e == ".scss") {
                stylesheets.push(Stylesheet::from(entry));
            }
        }

        self.posts = posts;
        self.stylesheets = stylesheets;

        Ok(())
    }

    #[tracing::instrument]
    pub fn render(&mut self) -> Result<()> {
        let _ = self
            .posts
            .iter_mut()
            .map(|p| p.render(&self.ctx.tera, &self.ctx.markdown_renderer, &self.root))
            .collect::<Result<Vec<()>>>()?;

        let _ = self
            .stylesheets
            .iter_mut()
            .map(|s| s.render(&self.root))
            .collect::<Result<Vec<()>>>()?;

        Ok(())
    }

    #[tracing::instrument]
    pub fn build(&mut self) -> Result<()> {
        for post in &mut self.posts {
            post.render(&self.ctx.tera, &self.ctx.markdown_renderer, "public/")?;
        }

        Ok(())
    }
}
