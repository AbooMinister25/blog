use color_eyre::eyre::Result;
use ignore::Walk;
use rsass::{compile_scss_path, output};
use std::fs;

pub fn compile_stylesheets() -> Result<()> {
    for result in Walk::new("sass/") {
        let path = result?.into_path();
        if path.is_dir() {
            continue;
        }

        // hopefully safe to unwrap here since I'm sure the CSS paths wont
        // contain any non-valid UTF-8 characters, and if they do, its my
        // fault and I deserve if it panics.
        let filename = path.file_stem().unwrap().to_str().unwrap();

        let format = output::Format {
            style: output::Style::Compressed,
            ..Default::default()
        };
        let css = compile_scss_path(&path, format)?;
        fs::File::create(format!("styles/{}.css", filename))?;
        fs::write(format!("styles/{}.css", filename), css)?;
    }

    Ok(())
}
