use color_eyre::eyre::Result;
use std::{fs, path::Path};

// If the given directory doesn't exist, creates it.
pub fn ensure_directory(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }

    Ok(())
}

// If the given file exists, delete it.
pub fn ensure_removed(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_file(path)?;
    }

    Ok(())
}
