mod config;
mod context;
mod entry;
mod output;
mod page;
pub mod sql;
mod utils;

use color_eyre::{eyre::ContextCompat, Result};
use config::Config;
use context::Context;
use entry::{discover_entries, Entry};
use markdown::MarkdownRenderer;
use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use tera::Tera;
use tracing::info;

pub const DATE_FORMAT: &str = "%b %e, %Y";

/// Represents a site, and holds all the pages that are currently being worked on.
pub struct Site<'c> {
    ctx: Context<'c>,
    entries: Vec<Entry>,
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
            entries: Vec::new(),
        })
    }

    /// Build the site.
    #[tracing::instrument(skip(self))]
    pub fn build(&mut self) -> Result<()> {
        info!("Discovering entries");
        self.entries = discover_entries(&self.ctx)?;

        Ok(())
    }
}
