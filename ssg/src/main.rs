#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::module_name_repetitions)]

mod assets;
mod build;
mod markdown;
mod post;
mod search;
mod stylesheets;

use build::build;
use clap::Parser;
use color_eyre::eyre::Result;
use rusqlite::Connection;
use std::time::Instant;
use tera::Tera;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

const DATE_FORMAT: &str = "%b %e, %Y";

#[derive(Parser)]
#[clap(version, about)]
struct Args {
    // The directory to output HTML files into
    #[clap(short = 'O', long, default_value_t = String::from("public"))]
    output_dir: String,
    // The directory to output generated CSS into
    #[clap(short = 'C', long, default_value_t = String::from("styles"))]
    css_output_dir: String,
    /// The directory to look for markdown files in
    #[clap(short = 'I', long, default_value_t = String::from("contents"))]
    html_input_dir: String,
    /// The directory to look for SCSS files in
    #[clap(short = 'S', long, default_value_t = String::from("sass"))]
    scss_input_dir: String,
    /// Whether to reload on file changes
    #[clap(long, action)]
    watch: bool,
}

fn setup() -> Result<Connection> {
    // Setting up logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    info!("Set up logging");

    // Establishing database connection
    let conn = Connection::open("blog.db")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS posts (
            id INTEGER PRIMARY KEY,
            title VARCHAR NOT NULL,
            path VARCHAR NOT NULL,
            hash TEXT NOT NULL,
            rendered_content TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            tags TEXT NOT NULL
        )",
        (),
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS styles (
            id INTEGER PRIMARY KEY,
            path VARCHAR NOT NULL,
            hash TEXT NOT NULL
        )",
        (),
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS assets (
            id INTEGER PRIMARY KEY,
            path VARCHAR NOT NULL,
            hash TEXT NOT NULL
        )",
        (),
    )?;
    info!("Established connection to database");

    Ok(conn)
}

fn main() -> Result<()> {
    let now = Instant::now();

    let conn = setup()?;
    let tera = Tera::new("templates/**/*.html")?;

    let args = Args::parse();

    build(
        conn,
        &tera,
        args.output_dir,
        args.css_output_dir,
        args.html_input_dir,
        args.scss_input_dir,
    )?;

    let elapsed = now.elapsed();
    info!("Built in {:.2?} seconds", elapsed);

    Ok(())
}
