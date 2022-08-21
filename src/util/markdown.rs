use crate::util::response::ErrorKind;

use chrono::prelude::*;
use lazy_static::lazy_static;
use pulldown_cmark::{html, CodeBlockKind, Event, Options, Parser, Tag};
use regex::Regex;
use std::collections::HashMap;
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

/// A parsed blog post.
/// Contains the content parsed from a markdown document.
#[derive(Debug)]
pub struct ParsedPost {
    pub title: String,
    pub date: NaiveDateTime,
    pub content: String,
    pub tags: Vec<String>,
}

/// Parse some markdown and return a `ParsedPost`rust
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
/// This function can return an `Err(ErrorKind)` if a required
/// header is found to be missing, or some error occurred during
/// parsing.
pub fn parse(content: &str) -> Result<ParsedPost, ErrorKind> {
    // Parse the headers in the markdown
    let headers = parse_headers(content);
    let title = headers
        .get("title")
        .ok_or_else(|| ErrorKind::MissingHeader("title".to_string()))?
        .clone();

    let tags = headers
        .get("tags")
        .ok_or_else(|| ErrorKind::MissingHeader("title".to_string()))?
        .split_whitespace()
        .map(String::from)
        .collect::<Vec<String>>();

    let today = Utc::now();
    let date = NaiveDate::from_ymd(today.year(), today.month(), today.day()).and_hms(
        today.hour(),
        today.minute(),
        today.second(),
    );

    // Parse the markdown and retrieve the content
    let parsed_content = parse_content(content)?;

    Ok(ParsedPost {
        title,
        date,
        content: parsed_content,
        tags,
    })
}

fn is_header(line: &str) -> bool {
    lazy_static! {
        static ref IS_HEADER: Regex = Regex::new("^[a-z]+:").unwrap();
    }

    // A line is considered a header if it is empty, or the content matches the `IS_HEADER` regex
    line.trim().is_empty() || IS_HEADER.is_match(line)
}

fn parse_headers(content: &str) -> HashMap<String, String> {
    lazy_static! {
        static ref QUOTED_HEADER_VALUE: Regex = Regex::new("^([a-z]+):\\s+\"([^\"]*)\"").unwrap();
        static ref HEADER_VALUE: Regex = Regex::new("^([a-z]+):\\s+(.*)$").unwrap();
    }

    // Take all the lines which are headers, leaving the rest of the content
    let header_lines = content
        .lines()
        .take_while(|l| is_header(l))
        .map(std::string::ToString::to_string)
        .collect::<Vec<String>>();

    // Construct the headers hashmap
    let mut headers = HashMap::new();
    for line in &header_lines {
        if let Some(c) = QUOTED_HEADER_VALUE.captures(line) {
            let key = String::from(&c[1]);
            let value = String::from(&c[2]);
            headers.insert(key, value);
        } else if let Some(c) = HEADER_VALUE.captures(line) {
            let key = String::from(&c[1]);
            let value = String::from(&c[2]);
            headers.insert(key, value);
        }
    }

    headers
}

fn parse_content(content: &str) -> Result<String, ErrorKind> {
    // Collect all of the markdown content which isn't a header
    let markdown = content
        .lines()
        .filter(|l| !is_header(l))
        .collect::<Vec<_>>()
        .join("\n");

    // Load syntax highlighting information
    let ss = SyntaxSet::load_defaults_newlines();
    let mut syntax = String::from("py");
    let theme =
        &ThemeSet::get_theme("themes/base16-onedark.tmTheme").expect("Unable to parse theme file");

    // Set up parser
    let options = Options::all();
    let parser = Parser::new_ext(&markdown, options);

    let mut new_parser = Vec::new();
    let mut to_highlight = String::new();
    let mut in_codeblock = false;

    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(c)) => {
                match c {
                    CodeBlockKind::Indented => {}
                    CodeBlockKind::Fenced(c) => {
                        syntax = if c.is_empty() {
                            "default".to_string()
                        } else {
                            get_syntax_extension(&c[..])
                        }
                    }
                }

                in_codeblock = true;
            }
            Event::End(Tag::CodeBlock(_)) => {
                if in_codeblock {
                    if syntax == "default" {
                        let mut html = String::from("<pre><code>");
                        html.push_str(to_highlight.as_str());
                        html.push_str("</code></pre>");

                        new_parser.push(Event::Html(html.into()));
                    } else {
                        let code_syntax = ss.find_syntax_by_extension(&syntax).unwrap();
                        let html =
                            highlighted_html_for_string(&to_highlight, &ss, code_syntax, theme)
                                .map_err(|_| ErrorKind::SyntaxHighlightingError)?;

                        new_parser.push(Event::Html(html.into()));
                        to_highlight = String::new();

                        in_codeblock = false;
                    }
                }
            }
            Event::Text(t) => {
                if in_codeblock {
                    to_highlight.push_str(&t);
                } else {
                    new_parser.push(Event::Text(t));
                }
            }
            e => new_parser.push(e),
        }
    }

    let mut html_output = String::new();
    html::push_html(&mut html_output, new_parser.into_iter());
    Ok(html_output)
}

fn get_syntax_extension(name: &str) -> String {
    match name {
        "rust" | "rs" => "rs".to_string(),
        "python" | "py" => "py".to_string(),
        _ => "default".to_string(),
    }
}
