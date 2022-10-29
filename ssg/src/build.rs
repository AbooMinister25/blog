use crate::assets::process_assets;
use crate::post::build_posts;
use crate::stylesheets::compile_stylesheets;
use color_eyre::eyre::Result;
use rusqlite::Connection;
use std::fs;
use std::path::Path;
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
    info!("Creating directories");
    create_directories(&output_dir, &css_output_dir)?;
    info!("Compiling stylesheets");
    compile_stylesheets(&conn, &css_output_dir, &scss_input_dir)?;
    info!("Minimizing assets");
    process_assets(&conn)?;
    info!("Building posts");
    build_posts(&conn, tera, &output_dir, &html_input_dir)?;

    Ok(())
}

fn create_directories(output_dir: &str, css_output_dir: &str) -> Result<()> {
    if !Path::new(output_dir).exists() {
        info!("Creating {output_dir}");
        fs::create_dir(output_dir)?;
    }
    if !Path::new(css_output_dir).exists() {
        info!("Creating {css_output_dir}");
        fs::create_dir(css_output_dir)?;
    }

    Ok(())
}
