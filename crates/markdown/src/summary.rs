use std::cell::RefCell;

use color_eyre::Result;
use lol_html::{element, html_content::TextType, rewrite_str, text, RewriteStrSettings};

// Truncate the first part of a post's content for it's summary.
pub fn get_summary(content: &str) -> Result<String> {
    let character_count = RefCell::new(0);
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