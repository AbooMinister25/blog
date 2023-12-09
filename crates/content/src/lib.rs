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

use color_eyre::{eyre::ContextCompat, Result};
use entry::Entry;
use markdown::MarkdownRenderer;
use tera::{Context, Tera};
use tracing::trace;
use utils::fs::ensure_directory;

pub const DATE_FORMAT: &str = "%b %e, %Y";

#[derive(Debug, Default)]
pub struct Page {
    pub path: PathBuf,
    pub raw_content: String,
    pub content: String,
}

impl Page {
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
        trace!("Rendering post at {:?}", self.path);

        let document = renderer.render(&self.raw_content)?;
        let out_path = self.out_path(output_directory, &document.frontmatter.title)?;
        ensure_directory(out_path.parent().context("Path should have a parent")?)?;

        trace!("Rendering post to {:?}", out_path);

        let mut context = Context::new();
        context.insert("title", &document.frontmatter.title);
        context.insert("tags", &document.frontmatter.tags.join(", "));
        context.insert("series", &document.frontmatter.series);
        context.insert("date", &document.date.format(DATE_FORMAT).to_string());
        context.insert("toc", &document.toc);
        context.insert("markup", &document.content);
        context.insert("summary", &document.summary);

        let rendered_html = tera.render("post.html.tera", &context)?;
        fs::write(&out_path, &rendered_html)?;
        self.content = rendered_html;

        trace!("Rendered post at {:?}", out_path);

        Ok(())
    }

    #[tracing::instrument]
    fn out_path<T: AsRef<Path> + Debug>(
        &self,
        output_directory: T,
        title: &str,
    ) -> Result<PathBuf> {
        let parent = self
            .path
            .parent()
            .context("Path should have a parent")?
            .file_name()
            .context("Path should have a filename")?
            .to_string_lossy();

        let filename = self
            .path
            .file_name()
            .context("Path should have a filename")?
            .to_string_lossy();

        let directory = output_directory.as_ref();

        match (parent.as_ref(), filename.as_ref()) {
            ("content", "index") => Ok(directory.join("index.html")),
            ("content", _) => Ok(directory.join(format!("{title}.html"))),
            (_, "index") => Ok(directory.join(parent.as_ref()).join("index.html")),
            ("posts", _) => Ok(directory
                .join(parent.as_ref())
                .join(format!("{title}.html"))),
            _ => Ok(directory
                .join("series")
                .join(parent.as_ref())
                .join(format!("{title}.html"))),
        }
    }
}

impl From<Entry> for Page {
    fn from(value: Entry) -> Self {
        Self::new(
            value.path,
            String::from_utf8_lossy(&value.raw_content).to_string(),
        )
    }
}
