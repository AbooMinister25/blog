use color_eyre::eyre::Result;
use std::fs;
use std::path::Path;

// If the given directory doesn't exist, creates it.
pub fn ensure_directory<T: AsRef<Path>>(path: T) -> Result<()> {
    if !path.as_ref().exists() {
        fs::create_dir_all(path)?;
    }

    Ok(())
}

// If the given file exists, delete it.
pub fn ensure_removed<T: AsRef<Path>>(path: T) -> Result<()> {
    let path = path.as_ref();

    if path.exists() {
        if path.is_dir() {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
    }

    Ok(())
}
