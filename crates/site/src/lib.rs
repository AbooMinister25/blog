mod asset;
pub mod config;
mod context;
mod entry;
mod incremental_index;
mod output;
mod page;
mod shortcodes;
pub mod sql;
mod static_file;
mod tera_functions;
pub mod utils;

use std::{ffi::OsStr, fs, path::Component};

use asset::Asset;
use chrono::Utc;
use color_eyre::{eyre::ContextCompat, Result};
use config::Config;
use context::Context;
use entry::discover_entries;
use incremental_index::IncrementalIndex;
use markdown::MarkdownRenderer;
use output::Output;
use page::Page;
use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use sql::{get_posts, insert_entry, insert_post, update_entry_hash, update_post};
use static_file::StaticFile;
use tera::{Context as TeraContext, Tera};
use tracing::info;

pub const DATE_FORMAT: &str = "%b %e, %Y";

/// Represents a site, and holds all the pages that are currently being worked on.
pub struct Site<'c> {
    ctx: Context<'c>,
    assets: Vec<Asset>,
    static_files: Vec<StaticFile>,
    incremental_index: IncrementalIndex,
}

impl<'c> Site<'c> {
    #[tracing::instrument(level = tracing::Level::DEBUG)]
    pub fn new(conn: PooledConnection<SqliteConnectionManager>, config: Config) -> Result<Self> {
        let renderer = MarkdownRenderer::new(&config.root, &config.theme)?;

        let mut tera = Tera::new(
            config
                .root
                .join("templates/**/*.tera")
                .to_str()
                .context("Filename should be valid UTF-8")?,
        )?;
        tera.register_function(
            "posts_in_series",
            tera_functions::make_posts_in_series(config.output_path.clone()),
        );
        tera.register_function(
            "get_series_indexes",
            tera_functions::make_get_series_indexes(config.output_path.clone()),
        );
        info!("Loaded templates");

        let ctx = Context::new(conn, tera, renderer, config);

        Ok(Self {
            ctx,
            assets: Vec::new(),
            static_files: Vec::new(),
            incremental_index: IncrementalIndex::default(),
        })
    }

    /// Build the site.
    #[tracing::instrument(level = tracing::Level::DEBUG, skip(self))]
    pub fn build(&mut self) -> Result<()> {
        info!("Discovering entries");
        let pages = self.discover_and_process()?;

        info!("Rendering entries");
        self.render(pages)?;
        info!("Rendered entries");

        info!("Building search index");
        self.incremental_index.build_index(&self.ctx)?;
        info!("Built search index");

        info!("Generating atom feed");
        self.generate_atom_feed()?;
        info!("Generated atom feed");

        info!("Generating sitemap");
        self.generate_sitemap()?;
        info!("Generated sitemap");

        Ok(())
    }

    #[tracing::instrument(level = tracing::Level::DEBUG, skip(self))]
    fn discover_and_process(&mut self) -> Result<Vec<Page>> {
        let entries = discover_entries(&self.ctx)?;
        let mut pages = Vec::new();

        info!("Processing entries");
        for entry in entries {
            match entry.path.parent().and_then(|p| {
                p.components()
                    .nth(1)
                    .map(Component::as_os_str)
                    .and_then(OsStr::to_str)
            }) {
                Some("content") => {
                    let content = String::from_utf8(entry.raw_content)?;
                    let page = Page::new(&self.ctx, entry.path, content, entry.hash, entry.fresh)?;
                    pages.push(page);
                }
                Some("assets") => {
                    let asset = Asset::new(
                        &self.ctx,
                        entry.path,
                        entry.raw_content,
                        entry.hash,
                        entry.fresh,
                    )?;
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

        Ok(pages)
    }

    #[tracing::instrument(level = tracing::Level::DEBUG, skip(self))]
    fn render(&mut self, pages: Vec<Page>) -> Result<()> {
        let (mut special_pages, posts): (Vec<_>, Vec<_>) = pages
            .into_iter()
            .filter(|p| !p.document.frontmatter.draft || self.ctx.config.development)
            .partition(|p| p.is_special);
        special_pages.sort_by(|a, b| b.document.date.cmp(&a.document.date));

        for output in posts
            .iter()
            .map(|p| p as &dyn Output)
            .chain(self.assets.iter().map(|a| a as &dyn Output))
            .chain(self.static_files.iter().map(|s| s as &dyn Output))
        {
            output.write(&self.ctx)?;
        }

        self.incremental_index = IncrementalIndex::from(posts);

        self.update_db()?;
        let posts_in_index = get_posts(&self.ctx.conn)?;
        self.ctx.posts = Some(posts_in_index);

        self.ctx.special_pages = Some(special_pages.clone());

        for page in special_pages {
            page.write(&self.ctx)?;
        }

        Ok(())
    }

    #[tracing::instrument(level = tracing::Level::DEBUG, skip(self))]
    fn update_db(&mut self) -> Result<()> {
        for output in self
            .incremental_index
            .posts
            .iter()
            .map(|p| p as &dyn Output)
            .chain(self.assets.iter().map(|a| a as &dyn Output))
            .chain(self.static_files.iter().map(|s| s as &dyn Output))
            .chain(
                self.ctx
                    .special_pages
                    .as_ref()
                    .unwrap_or(&Vec::new())
                    .iter()
                    .map(|p| p as &dyn Output),
            )
        {
            if output.fresh() {
                insert_entry(&self.ctx.conn, output.path(), output.hash())?;
            } else {
                update_entry_hash(&self.ctx.conn, output.path(), output.hash())?;
            }
        }

        for page in self
            .incremental_index
            .posts
            .iter()
            .filter(|p| !p.is_special)
        {
            if page.fresh {
                insert_post(&self.ctx.conn, page)?;
            } else {
                update_post(&self.ctx.conn, page)?;
            }
        }

        Ok(())
    }

    #[tracing::instrument(level = tracing::Level::DEBUG, skip(self))]
    fn generate_atom_feed(&mut self) -> Result<()> {
        let template = "atom.xml.tera";
        let out_path = self.ctx.config.output_path.join("atom.xml");
        let last_updated = Utc::now();
        let mut context = TeraContext::new();

        context.insert("feed_url", &format!("{}atom.xml", self.ctx.config.url));
        context.insert("base_url", &self.ctx.config.url);
        context.insert("last_updated", &last_updated);
        context.insert("pages", &self.incremental_index.posts);

        let rendered = self.ctx.tera.render(template, &context)?;
        fs::write(out_path, rendered)?;
        Ok(())
    }

    #[tracing::instrument(level = tracing::Level::DEBUG, skip(self))]
    fn generate_sitemap(&mut self) -> Result<()> {
        let sp = Vec::new();
        let pages = self
            .incremental_index
            .posts
            .iter()
            .chain(self.ctx.special_pages.as_ref().unwrap_or(&sp))
            .collect::<Vec<&Page>>();

        let template = "sitemap.xml.tera";
        let out_path = self.ctx.config.output_path.join("sitemap.xml");
        let mut context = TeraContext::new();
        context.insert("pages", &pages);

        let rendered = self.ctx.tera.render(template, &context)?;
        fs::write(out_path, rendered)?;

        Ok(())
    }
}
