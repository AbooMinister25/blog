use color_eyre::eyre::Result;
use std::{fs, path::Path};

// If the given directory doesn't exist, creates it.
pub fn ensure_directory(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }

    Ok(())
}
