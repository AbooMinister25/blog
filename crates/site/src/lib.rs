mod asset;
mod config;
mod context;
mod entry;
mod output;
mod page;
pub mod sql;
mod static_file;
mod utils;

use std::{ffi::OsStr, path::Component};

use asset::Asset;
use color_eyre::{eyre::ContextCompat, Result};
use config::Config;
use context::Context;
use entry::discover_entries;
use markdown::MarkdownRenderer;
use output::Output;
use page::Page;
use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use static_file::StaticFile;
use tera::Tera;
use tracing::info;

pub const DATE_FORMAT: &str = "%b %e, %Y";

/// Represents a site, and holds all the pages that are currently being worked on.
pub struct Site<'c> {
    ctx: Context<'c>,
    pages: Vec<Page>,
    assets: Vec<Asset>,
    static_files: Vec<StaticFile>,
}

impl<'c> Site<'c> {
    #[tracing::instrument]
    pub fn new(conn: PooledConnection<SqliteConnectionManager>, config: Config) -> Result<Self> {
        let renderer = MarkdownRenderer::new(&config.root, &config.theme)?;

        let tera = Tera::new(
            config
                .root
                .join("templates/**/*.tera")
                .to_str()
                .context("Filename should be valid UTF-8")?,
        )?;
        info!("Loaded templates");

        let ctx = Context::new(conn, tera, renderer, config);

        Ok(Self {
            ctx,
            pages: Vec::new(),
            assets: Vec::new(),
            static_files: Vec::new(),
        })
    }

    /// Build the site.
    #[tracing::instrument(skip(self))]
    pub fn build(&mut self) -> Result<()> {
        info!("Discovering entries");
        self.discover_and_process()?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    fn discover_and_process(&mut self) -> Result<()> {
        let entries = discover_entries(&self.ctx)?;

        info!("Processing entries");
        for entry in entries {
            let content = String::from_utf8(entry.raw_content)?;
            match entry.path.parent().and_then(|p| {
                p.components()
                    .nth(1)
                    .map(Component::as_os_str)
                    .and_then(OsStr::to_str)
            }) {
                Some("content") => {
                    let page = Page::new(&self.ctx, entry.path, content, entry.hash, entry.fresh)?;
                    self.pages.push(page);
                }
                Some("assets") => {
                    let asset =
                        Asset::new(&self.ctx, entry.path, content, entry.hash, entry.fresh)?;
                    self.assets.push(asset);
                }
                Some("static") => {
                    let static_file =
                        StaticFile::new(&self.ctx, entry.path, entry.hash, entry.fresh)?;
                    self.static_files.push(static_file);
                }
                _ => continue,
            }
        }
        info!("Processed entries");

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    fn render(&mut self) -> Result<()> {
        let (special_pages, posts): (Vec<_>, Vec<_>) =
            self.pages.iter().partition(|p| p.is_special);

        for output in posts
            .iter()
            .map(|p| *p as &dyn Output)
            .chain(self.assets.iter().map(|a| a as &dyn Output))
            .chain(self.static_files.iter().map(|s| s as &dyn Output))
        {
            output.write(&self.ctx)?;
        }

        Ok(())
    }
}
