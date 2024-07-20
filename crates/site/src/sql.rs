use std::path::Path;
use std::{fmt::Debug, path::PathBuf};

use chrono::{DateTime, Utc};
use color_eyre::{eyre::ContextCompat, Result};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;

/// Sets up the SQLite database, creating the initial tables if they don't exist, and acquiring the connection.
#[tracing::instrument]
pub fn setup_sql() -> Result<Pool<SqliteConnectionManager>> {
    // Establish connection to the database
    let manager = SqliteConnectionManager::file("blog.db");
    let pool = Pool::new(manager)?;
    let conn = pool.get()?;

    conn.execute(
        "
        CREATE TABLE IF NOT EXISTS entries (
            entry_id INTEGER PRIMARY KEY,
            path VARCHAR NOT NULL,
            hash TEXT NOT NULL
        )
    ",
        (),
    )?;

    Ok(pool)
}

/// Fetch hashes for a given path from the database
#[tracing::instrument]
pub fn get_hashes<P: AsRef<Path> + Debug>(conn: &Connection, path: P) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT hash FROM entries WHERE path = :path")?;
    let path_str = path
        .as_ref()
        .to_str()
        .context("Error while converting path to string")?;

    // Get hashes found for the given path
    let hashes_iter = stmt.query_map(&[(":path", path_str)], |row| row.get(0))?;
    let mut hashes: Vec<String> = Vec::new();

    for hash in hashes_iter {
        hashes.push(hash?);
    }

    Ok(hashes)
}

/// Insert an entry into the database
#[tracing::instrument]
pub fn insert_entry<P: AsRef<Path> + Debug>(conn: &Connection, path: P, hash: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO entries (path, hash) VALUES (?1, ?2)",
        (
            &path
                .as_ref()
                .to_str()
                .context("Error while converting path to string")?,
            &hash,
        ),
    )?;

    Ok(())
}

/// Update an existing entry in the database with a new hash
#[tracing::instrument]
pub fn update_entry_hash<P: AsRef<Path> + Debug>(
    conn: &Connection,
    path: P,
    new_hash: &str,
) -> Result<()> {
    let mut stmt = conn.prepare("UPDATE entries SET hash = (:hash) WHERE path = (:path)")?;
    stmt.execute(&[
        (":hash", &new_hash),
        (
            ":path",
            &path
                .as_ref()
                .to_str()
                .context("Path should be valid unicode")?,
        ),
    ])?;

    Ok(())
}