use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use tera::Tera;

use crate::config::Config;

/// Shared context for the site.
#[derive(Debug)]
pub struct Context {
    pub conn: PooledConnection<SqliteConnectionManager>,
    pub tera: Tera,
    pub config: Config,
}

impl Context {
    pub fn new(
        conn: PooledConnection<SqliteConnectionManager>,
        tera: Tera,
        config: Config,
    ) -> Self {
        Self { conn, tera, config }
    }
}
