use chrono::prelude::*;
use color_eyre::Result;
use comrak::plugins::syntect::SyntectAdapter;
use comrak::{markdown_to_html_with_plugins, ComrakOptions, ComrakPlugins};
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
    let adapter = SyntectAdapter::new("Solarized (light)");
    let mut options = ComrakOptions::default();
    options.extension.front_matter_delimiter = Some("---".to_owned());
    options.extension.header_ids = Some("".to_string());
    options.render.github_pre_lang = true;

    let mut plugins = ComrakPlugins::default();
    plugins.render.codefence_syntax_highlighter = Some(&adapter);

    let frontmatter = parse_frontmatter(content)?;
    let html = markdown_to_html_with_plugins(content, &options, &plugins);

    let today = Utc::now();

    Ok(ParsedPost {
        date: today,
        content: html,
        frontmatter,
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
