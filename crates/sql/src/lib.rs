#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::module_name_repetitions)]

use chrono::{DateTime, NaiveDateTime, Utc};
use color_eyre::eyre::{eyre, ContextCompat, Result};
use entry::summary::get_summary;
use rusqlite::{named_params, Connection};
use serde::Serialize;
use std::path::Path;

const DATE_FORMAT: &str = "%b %e, %Y";

// The table to execute an operation for
#[derive(Debug)]
pub enum For {
    STATIC,
    CONTENT,
}

// Represents a blog post
#[derive(Serialize)]
pub struct Post {
    pub title: String,
    pub content: String,
    pub summary: String,
    pub timestamp: String,
    pub tags: Vec<String>,
}

/// Sets up SQLite database, creating initial tables if they don't exist, and acquiring the connection.
#[tracing::instrument]
pub fn setup_sql() -> Result<Connection> {
    // Establish connection to database
    let conn = Connection::open("blog.db")?;

    conn.execute("PRAGMA foreign_keys = 1", ())?;
    conn.execute(
        "
        CREATE TABLE IF NOT EXISTS content (
            content_id INTEGER PRIMARY KEY,
            title VARCHAR NOT NULL,
            path VARCHAR NOT NULL,
            kind TEXT NOT NULL,
            hash TEXT NOT NULL,
            rendered_content TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            series_id INTEGER,
            FOREIGN KEY (series_id) REFERENCES series(series_id)
        )
    ",
        (),
    )?;
    conn.execute(
        "
        CREATE TABLE IF NOT EXISTS series (
            series_id INTEGER PRIMARY KEY,
            name VARCHAR NOT NULL
        )
    ",
        (),
    )?;
    conn.execute(
        "
        CREATE TABLE IF NOT EXISTS static_assets (
            asset_id INTEGER PRIMARY KEY,
            path VARCHAR NOT NULL,
            hash TEXT NOT NULL
        )
    ",
        (),
    )?;
    conn.execute(
        "
        CREATE TABLE IF NOT EXISTS tags (
            tag_id INTEGER PRIMARY KEY,
            name VARCHAR NOT NULL
        )
    ",
        (),
    )?;
    conn.execute(
        "
        CREATE TABLE IF NOT EXISTS tags_content (
            content_id INTEGER NOT NULL,
            tag_id INTEGER NOT NULL,
            PRIMARY KEY (content_id, tag_id),
            FOREIGN KEY (content_id) REFERENCES content(content_id),
            FOREIGN KEY (tag_id) REFERENCES tags(tag_id)
        )
    ",
        (),
    )?;
    Ok(conn)
}

// Given an array of tags, insert any of those tags that aren't already in the database, into the database.
#[tracing::instrument]
pub fn insert_tags(conn: &Connection, tags: &[String]) -> Result<()> {
    let mut stmt = conn.prepare("SELECT name FROM tags WHERE name = ?")?;
    for tag in tags {
        // If the tag doesn't already exist, create it
        if !stmt.exists([&tag])? {
            conn.execute("INSERT INTO tags (name) VALUES (?)", [&tag])?;
        }
    }

    Ok(())
}

// Insert an entry into the database
#[tracing::instrument]
pub fn insert_content(
    conn: &Connection,
    title: &str,
    path: &Path,
    kind: &str,
    hash: &str,
    rendered_content: &str,
    date: DateTime<Utc>,
) -> Result<()> {
    conn.execute(
        "INSERT INTO content
        (title, path, kind, hash, rendered_content, timestamp)
        VALUES (?1, ?2, ?3, ?4, ?5, datetime(?6))
        ",
        (
            &title,
            &path
                .to_str()
                .context("Error while converting path to string")?,
            &kind,
            &hash,
            &rendered_content,
            &date,
        ),
    )?;

    Ok(())
}

// Update existing content in the database
#[tracing::instrument]
pub fn update_content(
    conn: &Connection,
    title: &str,
    content: &str,
    date: DateTime<Utc>,
    path: &Path,
) -> Result<()> {
    conn.execute(
        "
    UPDATE content
    SET title = (:title),
        rendered_content = (:content),
        timestamp = datetime(:timestamp)
    WHERE path = (:path)
    ",
        named_params! {
            ":title": &title,
            ":content": &content,
            ":timestamp": &date,
            ":path": &path.to_str().context("Error while converting path to string")?,
        },
    )?;

    Ok(())
}

// Update hash for a content entry in the database
#[tracing::instrument]
pub fn update_hash(conn: &Connection, new_hash: &str, path: &Path, for_: For) -> Result<()> {
    let mut stmt = match for_ {
        For::CONTENT => conn.prepare("UPDATE content SET hash = (:hash) WHERE path = (:path)"),
        For::STATIC => conn.prepare("UPDATE static_assets SET hash = (:hash) WHERE path = (:path)"),
    }?;

    stmt.execute(&[
        (":hash", &new_hash),
        (
            ":path",
            &path.to_str().context("Path should be valid unicode")?,
        ),
    ])?;

    Ok(())
}

