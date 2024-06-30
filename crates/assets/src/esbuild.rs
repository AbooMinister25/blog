use async_std::task;
use color_eyre::{
    eyre::{bail, ContextCompat},
    Result,
};
use esbuild_rs::{build, BuildOptionsBuilder, Engine, EngineName, SourceMap};
use std::{fmt::Debug, path::Path};

pub fn bundle_js<T: AsRef<Path> + Debug>(path: T, out_path: T) -> Result<()> {
    let mut options_builder = BuildOptionsBuilder::new();
    options_builder.entry_points.push(
        path.as_ref()
            .to_str()
            .context("Path should be valid UTf-8.")?
            .to_string(),
    );
    options_builder.bundle = true;
    options_builder.minify_syntax = true;
    options_builder.minify_identifiers = true;
    options_builder.minify_whitespace = true;
    options_builder.source_map = SourceMap::Linked;
    options_builder.engines = vec![
        Engine {
            name: EngineName::Chrome,
            version: "58".to_string(),
        },
        Engine {
            name: EngineName::Firefox,
            version: "57".to_string(),
        },
        Engine {
            name: EngineName::Safari,
            version: "11".to_string(),
        },
        Engine {
            name: EngineName::Edge,
            version: "16".to_string(),
        },
    ];
    options_builder.write = true;
    options_builder.outfile = out_path
        .as_ref()
        .to_str()
        .context("Path should be valid UTF-8.")?
        .to_string();

    let options = options_builder.build();
    let res = task::block_on(build(options));

    let errors = res.errors.as_ref();
    if !errors.is_empty() {
        bail!(
            "The following errors occurred while bundling with esbuild: {}",
            errors
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>()
                .join("\n")
        )
    }

    Ok(())
}
