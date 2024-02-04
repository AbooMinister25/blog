#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_errors_doc)]

use std::fmt::Debug;
use std::path::{Path, PathBuf};

use color_eyre::eyre::ContextCompat;
use color_eyre::Result;
use content::{render_page, Page};
use entry::{discover_entries, Entry};
use markdown::MarkdownRenderer;
use rusqlite::Connection;
use sass::Stylesheet;
use static_assets::StaticAsset;
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
    ctx: Context,
    root: PathBuf,
    output_path: PathBuf,
    discovered_posts: Vec<Entry>,
    pub pages: Vec<Page>,
    pub stylesheets: Vec<Stylesheet>,
    pub static_assets: Vec<StaticAsset>,
}

impl Site {
    #[tracing::instrument]
    pub fn new<P: AsRef<Path> + Debug>(conn: Connection, path: P, output_path: P) -> Result<Self> {
        let renderer = MarkdownRenderer::new(&path)?;

        info!("Loaded templates");
        let tera = Tera::new(
            path.as_ref()
                .join("templates/**/*.tera")
                .to_str()
                .context("Filename should be valid UTF-8")?,
        )?;
        let ctx = Context::new(conn, renderer, tera);

        Ok(Self {
            ctx,
            root: path.as_ref().to_path_buf(),
            output_path: output_path.as_ref().to_path_buf(),
            discovered_posts: Vec::new(),
            stylesheets: Vec::new(),
            static_assets: Vec::new(),
            pages: Vec::new(),
        })
    }

    #[tracing::instrument(skip(self))]
    pub fn build(&mut self) -> Result<()> {
        self.discover()?;
        self.render()?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn discover(&mut self) -> Result<()> {
        info!("Discovering entries");
        let entries = discover_entries(&self.ctx.conn, &self.root)?;

        for entry in entries {
            match entry.path.extension() {
                Some(e) => match e.to_string_lossy().as_ref() {
                    "md" => self.discovered_posts.push(entry),
                    "scss" | "sass" => self.stylesheets.push(Stylesheet::from(entry)),
                    "png" | "ico" | "webmanifest" | "svg" | "woff2" => {
                        self.static_assets.push(StaticAsset::from(entry));
                    }
                    _ => continue,
                },
                _ => continue,
            }
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn render(&mut self) -> Result<()> {
        let pages = self
            .discovered_posts
            .iter_mut()
            .map(|p| {
                render_page(
                    &self.ctx.tera,
                    &self.ctx.markdown_renderer,
                    &p.path,
                    &self.output_path,
                    String::from_utf8_lossy(&p.raw_content).to_string(),
                )
            })
            .collect::<Result<Vec<Page>>>()?;

        self.pages = pages;

        let _ = self
            .stylesheets
            .iter_mut()
            .map(|s| s.render(&self.output_path))
            .collect::<Result<Vec<()>>>()?;

        let _ = self
            .static_assets
            .iter_mut()
            .map(|a| a.render(&self.output_path))
            .collect::<Result<Vec<()>>>()?;

        Ok(())
    }
}
