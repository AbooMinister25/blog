#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_errors_doc)]

use std::fmt::Debug;
use std::path::{Path, PathBuf};

use color_eyre::eyre::ContextCompat;
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
    pub output_path: PathBuf,
    pub posts: Vec<Post>,
    pub stylesheets: Vec<Stylesheet>,
}

impl Site {
    #[tracing::instrument]
    pub fn new<P: AsRef<Path> + Debug>(conn: Connection, path: P, output_path: P) -> Result<Self> {
        let renderer = MarkdownRenderer::new()?;

        info!("Loaded templates");
        let tera = Tera::new("templates/**/*.tera")?;
        let ctx = Context::new(conn, renderer, tera);

        Ok(Self {
            ctx,
            root: path.as_ref().to_path_buf(),
            output_path: path.as_ref().to_path_buf(),
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
            match entry
                .path
                .extension()
                .context("File name should have an extension.")?
                .to_str()
                .context("File name should be valid unicode")?
            {
                ".md" => posts.push(Post::from(entry)),
                ".scss" => stylesheets.push(Stylesheet::from(entry)),
                _ => todo!(),
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
            .map(|p| p.render(&self.ctx.tera, &self.ctx.markdown_renderer, &self.output_path))
            .collect::<Result<Vec<()>>>()?;

        let _ = self
            .stylesheets
            .iter_mut()
            .map(|s| s.render(&self.output_path))
            .collect::<Result<Vec<()>>>()?;

        Ok(())
    }

    #[tracing::instrument]
    pub fn build(&mut self) -> Result<()> {
        self.discover()?;
        self.build()?;

        Ok(())
    }
}
