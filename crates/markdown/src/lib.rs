#![warn(clippy::pedantic, clippy::nursery)]

mod summary;

use chrono::{DateTime, Utc};
use color_eyre::Result;
use comrak::{
    format_html_with_plugins,
    nodes::{AstNode, NodeCode, NodeValue},
    parse_document,
    plugins::syntect::{SyntectAdapter, SyntectAdapterBuilder},
    Arena, ComrakOptions, ComrakPlugins,
};
use serde::Deserialize;
use syntect::{highlighting::ThemeSet, parsing::SyntaxSet};

/// The frontmatter for a parsed document
#[derive(Debug, Deserialize)]
pub struct Frontmatter {
    pub title: String,
    pub tags: Vec<String>,
    pub series: Option<String>,
}

/// Renders some content to markdown
#[derive(Debug)]
pub struct MarkdownRenderer {
    adapter: SyntectAdapter,
    options: ComrakOptions,
}

/// Represent a parsed markdown document
pub struct Document {
    pub date: DateTime<Utc>,
    pub content: String,
    pub frontmatter: Frontmatter,
    pub toc: Option<Vec<String>>,
    pub summary: String,
}

impl MarkdownRenderer {
    #[tracing::instrument]
    pub fn new() -> Result<Self> {
        // Load the theme set
        let ss = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_from_folder("themes/")?;

        // Create an adapter and choose the syntax highlighting theme.
        let adapter = SyntectAdapterBuilder::new()
            .syntax_set(ss)
            .theme_set(theme_set)
            .theme("Catpuccin-frappe")
            .build();

        // Set the options
        let mut options = ComrakOptions::default();
        options.extension.front_matter_delimiter = Some("---".to_string());
        options.extension.header_ids = Some(String::new());
        options.extension.tasklist = true;
        options.extension.strikethrough = true;
        options.render.github_pre_lang = true;
        options.render.unsafe_ = true;

        Ok(Self { adapter, options })
    }

    #[tracing::instrument]
    pub fn render(&self, content: &str) -> Result<Document> {
        let arena = Arena::new();
        let root = parse_document(&arena, content, &self.options);
        let toc = self.parse_toc(root);

        let mut plugins = ComrakPlugins::default();
        plugins.render.codefence_syntax_highlighter = Some(&self.adapter);

        let frontmatter = self.parse_frontmatter(content)?;

        // Format AST to HTML
        let mut html = Vec::new();
        format_html_with_plugins(root, &self.options, &mut html, &plugins)?;

        let string_html = String::from_utf8(html)?;

        Ok(Document {
            date: Utc::now(),
            summary: summary::get_summary(&string_html)?,
            content: string_html,
            frontmatter,
            toc: (!toc.is_empty()).then_some(toc),
        })
    }

    #[tracing::instrument]
    fn parse_toc<'a>(&self, root: &'a AstNode<'a>) -> Vec<String> {
        let mut toc_headers = Vec::new();

        for node in root.children() {
            let NodeValue::Heading(header) = node.data.clone().into_inner().value else {
                continue;
            };

            if header.level != 2 {
                continue;
            }

            let mut text = Vec::new();
            self.collect_text(node, &mut text);

            // Safe to unwrap since input was good UTF-8 and comrak guarantees output will be good UTF-8
            toc_headers.push(String::from_utf8(text).unwrap());
        }

        toc_headers
    }

    #[tracing::instrument]
    fn collect_text<'a>(&self, node: &'a AstNode<'a>, output: &mut Vec<u8>) {
        match node.data.borrow().value {
            NodeValue::Text(ref literal) | NodeValue::Code(NodeCode { ref literal, .. }) => {
                output.extend_from_slice(literal.as_bytes());
            }
            NodeValue::LineBreak | NodeValue::SoftBreak => output.push(b' '),
            _ => {
                node.children().for_each(|n| self.collect_text(n, output));
            }
        }
    }

    #[tracing::instrument]
    fn parse_frontmatter(&self, content: &str) -> Result<Frontmatter> {
        let mut opening_delim = false;
        let mut frontmatter_content = String::new();

        for line in content.lines() {
            if line.trim() == "---" {
                if opening_delim {
                    break;
                }

                opening_delim = true;
                continue;
            }

            frontmatter_content.push_str(line);
            frontmatter_content.push('\n');
        }

        let frontmatter = toml::from_str(&frontmatter_content)?;
        Ok(frontmatter)
    }
}
