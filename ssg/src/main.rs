#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_panics_doc)]

pub mod markdown;

use color_eyre::eyre::Result;
use rusqlite::Connection;

fn setup() -> Result<()> {
    let conn = Connection::open("blog.db")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS posts (
            id INTEGER PRIMARY KEY,
            title VARCHAR NOT NULL,
            content TEXT NOT NULL,
            summary TEXT NOT NULL,
            tags TEXT NOT NULL,
            published BOOLEAN NOT NULL DEFAULT 'f',
            published_at TIMESTAMP NOT NULL
        )",
        (),
    )?;

    Ok(())
}

fn main() -> Result<()> {
    setup()?;
    Ok(())
}
