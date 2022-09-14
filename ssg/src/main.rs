#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_panics_doc)]

mod markdown;
mod build;

use color_eyre::eyre::Result;
use rusqlite::Connection;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

fn setup() -> Result<()> {
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
            hash TEXT NOT NULL,
            summary TEXT NOT NULL,
            tags TEXT NOT NULL,
            published BOOLEAN NOT NULL DEFAULT 'f',
            published_at TIMESTAMP NOT NULL
        )",
        (),
    )?;
    info!("Established connection to database");

    Ok(())
}

fn main() -> Result<()> {
    setup()?;
    Ok(())
}
