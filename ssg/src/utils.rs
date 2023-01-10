use color_eyre::eyre::Result;
use lol_html::{element, html_content::TextType, rewrite_str, text, RewriteStrSettings};
use std::{cell::RefCell, fs, path::Path};

// If the given directory doesn't exist, creates it.
pub fn ensure_directory(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }

    Ok(())
}

// If the given file exists, delete it.
pub fn ensure_removed(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_file(path)?;
    }

    Ok(())
}

// Truncate the first part of a post's content for it's summary.
pub fn get_summary(content: &str) -> Result<String> {
    let character_count = RefCell::new(0);
    let mut summary = String::new();
    let mut skip = false;

    let element_content_handlers = vec![
        element!("*", |el| {
            if *character_count.borrow() > 150 {
                skip = true;
            }

            if skip {
                el.remove();
            }

            Ok(())
        }),
        text!("*", |text| {
            if matches!(text.text_type(), TextType::Data) {
                let text_str = text.as_str();
                let mut cc = character_count.borrow_mut();
                *cc += text_str.len();
                summary.push_str(text_str);
            }

            Ok(())
        }),
    ];

    let truncated = rewrite_str(
        content,
        RewriteStrSettings {
            element_content_handlers,
            ..RewriteStrSettings::default()
        },
    )?;

    Ok(truncated)
}
