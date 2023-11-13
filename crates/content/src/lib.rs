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
use tracing::{info, trace};
use utils::fs::ensure_directory;

pub const DATE_FORMAT: &str = "%b %e, %Y";

#[derive(Debug, Default)]
pub struct Page {
    pub path: PathBuf,
    pub raw_content: String,
    pub content: String,
}

// #[derive(Debug)]
// enum ContentType {
//     Post,
//     Series,
//     Page,
// }

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
        let directory = output_directory.as_ref().join(self.directory());
        ensure_directory(&directory)?;

        trace!("Rendering post at {:?}", self.path);

        let document = renderer.render(&self.raw_content)?;
        let out_path = directory.join(format!("{}.html", document.frontmatter.title));

        trace!("Rendering post to {:?}", out_path);

        let mut context = Context::new();
        context.insert("title", &document.frontmatter.title);
        context.insert("tags", &document.frontmatter.tags.join(", "));
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
    fn directory(&self) -> PathBuf {
        self.path.parent().map_or_else(
            || Path::new("").to_path_buf(),
            |parent| {
                if parent == Path::new("blog/") {
                    Path::new("").to_path_buf()
                } else if parent == Path::new("blog/posts/") {
                    Path::new("posts").to_path_buf()
                } else {
                    Path::new("series/")
                        .join(parent.file_name().expect("Path should have a filename"))
                }
            },
        )
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
