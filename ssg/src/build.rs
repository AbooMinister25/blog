use crate::post::build_posts;
use crate::stylesheets::compile_stylesheets;
use color_eyre::eyre::Result;
use rusqlite::Connection;
use tera::Tera;
use tracing::info;

#[tracing::instrument(skip(
    tera,
    conn,
    output_dir,
    css_output_dir,
    html_input_dir,
    scss_input_dir
))]
pub fn build(
    conn: Connection,
    tera: &Tera,
    output_dir: String,
    css_output_dir: String,
    html_input_dir: String,
    scss_input_dir: String,
) -> Result<()> {
    info!("Compiling stylesheets");
    compile_stylesheets(&conn, &css_output_dir, &scss_input_dir)?;
    info!("Building posts");
    build_posts(&conn, tera, &output_dir, &html_input_dir)?;

    Ok(())
}
