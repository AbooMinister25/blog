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
use sql::{insert_post, update_post, PostSQL};
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
        let ctx = Context::new(conn, renderer, tera, config);

        Ok(Self {
            ctx,
            discovered_posts: Vec::new(),
            stylesheets: Vec::new(),
            static_assets: Vec::new(),
            working_index: Index::default(),
        })
    }

    #[tracing::instrument(skip(self))]
    pub fn build(&mut self) -> Result<()> {
        self.discover()?;
        self.render()?;

        self.working_index
            .build_index(&self.ctx.config.output_path)?;
        self.update_db()?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn load_index(&mut self) -> Result<()> {
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    fn discover(&mut self) -> Result<()> {
        info!("Discovering entries");
        let entries = discover_entries(&self.ctx.conn, &self.ctx.config.root)?;

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
                    &self.ctx.config.url,
                    &p.path,
                    &self.ctx.config.output_path,
                    String::from_utf8_lossy(&p.raw_content).to_string(),
                    &p.hash,
                    p.new,
                )
            })
            .collect::<Result<HashSet<Page>>>()?;

        self.working_index = Index::from(pages);

        let _ = self
            .stylesheets
            .iter_mut()
            .map(|s| s.render(&self.ctx.config.output_path))
            .collect::<Result<Vec<()>>>()?;

        let _ = self
            .static_assets
            .iter_mut()
            .map(|a| a.render(&self.ctx.config.output_path))
            .collect::<Result<Vec<()>>>()?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    fn update_db(&mut self) -> Result<()> {
        for page in self.working_index.pages.iter().filter(|p| !p.index) {
            let post_sql = PostSQL::new(
                &page.path,
                &page.permalink,
                &page.frontmatter.title,
                &page.frontmatter.tags,
                &page.date,
                &page.summary,
                &page.hash,
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
