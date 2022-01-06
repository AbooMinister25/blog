use crate::errors::ApiError;
use chrono::prelude::*;
use lazy_static::lazy_static;
use pulldown_cmark::{html, CodeBlockKind, Event, Options, Parser, Tag};
use regex::Regex;
use std::collections::HashMap;
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

#[derive(Debug)]
pub struct ParsedPost {
    pub title: String,
    pub date: NaiveDateTime,
    pub content: String,
}

pub fn parse(content: &str) -> Result<ParsedPost, ApiError> {
    let lines = content.lines().collect::<Vec<_>>();
    let headers = parse_headers(&lines);

    let title = headers
        .get("title")
        .ok_or_else(|| ApiError::MissingHeader("title".to_string()))?
        .to_owned();

    let today = Utc::now();
    let date: NaiveDateTime = NaiveDate::from_ymd(today.year(), today.month(), today.day())
        .and_hms(today.hour(), today.minute(), today.second());

    let content = parse_content(&lines)?;

    Ok(ParsedPost {
        title,
        date,
        content,
    })
}

fn parse_content(lines: &[&str]) -> Result<String, ApiError> {
    let mut markdown = String::new();

    for line in lines {
        if !is_header(line) {
            markdown.push_str(line);
            markdown.push('\n');
        }
    }

    let ss = SyntaxSet::load_defaults_newlines();
    // let ts = ThemeSet::load_defaults();
    let mut syntax = String::from("py");
    let theme =
        &ThemeSet::get_theme("themes/base16-onedark.tmTheme").expect("Unable to parse theme file");

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);

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
                        if c.is_empty() {
                            syntax = String::from("default");
                        } else {
                            syntax = get_syntax_extension(&c[..])
                        }
                    }
                }
                in_codeblock = true;
            }
            Event::End(Tag::CodeBlock(_)) => {
                if in_codeblock {
                    if syntax == "default" {
                        let mut html = String::from("<pre><code>");
                        html.push_str(&to_highlight);
                        html.push_str("</code></pre>");

                        new_parser.push(Event::Html(html.into()));
                    } else {
                        let code_syntax = ss.find_syntax_by_extension(&syntax).unwrap();

                        let html =
                            highlighted_html_for_string(&to_highlight, &ss, code_syntax, theme);

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

fn parse_headers(lines: &[&str]) -> HashMap<String, String> {
    lazy_static! {
        static ref QUOTED_HEADER_VALUE: Regex = Regex::new("^([a-z]+):\\s+\"([^\"]*)\"").unwrap();
        static ref HEADER_VALUE: Regex = Regex::new("^([a-z]+):\\s+(.*)$").unwrap();
    }

    let header_lines: Vec<String> = lines
        .iter()
        .take_while(|l| is_header(l))
        .map(|l| l.to_string())
        .collect();

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

fn is_header(line: &str) -> bool {
    lazy_static! {
        static ref IS_HEADER: Regex = Regex::new("^[a-z]+:").unwrap();
    }

    line.trim().is_empty() || IS_HEADER.is_match(line)
}

fn get_syntax_extension(name: &str) -> String {
    match name {
        "rust" | "rs" => String::from("rs"),
        "python" | "py" => String::from("py"),
        _ => String::from("default"),
    }
}
