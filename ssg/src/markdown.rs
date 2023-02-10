use chrono::{DateTime, Utc};
use color_eyre::Result;
use comrak::plugins::syntect::{SyntectAdapter, SyntectAdapterBuilder};
use comrak::{
    format_html_with_plugins,
    nodes::{AstNode, NodeCode, NodeValue},
    parse_document, Arena, ComrakOptions, ComrakPlugins,
};
use serde::Deserialize;
use std::{fs, path::Path};
use syntect::{highlighting::ThemeSet, parsing::SyntaxSet};

// Represents the frontmatter to a parsed document
#[derive(Debug, Deserialize)]
pub struct Frontmatter {
    pub title: String,
    pub tags: Vec<String>,
    pub series: Option<String>,
}

// Represents a parsed markdown document
#[derive(Debug)]
pub struct Document {
    pub date: DateTime<Utc>,
    pub content: String,
    pub frontmatter: Frontmatter,
    pub toc: Option<Vec<String>>,
}

impl Document {
    #[tracing::instrument]
    pub fn from_file(path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        Self::from_string(&contents)
    }

    #[tracing::instrument]
    pub fn from_string(content: &str) -> Result<Self> {
        // Set up syntax highlighter and render options
        let (options, adapter) = setup_comrak()?;

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
            toc: (!toc.is_empty()).then_some(toc),
        })
    }
}

#[tracing::instrument]
fn setup_comrak() -> Result<(ComrakOptions, SyntectAdapter<'static>)> {
    // Load the theme set
    let ss = SyntaxSet::load_defaults_newlines();
    let theme_set = ThemeSet::load_from_folder("themes/")?;
    // println!("{:?}", theme_set.themes["Catpuccin-frappe"]);

    // Create an adapter and choose the syntax highlighting theme
    let adapter = SyntectAdapterBuilder::new()
        .syntax_set(ss)
        .theme_set(theme_set)
        .theme("Catpuccin-frappe")
        .build();

    // Set the options
    let mut options = ComrakOptions::default();
    options.extension.front_matter_delimiter = Some("---".to_owned());
    options.extension.header_ids = Some("".to_string());
    options.extension.tasklist = true;
    options.extension.strikethrough = true;
    options.render.github_pre_lang = true;
    options.render.unsafe_ = true;

    Ok((options, adapter))
}

#[tracing::instrument]
fn parse_frontmatter(content: &str) -> Result<Frontmatter> {
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

#[tracing::instrument]
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

        // Safe to unwrap since input was good UTF-8 and comrak guarantees output will be good UTF-8
        toc_headers.push(String::from_utf8(text).unwrap());
    }

    toc_headers
}

#[tracing::instrument]
fn collect_text<'a>(node: &'a AstNode<'a>, output: &mut Vec<u8>) {
    match node.data.borrow().value {
        NodeValue::Text(ref literal) | NodeValue::Code(NodeCode { ref literal, .. }) => {
            output.extend_from_slice(literal);
        }
        NodeValue::LineBreak | NodeValue::SoftBreak => output.push(b' '),
        _ => {
            node.children().for_each(|n| collect_text(n, output));
        }
    }
}
