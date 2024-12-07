use std::{
    fmt::Debug,
    fs,
    hash::Hash,
    path::{Component, Path, PathBuf},
};

use color_eyre::{eyre::ContextCompat, Result};
use markdown::Document;
use minify_html::{minify, Cfg};
use serde::{Deserialize, Serialize};
use tera::Context as TeraContext;
use tracing::trace;

use crate::{
    context::Context, output::Output, shortcodes::evaluate_shortcodes, utils::fs::ensure_directory,
    DATE_FORMAT,
};

/// Represents a single markdown page.
#[derive(Debug, Serialize, Deserialize, Clone, Eq)]
pub struct Page {
    pub path: PathBuf,
    pub out_path: PathBuf,
    pub permalink: String,
    pub raw_content: String,
    pub hash: String,
    pub fresh: bool,
    pub is_special: bool,
    pub document: Document,
}

impl Hash for Page {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.path.hash(state);
    }
}

impl PartialEq for Page {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Page {
    #[tracing::instrument(level = tracing::Level::DEBUG)]
    pub fn new<P: AsRef<Path> + Debug>(
        ctx: &Context,
        path: P,
        raw_content: String,
        hash: String,
        fresh: bool,
    ) -> Result<Self> {
        trace!("Processing page at {path:?}");

        let evaluated_content = evaluate_shortcodes(ctx, &raw_content)?;
        let document = ctx.markdown_renderer.render(&evaluated_content)?;
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

        trace!("Finished processing page at {path:?}");

        Ok(Self {
            path: path.as_ref().to_owned(),
            out_path,
            permalink,
            raw_content,
            hash,
            fresh,
            is_special: is_special_page(path, &ctx.config.special_pages),
            document,
        })
    }
}

impl Output for Page {
    #[tracing::instrument(level = tracing::Level::DEBUG)]
    fn write(&self, ctx: &Context) -> Result<()> {
        trace!(
            "Writing page at {:?} to disk at {:?}",
            self.path,
            self.out_path
        );

        let frontmatter = &self.document.frontmatter;

        // Insert template context
        let mut context = TeraContext::new();
        context.insert("title", &frontmatter.title);
        context.insert("tags", &frontmatter.tags.join(", "));
        context.insert("slug", &frontmatter.slug);
        context.insert("series", &frontmatter.series);
        context.insert("date", &self.document.date.format(DATE_FORMAT).to_string());
        context.insert("toc", &self.document.toc);
        context.insert("markup", &self.document.content);
        context.insert("summary", &self.document.summary);
        context.insert("frontmatter", &frontmatter);
        context.insert("posts", &ctx.posts);
        context.insert("index_pages", &ctx.special_pages);

        let template = frontmatter
            .template
            .as_ref()
            .map_or("post.html.tera", |s| s);
        let rendered_html = ctx.tera.render(template, &context)?;

        let cfg = Cfg::new();
        let minified = minify(rendered_html.as_bytes(), &cfg);

        trace!(
            "Rendered template for page at {:?}, now writing to {:?}",
            self.path,
            self.out_path
        );
        fs::write(&self.out_path, minified)?;
        trace!(
            "Wrote page at {:?} to disk at {:?}",
            self.path,
            self.out_path
        );

        Ok(())
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn hash(&self) -> &str {
        &self.hash
    }

    fn fresh(&self) -> bool {
        self.fresh
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
