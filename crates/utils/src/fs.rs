use std::fs;
use std::path::Path;

use color_eyre::Result;

// If the given directory doesn't exist, creates it.
pub fn ensure_directory<T: AsRef<Path>>(path: T) -> Result<()> {
    if !path.as_ref().exists() {
        fs::create_dir_all(path)?;
    }

    Ok(())
}