// Fetch the hash from the database
#[tracing::instrument]
pub fn get_hash(conn: &Connection, path: &Path, for_: For) -> Result<Vec<String>> {
    let mut stmt = match for_ {
        For::CONTENT => conn.prepare("SELECT hash FROM content WHERE path = :path"),
        For::STATIC => conn.prepare("SELECT hash FROM static_assets WHERE path = :path"),
    }?;
    let path_str = path
        .to_str()
        .context("Error while converting path to string")?;

    // Get the hashes found for the path
    let hashes_iter = stmt.query_map(&[(":path", path_str)], |row| row.get(0))?;
    let mut hashes: Vec<String> = Vec::new();
    for hash in hashes_iter {
        hashes.push(hash?);
    }

    Ok(hashes)
}

#[tracing::instrument]
pub fn insert_series(conn: &Connection, name: &str) -> Result<()> {
    conn.execute("INSERT INTO series (name) VALUES (?1)", [&name])?;

    Ok(())
}

// Insert a static asset into the database
#[tracing::instrument]
pub fn insert_asset(conn: &Connection, path: &Path, hash: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO static_assets
                (path, hash)
                VALUES (?1, ?2)
                ",
        (
            &path
                .to_str()
                .context("Error while converting path to string")?,
            &hash,
        ),
    )?;

    Ok(())
}

// Insert tag maps for blog content
#[tracing::instrument]
pub fn insert_tagmaps(conn: &Connection, path: &Path, tags: &[String]) -> Result<()> {
    let path_str = path
        .to_str()
        .context("Error while converting path to string")?;

    // A statement to check whether a tag-entity map already exists
    let mut exists_stmt = conn.prepare(
        "SELECT content_id, tag_id from tags_content WHERE (content_id = (?1) AND tag_id = (?2))",
    )?;
    // A statement to select tag id's from the database
    let mut tags_stmt = conn.prepare("SELECT tag_id FROM tags WHERE name = ?")?;
    // A statement to select id's for the entity from the database
    let mut entity_stmt = conn.prepare("SELECT content_id FROM content WHERE path = ?")?;

    let mut rows = entity_stmt.query([path_str])?;
    let id: i32 = if let Some(row) = rows.next()? {
        Ok(row.get(0)?)
    } else {
        Err(eyre!("Error while querying content from database"))
    }?;

    for tag in tags {
        let mut rows = tags_stmt.query([&tag])?;
        if let Some(row) = rows.next()? {
            let tag_id: i32 = row.get(0)?;

            if !exists_stmt.exists((id, tag_id))? {
                conn.execute(
                    "INSERT INTO tags_content (content_id, tag_id) VALUES (?1, ?2)",
                    (id, tag_id),
                )?;
            }
        }
    }

    Ok(())
}

/// Fetch last ten posts from the database
///
/// this is like, super fucky, so I'm just going to refactor it later
///
/// # Errors
/// When an error is encountered when querying the database
pub fn get_posts(conn: &Connection) -> Result<Vec<Post>> {
    let mut content_stmt = conn.prepare("SELECT content_id, title, rendered_content, timestamp FROM content WHERE content.kind = 'post' ORDER BY content_id LIMIT 10")?;
    let mut tags_stmt = conn.prepare(
        "
        SELECT tags.name 
        FROM content
        INNER JOIN 
            tags_content ON tags_content.content_id = content.content_id 
        INNER JOIN 
            tags ON tags_content.tag_id = tags.tag_id 
        WHERE content.content_id = (?)",
    )?;

    // Load into an iterator of `Post`s
    let posts_iter = content_stmt.query_map([], |row| {
        let id: i32 = row.get(0)?;
        let summary_str: String = row.get(2)?;
        let date: NaiveDateTime = row.get(3)?;

        // Fetch tags for post
        let tags_iter = tags_stmt
            .query_map([id], |tags_row| {
                let tag_name = tags_row.get(0)?;
                Ok(tag_name)
            })
            .expect("Error while fetching tags from database");

        let mut tags = vec![];
        for tag in tags_iter {
            tags.push(tag.expect("Error while collecting tags"));
        }

        Ok(Post {
            title: row.get(1)?,
            content: row.get(2)?,
            summary: get_summary(&summary_str).expect("Error while rewriting HTML"),
            timestamp: date.format(DATE_FORMAT).to_string(),
            tags,
        })
    })?;

    // Collect iterator into a vec
    let mut posts = vec![];
    for post in posts_iter {
        posts.push(post?);
    }

    Ok(posts)
}
