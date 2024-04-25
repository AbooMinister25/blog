#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_errors_doc)]

use std::fmt::Debug;

use color_eyre::eyre::ContextCompat;
use color_eyre::Result;
use config::Config;
use content::page::{render_page, Page};
use entry::{discover_entries, Entry};
use markdown::MarkdownRenderer;
use rusqlite::Connection;
use sass::Stylesheet;
use static_assets::StaticAsset;
use std::collections::HashSet;
use tera::Tera;
use tracing::info;

pub mod index;

use crate::index::Index;

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
    config: Config,
    discovered_posts: Vec<Entry>,
    pub pages: Index,
    pub stylesheets: Vec<Stylesheet>,
    pub static_assets: Vec<StaticAsset>,
}

impl Site {
    #[tracing::instrument]
    pub fn new(conn: Connection, config: Config) -> Result<Self> {
        let renderer = MarkdownRenderer::new(&config.root)?;

        info!("Loaded templates");
        let tera = Tera::new(
            config
                .root
                .join("templates/**/*.tera")
                .to_str()
                .context("Filename should be valid UTF-8")?,
        )?;
        let ctx = Context::new(conn, renderer, tera);

        Ok(Self {
            ctx,
            config,
            discovered_posts: Vec::new(),
            stylesheets: Vec::new(),
            static_assets: Vec::new(),
            pages: Index::default(),
        })
    }

    #[tracing::instrument(skip(self))]
    pub fn build(&mut self) -> Result<()> {
        self.discover()?;
        self.render()?;

        self.pages.build_index(&self.config.output_path)?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    fn discover(&mut self) -> Result<()> {
        info!("Discovering entries");
        let entries = discover_entries(&self.ctx.conn, &self.config.root)?;

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
                    &self.config.output_path,
                    String::from_utf8_lossy(&p.raw_content).to_string(),
                    &p.hash,
                )
            })
            .collect::<Result<HashSet<Page>>>()?;

        self.pages = Index::from(pages);

        let _ = self
            .stylesheets
            .iter_mut()
            .map(|s| s.render(&self.config.output_path))
            .collect::<Result<Vec<()>>>()?;

        let _ = self
            .static_assets
            .iter_mut()
            .map(|a| a.render(&self.config.output_path))
            .collect::<Result<Vec<()>>>()?;

        Ok(())
    }
}
