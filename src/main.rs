use std::{fs, path::Path, time::Instant};

use clap::Parser;
use color_eyre::Result;
use figment::{
    providers::{Format, Serialized, Toml},
    Figment,
};
use notify::{Config as NotifyConfig, Event, RecommendedWatcher, RecursiveMode, Watcher};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use site::{config::Config, sql::setup_sql, utils::fs::ensure_removed, Site};
use tempfile::Builder;
use tokio::{
    signal::ctrl_c,
    sync::mpsc::{self, Receiver},
};
use tower_livereload::{LiveReloadLayer, Reloader};
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

    /// Run a web server and hot reload on file changes.
    #[clap(long, action)]
    watch: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
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

    setup_tracing(&config)?;
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

    if args.watch {
        info!("Development mode enabled, hot reloading on file system changes.");

        let livereload = LiveReloadLayer::new();
        let reloader = livereload.reloader();

        let (tx, rx) = mpsc::channel(32);
        let mut watcher = RecommendedWatcher::new(
            move |res| {
                tx.blocking_send(res)
                    .expect("Problem while sending message.");
            },
            NotifyConfig::default(),
        )?;
        watcher.watch(&config.root, RecursiveMode::Recursive)?;

        let op = config.original_output_path.clone();

        let t1 = tokio::spawn(live_reload(config.clone(), rx, reloader, pool));
        let t2 = tokio::spawn(async move { server::serve(livereload, config.output_path).await });
        let t3 = tokio::spawn(async move {
            ctrl_c().await?;

            info!("Server stopped, copying files to final destination.");
            copy_dir_all(tmp_dir.path().join("public"), op)?;
            tmp_dir.close()?;

            Ok::<(), color_eyre::Report>(())
        });

        t1.await??;
        t2.await??;
        t3.await??;
    } else {
        info!("Build successful, copying files to final destination.");
        copy_dir_all(tmp_dir.path().join("public"), config.original_output_path)?;
    }

    Ok(())
}

fn setup_tracing(config: &Config) -> Result<()> {
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

    Ok(())
}

#[allow(clippy::redundant_pub_crate)]
async fn live_reload(
    config: Config,
    mut rx: Receiver<Result<Event, notify::Error>>,
    reloader: Reloader,
    pool: Pool<SqliteConnectionManager>,
) -> Result<()> {
    loop {
        tokio::select! {
            Some(res) = rx.recv() => {
                let pool = pool.clone();
                let config = config.clone();
                if res.is_ok_and(|e| e.kind.is_modify() || e.kind.is_create()) {
                    info!("Building site");
                    let now = Instant::now();
                    tokio::task::spawn_blocking(move || {
                        let mut site = Site::new(pool.get()?, config)?;
                        site.build()
                    })
                    .await??;

                    let elapsed = now.elapsed();

                    info!("Built site in {:.2?} seconds", elapsed);

                    reloader.reload();
                }
            },
            _ = tokio::signal::ctrl_c() => {
                break;
            }
        }
    }

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
