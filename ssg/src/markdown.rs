use chrono::prelude::*;
// use color_eyre::eyre::Context;
use color_eyre::Result;
use comrak::plugins::syntect::SyntectAdapterBuilder;
use comrak::{
    format_html_with_plugins,
    nodes::{AstNode, NodeCode, NodeValue},
    parse_document, Arena, ComrakOptions, ComrakPlugins,
};
use serde::Deserialize;
use syntect::{highlighting::ThemeSet, parsing::SyntaxSet};

// Represents the frontmatter to a blog post.
#[derive(Deserialize, Debug)]
pub struct Frontmatter {
    pub title: String,
    pub tags: Vec<String>,
    pub summary: String,
}

/// A parsed blog post.
/// Contains the content parsed from a markdown document.
#[derive(Debug)]
pub struct ParsedPost {
    pub date: DateTime<Utc>,
    pub content: String,
    pub frontmatter: Frontmatter,
    pub toc: Vec<String>,
}

impl ParsedPost {
    /// Parse a post from some markdown
    pub fn from(content: &str) -> Result<Self> {
        // Load the theme set
        let ss = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_from_folder("themes/")?;

        // Create an adapter and choose the syntax highlighting theme
        let adapter = SyntectAdapterBuilder::new()
            .theme("Catpuccin-frappe")
            .syntax_set(ss)
            .theme_set(theme_set)
            .build();

        // Set the options
        let mut options = ComrakOptions::default();
        options.extension.front_matter_delimiter = Some("---".to_owned());
        options.extension.header_ids = Some("".to_string());
        options.extension.tasklist = true;
        options.extension.strikethrough = true;
        options.render.github_pre_lang = true;
        options.render.unsafe_ = true;

        // Parse table of contents
        let arena = Arena::new();
        let root = parse_document(&arena, content, &options);
        let toc = parse_toc(root);

        // Set the plugins
        let mut plugins = ComrakPlugins::default();
        plugins.render.codefence_syntax_highlighter = Some(&adapter);

        // Parse frontmatter
        let frontmatter = parse_frontmatter(content)?;

        // Format AST to HTML
        let mut html = Vec::new();
        format_html_with_plugins(root, &options, &mut html, &plugins)?;

        Ok(Self {
            date: Utc::now(),
            content: String::from_utf8(html)?,
            frontmatter,
            toc,
        })
    }
}

fn parse_frontmatter(content: &str) -> Result<Frontmatter> {
    let mut frontmatter_content = String::new();
    let mut opening_delim = false;

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

    let frontmatter: Frontmatter = toml::from_str(&frontmatter_content)?;
    Ok(frontmatter)
}

fn parse_toc<'a>(root: &'a AstNode<'a>) -> Vec<String> {
    let mut toc_headers = Vec::new();

    for node in root.children() {
        let header = match node.data.clone().into_inner().value {
            NodeValue::Heading(c) => c,
            _ => continue,
        };

        if header.level != 2 {
            continue;
        }

        let mut text = Vec::new();
        collect_text(node, &mut text);

        // Safe to unwrap since input was good UTF-8 and comrak guarantees
        // output will be good UTF-8
        toc_headers.push(String::from_utf8(text).unwrap());
    }

    toc_headers
}

fn collect_text<'a>(node: &'a AstNode<'a>, output: &mut Vec<u8>) {
    match node.data.borrow().value {
        NodeValue::Text(ref literal) | NodeValue::Code(NodeCode { ref literal, .. }) => {
            output.extend_from_slice(literal);
        }
        NodeValue::LineBreak | NodeValue::SoftBreak => output.push(b' '),
        _ => {
            for n in node.children() {
                collect_text(n, output);
            }
        }
    }
}
