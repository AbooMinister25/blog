#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_errors_doc)]

use std::{
    default::Default,
    fmt::Debug,
    fs,
    path::{Path, PathBuf},
};

use color_eyre::Result;
use entry::Entry;
use markdown::MarkdownRenderer;
use tera::{Context, Tera};
use tracing::trace;
use utils::fs::ensure_directory;

pub const DATE_FORMAT: &str = "%b %e, %Y";

#[derive(Debug, Default)]
pub struct Post {
    pub path: PathBuf,
    pub raw_content: String,
    pub content: String,
}

impl Post {
    #[tracing::instrument]
    pub fn new(path: PathBuf, content: String) -> Self {
        Self {
            path,
            raw_content: content,
            ..Default::default()
        }
    }

    #[tracing::instrument(skip(renderer))]
    pub fn render<T: AsRef<Path> + Debug>(
        &mut self,
        tera: &Tera,
        renderer: &MarkdownRenderer,
        output_directory: T,
    ) -> Result<()> {
        ensure_directory(output_directory.as_ref().join("posts/"))?;

        trace!("Rendering post at {:?}", self.path);

        let document = renderer.render(&self.raw_content)?;
        let out_path = output_directory
            .as_ref()
            .join("posts/")
            .join(format!("{}.html", document.frontmatter.title));

        trace!("Rendering post to {:?}", out_path);

        let mut context = Context::new();
        context.insert("title", &document.frontmatter.title);
        context.insert("tags", &document.frontmatter.tags.join(", "));
        context.insert("date", &document.date.format(DATE_FORMAT).to_string());
        context.insert("toc", &document.toc);
        context.insert("markup", &document.content);
        context.insert("summary", &document.summary);

        let rendered_html = tera.render("post.html.tera", &context)?;
        fs::write(out_path, &rendered_html)?;
        self.content = rendered_html;

        trace!("Rendered post");

        Ok(())
    }
}

impl From<Entry> for Post {
    fn from(value: Entry) -> Self {
        Self::new(value.path, value.content)
    }
}
