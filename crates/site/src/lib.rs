#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_errors_doc)]

use std::fs;
use std::path::Component;
use std::{ffi::OsStr, fmt::Debug};

use assets::Asset;
use chrono::Utc;
use color_eyre::eyre::ContextCompat;
use color_eyre::Result;
use config::Config;
use content::page::{render_page, write_index_to_disk, write_page_to_disk, Page};
use entry::{discover_entries, Entry};
use markdown::MarkdownRenderer;
use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use sql::{get_posts, insert_post, update_post, PostSQL};
use static_files::StaticFile;
use std::collections::HashSet;
use tera::{Context as TeraContext, Tera};
use tracing::{info, trace};

pub mod index;
mod tera_functions;

use crate::index::Index;

#[derive(Debug)]
pub struct Context {
    pub conn: PooledConnection<SqliteConnectionManager>,
    pub markdown_renderer: MarkdownRenderer,
    pub tera: Tera,
    pub config: Config,
}

impl Context {
    pub fn new(
        conn: PooledConnection<SqliteConnectionManager>,
        markdown_renderer: MarkdownRenderer,
        tera: Tera,
        config: Config,
    ) -> Self {
        Self {
            conn,
            markdown_renderer,
            tera,
            config,
        }
    }
}

/// Represents the static site, and hold all its pages.
#[derive(Debug)]
pub struct Site {
    pub ctx: Context,
    entries: Vec<Entry>,
    discovered_posts: Vec<Entry>,
    pub working_index: Index,
    pub static_files: Vec<StaticFile>,
    pub assets: Vec<Asset>,
}

impl Site {
    #[tracing::instrument]
    pub fn new(conn: PooledConnection<SqliteConnectionManager>, config: Config) -> Result<Self> {
        let renderer = MarkdownRenderer::new(&config.root)?;

        info!("Loaded templates");
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
        let ctx = Context::new(conn, renderer, tera, config);

        Ok(Self {
            ctx,
            entries: Vec::new(),
            discovered_posts: Vec::new(),
            static_files: Vec::new(),
            assets: Vec::new(),
            working_index: Index::default(),
        })
    }

    #[tracing::instrument(skip(self))]
    pub fn build(&mut self) -> Result<()> {
        self.discover()?;

        let (special_pages, posts) = self.build_entries()?;
        let index = self.render(posts, special_pages)?;

        self.build_atom_feed(&index)?;
        self.build_sitemap(index)?;

        self.reset();

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn load_index(&mut self) -> Result<Vec<Page>> {
        let posts = get_posts(&self.ctx.conn)?
            .into_iter()
            .map(Page::from)
            .collect::<Vec<Page>>();

        Ok(posts)
    }

    #[tracing::instrument(skip(self))]
    fn discover(&mut self) -> Result<()> {
        info!("Discovering entries");

        let entries = discover_entries(
            &self.ctx.conn,
            &self.ctx.config.root,
            &self.ctx.config.special_pages,
        )?;

        for entry in entries {
            match entry.path.parent().and_then(|p| {
                p.components()
                    .nth(1)
                    .map(Component::as_os_str)
                    .and_then(OsStr::to_str)
            }) {
                Some("content") => self.discovered_posts.push(entry),
                Some("assets") => self.assets.push(Asset::from(entry)),
                Some("static") => self.static_files.push(StaticFile::from(entry)),
                _ => continue,
            }
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    fn build_entries(&mut self) -> Result<(Vec<Page>, Vec<Page>)> {
        // Render all pages and separate them into either special pages or normal posts.
        let pages = self
            .discovered_posts
            .iter_mut()
            .map(|p| {
                render_page(
                    &self.ctx.markdown_renderer,
                    &self.ctx.config.url,
                    &p.path,
                    &self.ctx.config.output_path,
                    String::from_utf8_lossy(&p.raw_content).to_string(),
                    &p.hash,
                    p.new,
                    &self.ctx.config.special_pages,
                )
            })
            .collect::<Result<HashSet<Page>>>()?;

        let mut special_pages = Vec::new();
        let mut posts = Vec::new();

        for page in pages.into_iter().filter(|p| {
            p.frontmatter.as_ref().is_some_and(|f| !f.draft) || self.ctx.config.development
        }) {
            if page.index {
                special_pages.push(page);
            } else {
                posts.push(page);
            }
        }

        Ok((special_pages, posts))
    }

    #[tracing::instrument(skip(self))]
    fn render(&mut self, posts: Vec<Page>, mut special_pages: Vec<Page>) -> Result<Vec<Page>> {
        let written_posts = posts
            .into_iter()
            .map(|p| write_page_to_disk(&self.ctx.tera, p))
            .collect::<Result<HashSet<Page>>>()?;

        self.working_index = Index::from(written_posts);
        self.working_index
            .build_index(&self.ctx.config.output_path)?;
        self.update_db()?;

        // TODO: Have a separate building step for assets and static files too, so that errors are
        // TODO: caught before writing anything to disk.
        self.assets
            .iter_mut()
            .map(|s| s.render(&self.ctx.config.output_path))
            .collect::<Result<Vec<()>>>()?;

        self.static_files
            .iter_mut()
            .map(|a| a.render(&self.ctx.config.output_path))
            .collect::<Result<Vec<()>>>()?;

        // Build index pages.
        special_pages.sort_by(|a, b| b.date.cmp(&a.date));
        let mut index = self.load_index()?;
        special_pages
            .iter()
            .map(|p| write_index_to_disk(&self.ctx.tera, p, &index, &special_pages))
            .collect::<Result<Vec<()>>>()?;
        index.append(&mut special_pages);

        Ok(index)
    }

    #[tracing::instrument(skip(self))]
    fn update_db(&mut self) -> Result<()> {
        for page in self.working_index.working_pages.iter().filter(|p| !p.index) {
            let post_sql = PostSQL::new(
                &page.path,
                &page.permalink,
                &page.title,
                &page.tags,
                &page.date,
                &page.updated,
                &page.summary,
                &page.hash,
                page.new,
            );

            if page.new {
                insert_post(&self.ctx.conn, post_sql)?;
            } else {
                update_post(&self.ctx.conn, post_sql)?;
            }
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    fn build_atom_feed(&mut self, pages: &[Page]) -> Result<()> {
        trace!("Generating Atom feed");
        let template = "atom.xml.tera";
        let out_path = self.ctx.config.output_path.join("atom.xml");
        let last_updated = Utc::now();
        let mut context = TeraContext::new();

        context.insert("feed_url", &format!("{}atom.xml", self.ctx.config.url));
        context.insert("base_url", &self.ctx.config.url);
        context.insert("last_updated", &last_updated);
        context.insert("pages", &pages);

        let rendered = self.ctx.tera.render(template, &context)?;
        fs::write(out_path, rendered)?;

        trace!("Generated Atom feed.");

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    fn build_sitemap(&mut self, pages: Vec<Page>) -> Result<()> {
        trace!("Generating sitemap");

        let template = "sitemap.xml.tera";
        let out_path = self.ctx.config.output_path.join("sitemap.xml");
        let mut context = TeraContext::new();
        context.insert("pages", &pages);

        let rendered = self.ctx.tera.render(template, &context)?;
        fs::write(out_path, rendered)?;

        trace!("Generated sitemap");

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    fn reset(&mut self) {
        self.discovered_posts.clear();
        self.static_files.clear();
        self.assets.clear();
        self.working_index.clear();
    }
}
