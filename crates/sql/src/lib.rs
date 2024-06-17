#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::must_use_candidate, clippy::missing_const_for_fn)]

use std::path::Path;
use std::{fmt::Debug, path::PathBuf};

use chrono::{DateTime, Utc};
use color_eyre::{eyre::ContextCompat, Result};
use rusqlite::Connection;

#[derive(Debug)]
pub struct PostSQL<'a> {
    path: &'a Path,
    permalink: &'a str,
    title: &'a str,
    tags: &'a Vec<String>,
    date: &'a DateTime<Utc>,
    summary: &'a str,
    hash: &'a str,
    new: bool,
}

impl<'a> PostSQL<'a> {
    pub fn new(
        path: &'a Path,
        permalink: &'a str,
        title: &'a str,
        tags: &'a Vec<String>,
        date: &'a DateTime<Utc>,
        summary: &'a str,
        hash: &'a str,
        new: bool,
    ) -> Self {
        Self {
            path,
            permalink,
            title,
            tags,
            date,
            summary,
            hash,
            new,
        }
    }
}

#[derive(Debug)]
pub struct RetPostSQL {
    pub path: PathBuf,
    pub permalink: String,
    pub title: String,
    pub tags: Vec<String>,
    pub date: DateTime<Utc>,
    pub summary: String,
    pub hash: String,
    pub new: bool,
}

impl RetPostSQL {
    pub fn new(
        path: PathBuf,
        permalink: String,
        title: String,
        tags: Vec<String>,
        date: DateTime<Utc>,
        summary: String,
        hash: String,
        new: bool,
    ) -> Self {
        Self {
            path,
            permalink,
            title,
            tags,
            date,
            summary,
            hash,
            new,
        }
    }
}

/// Sets up the SQLite database, creating the initial tables if they don't exist, and acquiring the connection.
#[tracing::instrument]
pub fn setup_sql() -> Result<Connection> {
    // Establish connection to the database
    let conn = Connection::open("blog.db")?;

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

    conn.execute(
        "
        CREATE TABLE IF NOT EXISTS posts (
            post_id INTEGER PRIMARY KEY,
            path VARCHAR NOT NULL,
            permalink TEXT NOT NULL,
            title TEXT NOT NULL,
            tags JSON NOT NULL,
            date TEXT NOT NULL,
            summary TEXT NOT NULL,
            hash TEXT NOT NULL,
            new INTEGER NOT NULL
        )
        ",
        (),
    )?;

    Ok(conn)
}

/// Fetch hash from database
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

/// Insert a post into the database
#[tracing::instrument]
pub fn insert_post(conn: &Connection, post: PostSQL) -> Result<()> {
    conn.execute(
        "
    INSERT INTO posts (path, permalink, title, tags, date, summary, hash, new)
    VALUES (?1, ?2, ?3, json(?4), datetime(?5), ?6, ?7, ?8)
    ",
        (
            &post.path.to_str().context("Path should be valid unicode")?,
            &post.permalink,
            &post.title,
            &serde_json::to_string(&post.tags)?,
            &post.date,
            &post.summary,
            &post.hash,
            &post.new,
        ),
    )?;

    Ok(())
}

/// Update an existing post in the database
#[tracing::instrument]
pub fn update_post(conn: &Connection, post: PostSQL) -> Result<()> {
    conn.execute(
        "
    UPDATE posts 
    SET permalink = ?1,
        title = ?2,
        tags = json(?3),
        date = datetime(?4),
        summary = ?5,
        hash = ?6,
        new = ?7
    WHERE path = (?8)
    ",
        (
            &post.permalink,
            &post.title,
            &serde_json::to_string(&post.tags)?,
            &post.date,
            &post.summary,
            &post.hash,
            &post.path.to_str().context("Path should be valid unicode")?,
            &post.new,
        ),
    )?;

    Ok(())
}

/// Fetch posts from database
#[tracing::instrument]
pub fn get_posts(conn: &Connection) -> Result<Vec<RetPostSQL>> {
    let mut stmt = conn.prepare(
        "
    SELECT path, permalink, title, tags, date, summary, hash, new
    FROM posts
    ",
    )?;

    let posts_iter = stmt.query_map([], |row| {
        let path: String = row.get(0)?;
        let tags: String = row.get(3)?;
        let parsed_tags: Vec<String> = serde_json::from_str(&tags).expect("JSON should be valid.");

        Ok(RetPostSQL::new(
            PathBuf::from(&path),
            row.get(1)?,
            row.get(2)?,
            parsed_tags,
            row.get(4)?,
            row.get(5)?,
            row.get(6)?,
            row.get(7)?,
        ))
    })?;

    let mut posts = Vec::new();
    for post in posts_iter {
        posts.push(post?);
    }

    Ok(posts)
}
