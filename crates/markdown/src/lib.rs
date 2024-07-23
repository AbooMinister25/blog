mod summary;

use std::{fmt::Debug, path::Path};

use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use color_eyre::Result;
use comrak::{
    format_html_with_plugins,
    nodes::{AstNode, NodeCode, NodeValue},
    parse_document,
    plugins::syntect::{SyntectAdapter, SyntectAdapterBuilder},
    Arena, ComrakOptions, ComrakPlugins,
};
use serde::{Deserialize, Serialize};
use syntect::{highlighting::ThemeSet, parsing::SyntaxSet};

/// The frontmatter for a parsed document
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Frontmatter {
    pub title: String,
    pub tags: Vec<String>,
    pub template: Option<String>,
    pub date: Option<String>,
    pub updated: Option<String>,
    pub series: Option<SeriesInfo>,
    pub slug: Option<String>,
    #[serde(default)]
    pub draft: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SeriesInfo {
    pub part: Option<i32>,
}

/// Used to parse and format a markdown document
#[derive(Debug)]
pub struct MarkdownRenderer<'c> {
    adapter: SyntectAdapter,
    options: ComrakOptions<'c>,
}

/// Represents a parsed markdown document
#[derive(Debug, Serialize, Clone)]
pub struct Document {
    pub date: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub content: String,
    pub frontmatter: Frontmatter,
    pub toc: Vec<String>,
    pub summary: String,
}

impl<'c> MarkdownRenderer<'c> {
    #[tracing::instrument]
    pub fn new<P: AsRef<Path> + Debug>(path: P, theme: &str) -> Result<Self> {
        // Load the theme set
        let ss = SyntaxSet::load_defaults_newlines();
        let mut theme_set = ThemeSet::load_defaults();
        theme_set.add_from_folder(path.as_ref().join("themes/"))?;

        // Create an adapter and choose the syntax highlighting theme.
        let adapter = SyntectAdapterBuilder::new()
            .syntax_set(ss)
            .theme_set(theme_set)
            .theme(theme)
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

    #[tracing::instrument(skip(self))]
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

        let date = frontmatter.date.as_ref().map_or(
            Ok::<DateTime<Utc>, color_eyre::Report>(Utc::now()),
            |d| {
                let parsed = d.parse::<NaiveDateTime>()?;
                Ok(Utc.from_utc_datetime(&parsed))
            },
        )?;

        let updated = frontmatter.updated.as_ref().map_or(
            Ok::<DateTime<Utc>, color_eyre::Report>(date),
            |d| {
                let parsed = d.parse::<NaiveDateTime>()?;
                Ok(Utc.from_utc_datetime(&parsed))
            },
        )?;

        Ok(Document {
            date,
            updated,
            summary: summary::get_summary(&string_html)?,
            content: string_html,
            frontmatter,
            toc,
        })
    }

    #[tracing::instrument(skip(self))]
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

    #[tracing::instrument(skip(self))]
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

    #[tracing::instrument(skip(self))]
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::create_dir;
    use tempfile::tempdir;

    fn get_renderer<'a>() -> Result<MarkdownRenderer<'a>> {
        let tmp_dir = tempdir()?;
        create_dir(tmp_dir.path().join("themes/"))?;
        let renderer = MarkdownRenderer::new(tmp_dir.path(), "Solarized (light)")?;

        Ok(renderer)
    }

