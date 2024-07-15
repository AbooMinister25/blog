#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::redundant_pub_crate)]

mod server;

use std::fs;
use std::path::Path;
use std::time::Instant;

use clap::Parser;
use color_eyre::Result;
use config::Config;
use figment::{
    providers::{Format, Serialized, Toml},
    Figment,
};
use notify::{Config as NotifyConfig, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Serialize;
use site::Site;
use sql::setup_sql;
use tempfile::Builder;
use tokio::signal::ctrl_c;
use tokio::sync::mpsc;
use tower_livereload::LiveReloadLayer;
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
#[tokio::main]
async fn main() -> Result<()> {
    let now = Instant::now();

    // Install panic and error report handlers
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

    let pool = setup_sql()?;
    info!("Connected to database, created tables");

    let mut site = Site::new(pool.get()?, config.clone())?;

    tokio::task::spawn_blocking(move || site.build()).await??;

    info!("Built site");

    let elapsed = now.elapsed();
    info!("Built in {:.2?} seconds", elapsed);

    if args.dev {
        info!("Development mode enabled, hot reloading on file system changes.");

        let livereload = LiveReloadLayer::new();
        let reloader = livereload.reloader();

        let (tx, mut rx) = mpsc::channel(32);
        let mut watcher = RecommendedWatcher::new(
            move |res| {
                tx.blocking_send(res)
                    .expect("Problem while sending message.");
            },
            NotifyConfig::default(),
        )?;
        watcher.watch(&config.root, RecursiveMode::Recursive)?;

        let c2 = config.clone();
        let op = original_output_path.clone();
        let tp = tmp_dir.path().join("public");
        let t1 = tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(res) = rx.recv() => {
                        let pool = pool.clone();
                        let config = c2.clone();
                        if res.is_ok_and(|e| e.kind.is_modify() || e.kind.is_create()) {
                            info!("Building site");
                            let now = Instant::now();
                            tokio::task::spawn_blocking(move || {
                                let mut site = Site::new(pool.get()?, config)?;
                                site.build()
                            })
                            .await??;

                            let elapsed = now.elapsed();

                            info!("Built site.");
                            info!("Built in {:.2?} seconds", elapsed);

                            info!("Build successful, copying files to final destination.");
                            copy_dir_all(&tp, &op)?;

                            reloader.reload();
                        }
                    },
                    _ = tokio::signal::ctrl_c() => {
                        break;
                    }
                }
            }
            Ok::<(), color_eyre::Report>(())
        });

        let op = original_output_path.clone();
        let t2 = tokio::spawn(async move { server::serve(livereload, op).await });
        let t3 = tokio::spawn(async move {
            ctrl_c().await?;
            tmp_dir.close()?;

            Ok::<(), color_eyre::Report>(())
        });

        t1.await??;
        t2.await??;
        t3.await??;
    } else {
        info!("Build successful, copying files to final destination.");
        copy_dir_all(tmp_dir.path().join("public"), original_output_path)?;
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
