mod config;
mod context;

use color_eyre::{eyre::ContextCompat, Result};
use config::Config;
use context::Context;
use markdown::MarkdownRenderer;
use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use tera::Tera;
use tracing::info;

/// Represents a site, and holds all the pages that are currently being worked on.
pub struct Site<'c> {
    ctx: Context<'c>,
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

        Ok(Self { ctx })
    }
}
