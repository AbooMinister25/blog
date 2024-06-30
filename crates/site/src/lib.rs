#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_errors_doc)]

use std::path::Component;
use std::{ffi::OsStr, fmt::Debug};

use assets::Asset;
use color_eyre::eyre::ContextCompat;
use color_eyre::Result;
use config::Config;
use content::page::{render_page, write_index_to_disk, write_page_to_disk, Page};
use entry::{discover_entries, Entry};
use markdown::MarkdownRenderer;
use rusqlite::Connection;
use sql::{get_posts, insert_post, update_post, PostSQL};
use static_files::StaticFile;
use std::collections::HashSet;
use tera::Tera;
use tracing::info;

pub mod index;
mod tera_functions;

use crate::index::Index;

#[derive(Debug)]
pub struct Context {
    pub conn: Connection,
    pub markdown_renderer: MarkdownRenderer,
    pub tera: Tera,
    pub config: Config,
}

impl Context {
    pub fn new(
        conn: Connection,
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
    ctx: Context,
    discovered_posts: Vec<Entry>,
    pub working_index: Index,
    pub static_files: Vec<StaticFile>,
    pub assets: Vec<Asset>,
}

impl Site {
    #[tracing::instrument]
    pub fn new(conn: Connection, config: Config) -> Result<Self> {
        let renderer = MarkdownRenderer::new(&config.root)?;

        info!("Loaded templates");
        let mut tera = Tera::new(
            config
                .root
                .join("templates/**/*.tera")
                .to_str()
                .context("Filename should be valid UTF-8")?,
        )?;
        tera.register_function("posts_in_series", tera_functions::posts_in_series);
        tera.register_function("get_series_indexes", tera_functions::get_series_indexes);
        let ctx = Context::new(conn, renderer, tera, config);

        Ok(Self {
            ctx,
            discovered_posts: Vec::new(),
            static_files: Vec::new(),
            assets: Vec::new(),
            working_index: Index::default(),
        })
    }

    #[tracing::instrument(skip(self))]
    pub fn build(&mut self) -> Result<()> {
        self.discover()?;
        let mut index_pages = self.render()?;
        index_pages.sort_by(|a, b| b.date.cmp(&a.date));

        self.working_index
            .build_index(&self.ctx.config.output_path)?;
        self.update_db()?;

        let index = self.load_index()?;
        index_pages
            .iter()
            .map(|p| write_index_to_disk(&self.ctx.tera, p, &index, &index_pages))
            .collect::<Result<Vec<()>>>()?;

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
        let entries = discover_entries(&self.ctx.conn, &self.ctx.config.root)?;

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
    pub fn render(&mut self) -> Result<Vec<Page>> {
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
                )
            })
            .collect::<Result<HashSet<Page>>>()?;

        let mut index_pages = Vec::new();
        let mut posts = Vec::new();

        for page in pages.into_iter().filter(|p| {
            p.frontmatter.as_ref().is_some_and(|f| !f.draft) || self.ctx.config.development
        }) {
            if page.index {
                index_pages.push(page);
            } else {
                posts.push(page);
            }
        }

        let written_posts = posts
            .into_iter()
            .map(|p| write_page_to_disk(&self.ctx.tera, p))
            .collect::<Result<HashSet<Page>>>()?;

        self.working_index = Index::from(written_posts);

        // let _ = self
        //     .stylesheets
        //     .iter_mut()
        //     .map(|s| s.render(&self.ctx.config.output_path))
        //     .collect::<Result<Vec<()>>>()?;

        let _ = self
            .assets
            .iter_mut()
            .map(|s| s.render(&self.ctx.config.output_path))
            .collect::<Result<Vec<()>>>()?;

        let _ = self
            .static_files
            .iter_mut()
            .map(|a| a.render(&self.ctx.config.output_path))
            .collect::<Result<Vec<()>>>()?;

        Ok(index_pages)
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
}
