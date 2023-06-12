use std::{fs, path::Path};

use color_eyre::Result;
use rusqlite::Connection;
use sql::get_posts;
use tera::{Context, Tera};
use tracing::debug;

/// Builds the index for a series
///
/// This is most likely a less than ideal solution, so it stands
/// to be refactored later on.
#[tracing::instrument]
pub fn build_series_index(conn: &Connection, tera: &Tera) -> Result<()> {
    debug!("Building series index");
    // Create the file
    let to_path = Path::new("public/series/index.html");
    let file = fs::File::create(to_path)?;

    // Insert context for the template
    let mut context = Context::new();

    let posts = get_posts(conn, 500, "series")?;
    context.insert("posts", &posts);

    // Render the template
    tera.render_to("series_index.html.tera", &context, file)?;
    debug!("Built series index");

    Ok(())
}
