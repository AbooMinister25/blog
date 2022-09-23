use crate::post::build_posts;
use crate::stylesheets::compile_stylesheets;
use color_eyre::eyre::Result;
use rusqlite::Connection;
use tera::Tera;
use tracing::info;

#[tracing::instrument(skip(tera, conn))]
pub fn build(conn: Connection, tera: &Tera) -> Result<()> {
    info!("Compiling stylesheets");
    compile_stylesheets(&conn)?;
    info!("Building posts");
    build_posts(&conn, tera)?;

    Ok(())
}
