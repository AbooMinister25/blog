use color_eyre::Result;
use comrak::{
    plugins::syntect::{SyntectAdapter, SyntectAdapterBuilder},
    ComrakOptions,
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
}
