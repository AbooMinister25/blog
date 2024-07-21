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
    outputs: Vec<Box<dyn Output>>,
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
            outputs: Vec::new(),
        })
    }

    /// Build the site.
    #[tracing::instrument(skip(self))]
    pub fn build(&mut self) -> Result<()> {
        info!("Discovering entries");
        self.discover()?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    fn discover(&mut self) -> Result<()> {
        let entries = discover_entries(&self.ctx)?;

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
                    self.outputs.push(Box::new(page));
                }
                Some("assets") => {
                    let asset =
                        Asset::new(&self.ctx, entry.path, content, entry.hash, entry.fresh)?;
                    self.outputs.push(Box::new(asset));
                }
                Some("static") => {
                    let static_file =
                        StaticFile::new(&self.ctx, entry.path, entry.hash, entry.fresh)?;
                    self.outputs.push(Box::new(static_file));
                }
                _ => continue,
            }
        }

        Ok(())
    }
}
