use chrono::prelude::*;
use color_eyre::eyre::{eyre, Result};
use pulldown_cmark::{html, CodeBlockKind, Event, Options, Parser, Tag};
use std::collections::HashMap;
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

/// A parsed blog post.
/// Contains the content parsed from a markdown document.
#[derive(Debug)]
pub struct ParsedPost {
    pub date: NaiveDateTime,
    pub content: String,
    pub headers: HashMap<String, HeaderValue>,
}

#[derive(Debug)]
pub enum HeaderValue {
    Single(String),
    List(Vec<String>),
    Boolean(bool),
}

impl From<&str> for HeaderValue {
    fn from(s: &str) -> Self {
        match s {
            "true" => Self::Boolean(true),
            "false" => Self::Boolean(false),
            s if s.starts_with('[') => {
                let mut splitted = s.split(',');
                splitted.next(); // Consume opening bracket

                let mut values = splitted
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<String>>();

                if let Some(s) = values.last_mut() {
                    *s = s.replace(']', "");
                } // Replace the closing bracket from the last string.

                Self::List(values)
            }
            s => Self::Single(s.to_string()),
        }
    }
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
#[tracing::instrument]
pub fn parse(content: &str) -> Result<ParsedPost> {
    // Parse the headers in the markdown
    let headers = parse_headers(content)?;
    let today = Utc::now();
    let date = NaiveDate::from_ymd(today.year(), today.month(), today.day()).and_hms(
        today.hour(),
        today.minute(),
        today.second(),
    );

    // Parse the markdown and retrieve the content
    let parsed_content = parse_content(content)?;

    Ok(ParsedPost {
        date,
        content: parsed_content,
        headers,
    })
}

fn parse_headers(content: &str) -> Result<HashMap<String, HeaderValue>> {
    let mut headers = HashMap::new();

    let lines = content
        .lines()
        .take_while(|l| !l.contains("<!-- End Headers -->") && !l.trim().is_empty())
        .map(std::string::ToString::to_string)
        .collect::<Vec<String>>();

    for line in lines {
        let mut splitted = line.split('=');
        let name = splitted
            .next()
            .ok_or_else(|| eyre!("Missing key for a header"))?
            .trim();

        let value = splitted
            .next()
            .ok_or_else(|| eyre!("Missing value for header {name}"))?
            .trim();

        let header_value = HeaderValue::from(value);

        headers.insert(name.to_string(), header_value);
    }

    Ok(headers)
}

fn parse_content(content: &str) -> Result<String> {
    // Collect all of the markdown content which isn't a header
    let markdown = content
        .split("<!-- End Headers -->")
        .last()
        .ok_or_else(|| eyre!("No content after headers"))?;

    // Load syntax highlighting information
    let ss = SyntaxSet::load_defaults_newlines();
    let mut syntax = String::from("py");
    let theme =
        &ThemeSet::get_theme("themes/base16-onedark.tmTheme").expect("Unable to parse theme file");

    // Set up parser
    let options = Options::all();
    let parser = Parser::new_ext(markdown, options);

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
                            highlighted_html_for_string(&to_highlight, &ss, code_syntax, theme)?;

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
