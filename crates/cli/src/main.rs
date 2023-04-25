#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::module_name_repetitions)]

use clap::Parser;
use color_eyre::Result;
use entry::filesystem::ensure_removed;
use site::Site;
use sql::setup_sql;
use std::path::Path;
use std::time::Instant;
use tera::Tera;
use tracing::{info, metadata::LevelFilter, subscriber};
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;

#[derive(Parser)]
struct Args {
    /// Reload on file changes
    #[clap(long, action)]
    watch: bool,

    // Whether to run a clean build
    #[clap(long, action)]
    clean: bool,
}

#[tracing::instrument]
fn main() -> Result<()> {
    let now = Instant::now();

    // Install panic and error report handlers
    color_eyre::install()?;

    // Set up tracing subscribers
    let file_appender = tracing_appender::rolling::hourly("log/", "ssg.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let subscriber = tracing_subscriber::registry()
        .with(
            fmt::Layer::default()
                .with_writer(non_blocking)
                .with_ansi(false)
                .with_filter(LevelFilter::TRACE),
        )
        .with(fmt::layer().compact().with_filter(LevelFilter::INFO));

    subscriber::set_global_default(subscriber)?;
    info!("Set up subscribers");

    // Parse command line arguments
    let args = Args::parse();

    // Clean build
    if args.clean {
        info!("Clean build, making sure existing database removed");
        ensure_removed(Path::new("blog.db"))?;
    }

    let conn = setup_sql()?;
    info!("Connected to database, created tables");

    let tera = Tera::new("templates/**/*.tera")?;
    info!("Loaded templates");

    let site = Site::new(Path::new(".").to_owned(), tera, conn);
    site.build()?;
    info!("Built site");

    let elapsed = now.elapsed();
    info!("Built in {:.2?} seconds", elapsed);

    Ok(())
}
