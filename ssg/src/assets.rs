use color_eyre::eyre::{eyre, Result};
use ignore::Walk;
use rusqlite::Connection;
use std::process::{Command, Stdio};
use std::{fs, path::PathBuf};
use tracing::info;

enum ToProcess {
    Nonexistent(String),
    Exist(String),
    No,
}

pub fn process_assets(conn: &Connection) -> Result<()> {
    let mut processed = 0;
    let mut skipped = 0;

    for result in Walk::new("assets/") {
        let path = result?.into_path();
        if path.is_dir() {
            continue;
        }

        if path.extension().expect("if this fails, its my fault") != ".svg" {
            continue;
        }

        let to_build = to_build(conn, &path)?;
        let path_str = path
            .to_str()
            .ok_or_else(|| eyre!("Error while converting path to string"))?;

        match to_build {
            ToProcess::Nonexistent(raw_asset) => {
                invoke_svgo(path_str)?;
                let asset_hash = format!("{:016x}", seahash::hash(raw_asset.as_bytes()));
                // Write asset into database
                conn.execute(
                    "INSERT INTO assets (path, hash) VALUES (?1, ?2)",
                    (path_str, &asset_hash),
                )?;

                processed += 1;
            }
            ToProcess::Exist(raw_asset) => {
                invoke_svgo(path_str)?;
                let asset_hash = format!("{:016x}", seahash::hash(raw_asset.as_bytes()));
                // Update hash in database
                conn.execute(
                    "UPDATE posts SET hash = (:hash) WHERE path = (:path)",
                    &[(":hash", &asset_hash), (":path", &path_str.to_string())],
                )?;

                processed += 1;
            }
            ToProcess::No => skipped += 1,
        }
    }

    info!("Minimized {processed} assets");
    info!("{skipped} assets left unchanged, skipping");

    Ok(())
}

fn to_build(conn: &Connection, path: &PathBuf) -> Result<ToProcess> {
    let raw_asset = fs::read_to_string(path)?;
    // Hash the asset, format as string.
    let asset_hash = format!("{:016x}", seahash::hash(raw_asset.as_bytes()));
    let mut stmt = conn.prepare("SELECT hash FROM assets WHERE path = :path")?;
    let path_str = path
        .to_str()
        .ok_or_else(|| eyre!("Error while converting path to string"))?;

    // Get the hashes found for this path
    let hashes_iter = stmt.query_map(&[(":path", path_str)], |row| row.get(0))?;
    let mut hashes: Vec<String> = Vec::new();
    for hash in hashes_iter {
        hashes.push(hash?);
    }

    // If the hashes are empty, a new file was created. If it is different from the new
    // hash, then the files contents changed. Otherwise the file was not changed.
    let build = if hashes.is_empty() {
        ToProcess::Nonexistent(raw_asset)
    } else if hashes[0] != asset_hash {
        ToProcess::Exist(raw_asset)
    } else {
        ToProcess::No
    };

    Ok(build)
}

fn invoke_svgo(path_str: &str) -> Result<()> {
    // Invoke svgo to minimize svg
    let mut cmd = Command::new("svgo")
        .arg(path_str)
        .arg("-o")
        .arg(path_str)
        .stdout(Stdio::null())
        .spawn()?;

    // Wait for the process to end, since we hash the minified data
    cmd.wait()?;

    Ok(())
}
