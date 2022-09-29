use chrono::prelude::*;
use color_eyre::eyre::Context;
use color_eyre::Result;
use comrak::plugins::syntect::SyntectAdapter;
use comrak::{
    format_html_with_plugins,
    nodes::{AstNode, NodeCode, NodeValue},
    parse_document, Arena, ComrakOptions, ComrakPlugins,
};
use serde::Deserialize;

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

/// Parse some markdown and return a `ParsedPost`
///
/// # Examples
///
/// ```
/// use blog::util::markdown::parse;
///
/// let markdown_content = "
/// title: test
///
/// ## Hello, World
/// ### Yay
/// ";
///
/// let parsed_markdown = parse(markdown_content).expect("Error while parsing");
///
/// assert_eq!(&parsed_markdown.title, "test");
/// assert_eq!(&parsed_markdown.content, "<h1>Hello, World</h1>\n<h2>Yay</h2>\n");
/// ```
///
/// # Errors
/// This function can return an error if a required
/// header is found to be missing, or some error occurred during
/// parsing.
pub fn parse(content: &str) -> Result<ParsedPost> {
    // Choose the syntax highlighting theme
    let adapter = SyntectAdapter::new("Solarized (light)");

    // Set the options
    let mut options = ComrakOptions::default();
    options.extension.front_matter_delimiter = Some("---".to_owned());
    options.extension.header_ids = Some("".to_string());
    options.extension.tasklist = true;
    options.extension.strikethrough = true;
    options.render.github_pre_lang = true;
    options.render.unsafe_ = true;

    // Parse the table of contents
    let arena = Arena::new();
    let root = parse_document(&arena, content, &options);
    let toc = parse_toc(root);

    // Set the plugins
    let mut plugins = ComrakPlugins::default();
    plugins.render.codefence_syntax_highlighter = Some(&adapter);

    // Parse the frontmatter and format the AST
    let frontmatter = parse_frontmatter(content)?;
    let mut html = Vec::new();
    format_html_with_plugins(root, &options, &mut html, &plugins)?;

    let today = Utc::now();
    Ok(ParsedPost {
        date: today,
        content: String::from_utf8(html).wrap_err("why is this invalid utf-8 you suck")?,
        frontmatter,
        toc,
    })
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