    #[test]
    fn test_parse_markdown() -> Result<()> {
        let content = r#"---
title = "Test"
tags = ["a", "b", "c"]
---

Hello World
"#;
        let renderer = get_renderer()?;
        let document = renderer.render(content)?;

        assert_eq!(document.frontmatter.title, "Test".to_string());
        assert_eq!(
            document.frontmatter.tags,
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
        assert_eq!(document.content, "<p>Hello World</p>\n".to_string());

        Ok(())
    }

    #[test]
    fn test_get_summary() -> Result<()> {
        let content = r#"---
title = "Test"
tags = ["a", "b", "c"]
---

Lorem ipsum dolor sit amet, consectetur adipiscing elit. 
Suspendisse ut mattis felis. Mauris sed ex vitae est pharetra 
scelerisque. Ut ut sem arcu. Morbi molestie dictum venenatis. 
Quisque sit amet consequat libero. Cras id tellus diam. Cras 
pulvinar tristique nisl vel porttitor. Fusce enim magna, porta 
sed nisl non, dignissim ultrices massa. Sed ultrices tempus dolor sit 
amet fringilla. Proin at mauris porta, efficitur magna sit amet, 
rutrum elit. In efficitur vitae erat id scelerisque. Cras laoreet 
elit eu neque condimentum auctor. Lorem ipsum dolor sit amet, 
consectetur adipiscing elit. Vivamus nec auctor neque, at 
consectetur velit. Maecenas at massa ante.
        "#;
        let renderer = get_renderer()?;
        let document = renderer.render(content)?;

        assert_eq!(document.summary, "<p>Lorem ipsum dolor sit amet, consectetur adipiscing elit.\nSuspendisse ut mattis felis. Mauris sed ex vitae est pharetra\nscelerisque. Ut ut sem arcu. Morbi molestie dictum venenatis.\nQuisque sit amet consequat libero. Cras id tellus diam. Cras\npulvinar tristique nisl vel porttitor. Fusce enim magna, porta\nsed nisl non, dignissim ultrices massa. Sed ultrices tempus dolor sit\namet fringilla. Proin at mauris porta, efficitur magna sit amet,\nrutrum elit. In efficitur vitae erat id scelerisque. Cras laoreet\nelit eu neque condimentum auctor. Lorem ipsum dolor sit amet,\nconsectetur adipiscing elit. Vivamus nec auctor neque, at\nconsectetur velit. Maecenas at massa ante.</p>\n".to_string());

        Ok(())
    }

    #[test]
    fn test_parse_template() -> Result<()> {
        let content = r#"---
title = "Test"
tags = ["a", "b", "c"]
template = "index.html.tera"
---

Hello World
"#;
        let renderer = get_renderer()?;
        let document = renderer.render(content)?;

        assert_eq!(
            document.frontmatter.template,
            Some("index.html.tera".to_string())
        );
        Ok(())
    }

    #[test]
    fn test_parse_date() -> Result<()> {
        let content = r#"---
title = "Test"
tags = ["a", "b", "c"]
date = "2024-07-17T6:00:00"
---

Hello World
"#;
        let renderer = get_renderer()?;
        let document = renderer.render(content)?;

        assert_eq!(
            document.date,
            Utc.from_utc_datetime(&"2024-07-17T6:00:00".parse::<NaiveDateTime>()?)
        );
        Ok(())
    }

    #[test]
    fn test_parse_updated() -> Result<()> {
        let content = r#"---
title = "Test"
tags = ["a", "b", "c"]
date = "2024-07-12T6:00:00"
updated = "2024-07-17T6:00:00"
---

Hello World
"#;
        let renderer = get_renderer()?;
        let document = renderer.render(content)?;

        assert_eq!(
            document.updated,
            Utc.from_utc_datetime(&"2024-07-17T6:00:00".parse::<NaiveDateTime>()?)
        );
        Ok(())
    }

    #[test]
    fn test_parse_updated_default() -> Result<()> {
        let content = r#"---
title = "Test"
tags = ["a", "b", "c"]
date = "2024-07-12T6:00:00"
---

Hello World
"#;
        let renderer = get_renderer()?;
        let document = renderer.render(content)?;

        assert_eq!(
            document.updated,
            Utc.from_utc_datetime(&"2024-07-12T6:00:00".parse::<NaiveDateTime>()?)
        );
        Ok(())
    }

    #[test]
    fn test_parse_slug() -> Result<()> {
        let content = r#"---
title = "Test"
tags = ["a", "b", "c"]
slug = "test"
---

Hello World
"#;
        let renderer = get_renderer()?;
        let document = renderer.render(content)?;

        assert_eq!(document.frontmatter.slug, Some("test".to_string()));
        Ok(())
    }

    #[test]
    fn test_parse_draft() -> Result<()> {
        let content = r#"---
title = "Test"
tags = ["a", "b", "c"]
---

Hello World
"#;
        let renderer = get_renderer()?;
        let document = renderer.render(content)?;

        assert!(!document.frontmatter.draft);

        let content = r#"---
title = "Test"
tags = ["a", "b", "c"]
draft = true
---

Hello World
"#;
        let document = renderer.render(content)?;
        assert!(document.frontmatter.draft);
        Ok(())
    }

    #[test]
    fn test_parse_series() -> Result<()> {
        let content = r#"---
title = "Test"
tags = ["a", "b", "c"]

[series]
part = 1
---

Hello World
"#;
        let renderer = get_renderer()?;
        let document = renderer.render(content)?;

        assert_eq!(document.frontmatter.series.unwrap().part, Some(1));
        Ok(())
    }

    #[test]
    fn test_parse_toc() -> Result<()> {
        let content = r#"---
title = "Test"
tags = ["a", "b", "c"]
---

## First
hi

## Second
hi

## Third
hi
"#;
        let renderer = get_renderer()?;
        let document = renderer.render(content)?;

        assert_eq!(document.toc, vec!["First", "Second", "Third"]);
        Ok(())
    }
}
