use color_eyre::eyre::Result;
use rusqlite::Connection;

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
