use color_eyre::eyre::{eyre, Result};
use ignore::Walk;
use rsass::{compile_scss_path, output};
use rusqlite::Connection;
use std::{fs, path::PathBuf};
use tracing::info;

enum ToBuild {
    Nonexistent(String),
    Exist,
    No,
}

pub fn compile_stylesheets(
    conn: &Connection,
    css_output_dir: &str,
    scss_input_dir: &str,
) -> Result<()> {
    let mut rendered = 0;
    let mut skipped = 0;

    for result in Walk::new(scss_input_dir) {
        let path = result?.into_path();
        if path.is_dir() {
            continue;
        }

        let to_build = to_build(conn, &path)?;

        match to_build {
            ToBuild::Nonexistent(css_hash) => {
                // If for some wild reason the CSS paths don't contain any non-valid
                // UTF-8 characters, I deserve this panicking.
                let filename = path.file_stem().unwrap().to_str().unwrap();

                let format = output::Format {
                    style: output::Style::Compressed,
                    ..Default::default()
                };
                let css = compile_scss_path(&path, format)?;
                fs::File::create(format!("{css_output_dir}/{filename}.css"))?;
                fs::write(format!("{css_output_dir}/{filename}.css"), css)?;

                conn.execute(
                    "INSERT INTO styles (path, hash) VALUES (?1, ?2) ",
                    (
                        &path
                            .to_str()
                            .ok_or_else(|| eyre!("Error while converting path to string"))?,
                        &css_hash,
                    ),
                )?;

                rendered += 1;
            }
            ToBuild::Exist => {
                // If for some wild reason the CSS paths don't contain any non-valid
                // UTF-8 characters, I deserve this panicking.
                let filename = path.file_stem().unwrap().to_str().unwrap();

                let format = output::Format {
                    style: output::Style::Compressed,
                    ..Default::default()
                };
                let css = compile_scss_path(&path, format)?;
                fs::write(format!("{css_output_dir}/{filename}.css"), css)?;

                rendered += 1;
            }
            ToBuild::No => skipped += 1,
        }
    }

    info!("Built {rendered} stylesheets");
    info!("{skipped} stylesheets left unchanged, skipping");

    Ok(())
}

fn to_build(conn: &Connection, path: &PathBuf) -> Result<ToBuild> {
    let raw_css = fs::read_to_string(path)?;
    let css_hash = format!("{:016x}", seahash::hash(raw_css.as_bytes()));
    let mut stmt = conn.prepare("SELECT hash FROM styles WHERE path = :path")?;
    let path_str = path
        .to_str()
        .ok_or_else(|| eyre!("Error while converting path to string"))?;

    let hashes_iter = stmt.query_map(&[(":path", path_str)], |row| row.get(0))?;
    let mut hashes: Vec<String> = Vec::new();
    for hash in hashes_iter {
        hashes.push(hash?);
    }

    let build = if hashes.is_empty() {
        ToBuild::Nonexistent(css_hash)
    } else if hashes[0] != css_hash {
        conn.execute(
            "
            UPDATE styles SET hash = (:hash) WHERE path = (:path)",
            &[(":hash", &css_hash), (":path", &path_str.to_string())],
        )?;
        ToBuild::Exist
    } else {
        ToBuild::No
    };

    Ok(build)
}
