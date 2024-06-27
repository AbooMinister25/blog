#![warn(clippy::pedantic, clippy::nursery)]

use std::time::Instant;

use clap::Parser;
use color_eyre::Result;
use config::Config;
use figment::{
    providers::{Format, Serialized, Toml},
    Figment,
};
use serde::Serialize;
use site::Site;
use sql::setup_sql;
use tracing::{info, metadata::LevelFilter, subscriber};
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;
use utils::fs::ensure_removed;

#[derive(Parser, Serialize)]
struct Args {
    /// Reload on file changes
    #[clap(long, action)]
    watch: bool,

    /// Whether to run a clean build
    #[clap(long, action)]
    clean: bool,

    /// Whether or not to run a development build. In development builds, drafts are rendered.
    #[clap(long, action)]
    dev: bool,
}

#[tracing::instrument]
fn main() -> Result<()> {
    let now = Instant::now();

    // Install panic and error report handlers
    color_eyre::install()?;

    let args = Args::parse();

    let config: Config = Figment::from(Serialized::defaults(Config::default()))
        .merge(Toml::file("Config.toml"))
        .join(("development", args.dev))
        .extract()?;

    let file_appender = tracing_appender::rolling::hourly(&config.log_folder, "ssg.log");
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

    // Clean build
    if args.clean {
        info!("Clean build, making sure existing database removed.");
        ensure_removed("blog.db")?;
        ensure_removed(&config.output_path)?;
    }

    let conn = setup_sql()?;
    info!("Connected to database, created tables");

    let mut site = Site::new(conn, config)?;
    site.build()?;

    info!("Built site");

    let elapsed = now.elapsed();
    info!("Built in {:.2?} seconds", elapsed);

    Ok(())
}
