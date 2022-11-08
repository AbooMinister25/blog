use color_eyre::eyre::Result;
use ignore::Walk;
use rsass::{compile_scss_path, output};
use std::fs;
use tracing::info;

#[tracing::instrument]
pub fn compile_stylesheets(css_output_dir: &str, scss_input_dir: &str) -> Result<()> {
    let mut built = 0;

    for result in Walk::new(scss_input_dir) {
        let path = result?.into_path();
        if path.is_dir() {
            continue;
        }

        // If for some wild reason the CSS paths don't contain any non-valid
        // UTF-8 characters, I deserve for this to panic.
        let filename = path.file_stem().unwrap().to_str().unwrap();

        let format = output::Format {
            style: output::Style::Compressed,
            ..Default::default()
        };
        let css = compile_scss_path(&path, format)?;
        fs::File::create(format!("{css_output_dir}/{filename}.css"))?;
        fs::write(format!("{css_output_dir}/{filename}.css"), css)?;

        built += 1;
    }

    info!("Built {built} stylesheets");

    Ok(())
}
