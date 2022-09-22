#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_panics_doc)]

mod build;
mod markdown;
mod stylesheets;

use build::build;
use color_eyre::eyre::Result;
use rusqlite::Connection;
use tera::Tera;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

fn setup() -> Result<Connection> {
    // Setting up logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    info!("Set up logging");

    // Establishing database connection
    let conn = Connection::open("blog.db")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS posts (
            id INTEGER PRIMARY KEY,
            title VARCHAR NOT NULL,
            path VARCHAR NOT NULL,
            hash TEXT NOT NULL,
            tags TEXT NOT NULL
        )",
        (),
    )?;
    info!("Established connection to database");

    Ok(conn)
}

fn main() -> Result<()> {
    let conn = setup()?;
    let tera = Tera::new("templates/**/*.html")?;

    build(conn, &tera)?;

    Ok(())
}
