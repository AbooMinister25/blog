#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::module_name_repetitions)]

mod build;
mod entry;
mod markdown;
mod post;
mod sql;
mod stylesheet;

use crate::sql::setup_sql;
use clap::Parser;
use color_eyre::eyre::Result;
use tera::Tera;
use tracing::{info, subscriber, Level};
use tracing_subscriber::FmtSubscriber;

pub const DATE_FORMAT: &str = "%b %e, %Y";

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

#[tracing::instrument]
fn main() -> Result<()> {
    // Install panic and error report handlers
    color_eyre::install()?;
    // Set up tracing subscribers
    let fmt_subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE) // TODO: Make this DEBUG and use another subscriber for verbose traces.
        .finish();
    subscriber::set_global_default(fmt_subscriber)?;
    info!("Set up subscribers");

    let conn = setup_sql()?;
    info!("Connected to database, created tables");

    let tera = Tera::new("templates/**/*.tera")?;
    info!("Loaded templates");

    // Parse command line arguments
    let args = Args::parse();

    Ok(())
}
