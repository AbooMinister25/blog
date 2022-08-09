use crate::util::response::ErrorKind;

use chrono::prelude::*;
use lazy_static::lazy_static;
use pulldown_cmark::{html, CodeBlockKind, Event, Options, Parser, Tag};
use regex::Regex;
use std::collections::HashMap;
use std::io::Lines;
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
}

pub fn parse(content: &str) -> Result<ParsedPost, ErrorKind> {
    // Parse the headers in the markdown
    let headers = parse_headers(content);
    let title = headers
        .get("title")
        .ok_or_else(|| ErrorKind::MissingHeader("title".to_string()))?
        .to_owned();

    let today = Utc::now();
    let date = NaiveDate::from_ymd(today.year(), today.month(), today.day()).and_hms(
        today.hour(),
        today.minute(),
        today.second(),
    );

    // Parse the markdown and retrieve the content
    let parsed_content = parse_content(content);
}

fn is_header(line: &str) -> bool {
    lazy_static! {
        static ref IS_HEADER: Regex = Regex::new("^[a-z]+:").unwrap();
    }

    // A line is considered a header if it is empty, or the content matches the `IS_HEADER` regex
    line.trim().is_empty() || IS_HEADER.is_match(line)
}

fn parse_headers<'a>(content: &str) -> HashMap<String, String> {
    lazy_static! {
        static ref QUOTED_HEADER_VALUE: Regex = Regex::new("^([a-z]+):\\s+\"([^\"]*)\"").unwrap();
        static ref HEADER_VALUE: Regex = Regex::new("^([a-z]+):\\s+(.*)$").unwrap();
    }

    let lines = content.lines();
    // Take all the lines which are headers, leaving the rest of the content
    let header_lines = content
        .lines()
        .take_while(|l| is_header(l))
        .map(|l| l.to_string())
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
    let mut markdown = content
        .lines()
        .filter(|l| !is_header(l))
        .collect::<Vec<_>>()
        .join("\n");
    
    let ss = SyntaxSet::load_defaults_newlines();
    let mut syntax = String::from("py");
    
}
