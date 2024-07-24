use std::{fs, path::Path, time::Instant};

use clap::Parser;
use color_eyre::Result;
use figment::{
    providers::{Format, Serialized, Toml},
    Figment,
};
use site::{config::Config, sql::setup_sql, utils::fs::ensure_removed, Site};
use tempfile::Builder;
use tracing::{info, level_filters::LevelFilter, subscriber};
use tracing_subscriber::{fmt, layer::SubscriberExt, Layer};

#[derive(Parser)]
struct Args {
    /// Whether to run a clean build
    #[clap(long, action)]
    clean: bool,

    /// Whether or not to run a development build. In development builds, drafts are rendered.
    #[clap(long, action)]
    dev: bool,
}

fn main() -> Result<()> {
    // Install panic and error report handlers.
    color_eyre::install()?;

    let args = Args::parse();

    let tmp_dir = Builder::new()
        .prefix("temp")
        .rand_bytes(0)
        .tempdir_in(".")?;

    let mut config: Config = Figment::from(Serialized::defaults(Config::default()))
        .merge(Toml::file("Config.toml"))
        .join(("development", args.dev))
        .extract()?;

    let original_output_path = config.output_path;
    config.output_path = tmp_dir.path().join("public");
    config.original_output_path = original_output_path;

    let file_appender = tracing_appender::rolling::hourly(&config.log_path, "ssg.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let subscriber =
        tracing_subscriber::registry().with(fmt::layer().compact().with_filter(LevelFilter::INFO));

    let file_log = if cfg!(debug_assertions) {
        Some(
            fmt::Layer::default()
                .with_writer(non_blocking)
                .with_ansi(false)
                .with_filter(LevelFilter::TRACE),
        )
    } else {
        None
    };

    let subscriber = subscriber.with(file_log);
    subscriber::set_global_default(subscriber)?;

    info!("Set up subscribers");

    // Clean build
    if args.clean {
        info!("Clean build, making sure existing database removed.");
        ensure_removed("blog.db")?;
        ensure_removed(&config.original_output_path)?;
    }

    let pool = setup_sql()?;
    info!("Connected to database, created connection pool, created tables.");

    let now = Instant::now();
    let mut site = Site::new(pool.get()?, config.clone())?;
    site.build()?;
    let elapsed = now.elapsed();

    info!("Built site in {:.2?} seconds", elapsed);

    info!("Build successful, copying files to final destination.");
    copy_dir_all(tmp_dir.path().join("public"), &config.original_output_path)?;

    Ok(())
}

fn copy_dir_all<T: AsRef<Path>, Z: AsRef<Path>>(src: T, out: Z) -> Result<()> {
    fs::create_dir_all(&out)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            fs::copy(entry.path(), out.as_ref().join(entry.file_name()))?;
        } else {
            copy_dir_all(entry.path(), out.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
