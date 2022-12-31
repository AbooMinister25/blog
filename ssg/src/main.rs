#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::module_name_repetitions)]

mod build;
mod entry;
mod markdown;
mod post;
mod sql;
mod stylesheet;

use crate::build::build;
use crate::sql::setup_sql;
use clap::Parser;
use color_eyre::eyre::Result;
use std::time::Instant;
use tera::Tera;
use tracing::{info, subscriber, Level};
use tracing_subscriber::FmtSubscriber;

pub const DATE_FORMAT: &str = "%b %e, %Y";

#[derive(Parser)]
#[clap(version, about)]
struct Args {
    /// Whether to reload on file changes
    #[clap(long, action)]
    watch: bool,
    /// Whether to run a clean build
    #[clap(long, action)]
    clean: bool,
}

#[tracing::instrument]
fn main() -> Result<()> {
    let now = Instant::now();

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

    build(&conn, &tera)?;
    info!("Built site");

    let elapsed = now.elapsed();
    info!("Built in {:.2?} seconds", elapsed);

    Ok(())
}
