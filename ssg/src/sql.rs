use std::path::Path;

use chrono::{DateTime, Utc};
use color_eyre::eyre::{eyre, ContextCompat, Result};
use rusqlite::{named_params, Connection};

// Represents what entity the function `insert_tagmaps` should insert the tag-entity maps for.
#[derive(Debug)]
pub enum MapFor {
    Post,
    Series,
}

/// Sets up SQLite database, creating initial tables if they don't exist, and acquiring the connection.
#[tracing::instrument]
pub fn setup_sql() -> Result<Connection> {
    // Establish connection to database
    let conn = Connection::open("blog.db")?;

    conn.execute("PRAGMA foreign_keys = 1", ())?;
    conn.execute(
        "
        CREATE TABLE IF NOT EXISTS posts (
            post_id INTEGER PRIMARY KEY,
            title VARCHAR NOT NULL,
            path VARCHAR NOT NULL,
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
            name VARCHAR NOT NULL,
            path VARCHAR NOT NULL,
            hash TEXT NOT NULL,
            description VARCHAR NOT NULL,
            timestamp TEXT NOT NULL
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
        CREATE TABLE IF NOT EXISTS assets (
            asset_id INTEGER PRIMARY KEY,
            path VARCHAR NOT NULL,
            hash TEXT NOT NULL
        )
    ",
        (),
    )?;
    conn.execute(
        "
        CREATE TABLE IF NOT EXISTS tags_posts (
            post_id INTEGER NOT NULL,
            tag_id INTEGER NOT NULL,
            PRIMARY KEY (post_id, tag_id),
            FOREIGN KEY (post_id) REFERENCES posts(post_id),
            FOREIGN KEY (tag_id) REFERENCES tags(tag_id)
        )
    ",
        (),
    )?;
    conn.execute(
        "
        CREATE TABLE IF NOT EXISTS tags_series (
            series_id INTEGER NOT NULL,
            tag_id INTEGER NOT NULL,
            PRIMARY KEY (series_id, tag_id),
            FOREIGN KEY (series_id) REFERENCES series(series_id),
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

// Insert a post into the database
#[tracing::instrument]
pub fn insert_post(
    conn: &Connection,
    title: &str,
    path: &Path,
    hash: &str,
    content: &str,
    date: DateTime<Utc>,
) -> Result<()> {
    conn.execute(
        "INSERT INTO posts
        (title, path, hash, rendered_content, timestamp)
        VALUES (?1, ?2, ?3, ?4, datetime(?5))
        ",
        (
            &title,
            &path
                .to_str()
                .context("Error while converting path to string")?,
            &hash,
            &content,
            &date,
        ),
    )?;

    Ok(())
}

// Update an existing post in the database
#[tracing::instrument]
pub fn update_post(
    conn: &Connection,
    title: &str,
    content: &str,
    date: DateTime<Utc>,
    path: &Path,
) -> Result<()> {
    conn.execute(
        "
    UPDATE posts
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

// Insert into the tags_posts or tags_series table
#[tracing::instrument]
pub fn insert_tagmaps(
    conn: &Connection,
    path: &Path,
    tags: &[String],
    entity: MapFor,
) -> Result<()> {
    let path_str = path
        .to_str()
        .context("Error while converting path to string")?;

    // A statement to check whether a tag-entity map already exists
    let mut exists_stmt = match entity {
        MapFor::Post => conn.prepare(
            "SELECT post_id, tag_id from tags_posts WHERE (post_id = (?1) AND tag_id = (?2))",
        )?,
        MapFor::Series => conn.prepare(
            "SELECT series_id, tag_id from tags_series WHERE (series_id = (?1) AND tag_id = (?2))",
        )?,
    };
    // A statement to select tag id's from the database
    let mut tags_stmt = conn.prepare("SELECT tag_id FROM tags WHERE name = ?")?;
    // A statement to select id's for the entity from the database
    let mut entity_stmt = match entity {
        MapFor::Post => conn.prepare("SELECT post_id FROM posts WHERE path = ?")?,
        MapFor::Series => conn.prepare("SELECT series_id FROM series WHERE path = ?")?,
    };

    let mut rows = entity_stmt.query([path_str])?;
    let id: i32 = if let Some(row) = rows.next()? {
        Ok(row.get(0)?)
    } else {
        Err(eyre!("Error while querying post"))
    }?;

    for tag in tags {
        let mut rows = tags_stmt.query([&tag])?;
        if let Some(row) = rows.next()? {
            let tag_id: i32 = row.get(0)?;

            if !exists_stmt.exists((id, tag_id))? {
                match entity {
                    MapFor::Post => conn.execute(
                        "INSERT INTO tags_posts (post_id, tag_id) VALUES (?1, ?2)",
                        (id, tag_id),
                    )?,
                    MapFor::Series => conn.execute(
                        "INSERT INTO tags_series (series_id, tag_id) VALUES (?1, ?2)",
                        (id, tag_id),
                    )?,
                };
            }
        }
    }

    Ok(())
}
