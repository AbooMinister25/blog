use markdown::MarkdownRenderer;
use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use tera::Tera;

use crate::{config::Config, page::Page, sql::PostInIndex};

/// Shared context for the site.
#[derive(Debug)]
pub struct Context<'c> {
    pub conn: PooledConnection<SqliteConnectionManager>,
    pub tera: Tera,
    pub markdown_renderer: MarkdownRenderer<'c>,
    pub config: Config,
    pub posts: Option<Vec<PostInIndex>>,
    pub special_pages: Option<Vec<Page>>,
}

impl<'c> Context<'c> {
    pub fn new(
        conn: PooledConnection<SqliteConnectionManager>,
        tera: Tera,
        markdown_renderer: MarkdownRenderer<'c>,
        config: Config,
    ) -> Self {
        Self {
            conn,
            tera,
            markdown_renderer,
            config,
            posts: None,
            special_pages: None,
        }
    }
}
