use std::{
    fmt::Debug,
    fs,
    path::{Component, Path, PathBuf},
};

use chrono::{DateTime, Utc};
use color_eyre::{eyre::ContextCompat, Result};
use markdown::{Document, Frontmatter, MarkdownRenderer};
use serde::{Deserialize, Serialize};
use tera::{Context as TeraContext, Tera};
use tracing::trace;

use crate::{context::Context, utils::fs::ensure_directory, DATE_FORMAT};

/// Represents a single markdown page.
#[derive(Debug)]
pub struct Page {
    pub path: PathBuf,
    pub permalink: String,
    pub raw_content: String,
    pub hash: String,
    pub new: bool,
    pub special: bool,
    pub document: Document,
}

impl Page {
    #[tracing::instrument]
    pub fn new<P: AsRef<Path> + Debug>(
        ctx: &Context,
        path: P,
        raw_content: String,
        hash: String,
        new: bool,
    ) -> Result<Self> {
        trace!("Processing page at {path:?}");

        let document = ctx.markdown_renderer.render(&raw_content)?;
        let out_path = out_path(
            &path,
            &ctx.config.output_path,
            &document.frontmatter.title,
            document.frontmatter.slug.as_deref(),
        );
        ensure_directory(out_path.parent().context("Path should have a parent")?)?;

        let permalink = {
            let mut components = out_path.components();
            let out = ctx
                .config
                .output_path
                .file_name()
                .context("Output directory shouldn't terminate in ..")?;

            for c in components.by_ref() {
                if let Component::Normal(o) = c {
                    if o == out {
                        break;
                    }
                }
            }
            components.next_back();
            format!(
                "{}{}",
                ctx.config.url,
                components
                    .as_path()
                    .to_str()
                    .context("Path should be valid unicode")?,
            )
        };

        Ok(Self {
            path: out_path,
            permalink,
            raw_content,
            hash,
            new,
            special: is_special_page(path, &ctx.config.special_pages),
            document,
        })
    }
}

fn out_path<P: AsRef<Path>, T: AsRef<Path>>(
    path: P,
    output_path: T,
    title: &str,
    slug: Option<&str>,
) -> PathBuf {
    let directory = output_path.as_ref();

    let ending = if path.as_ref().ends_with("index.md") {
        PathBuf::from("index.html")
    } else {
        PathBuf::from(slug.map_or_else(|| title.replace(' ', "-"), ToOwned::to_owned))
            .join("index.html")
    };

    let mut components = path.as_ref().components();
    components.next_back();

    directory
        .join(components.skip(2).collect::<PathBuf>())
        .join(ending)
}

fn is_special_page<T: AsRef<Path>>(path: T, special_pages: &[String]) -> bool {
    special_pages
        .iter()
        .any(|ending| path.as_ref().ends_with(ending))
}
