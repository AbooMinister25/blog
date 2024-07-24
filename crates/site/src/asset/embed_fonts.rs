use std::{collections::HashMap, fmt::Debug, fs::File, io::Read, path::Path, sync::Mutex};

use base64::prelude::*;
use color_eyre::Result;
use lazy_static::lazy_static;
use regex::Regex;
use xmltree::{Element, XMLNode};

#[derive(Debug)]
struct Cacher {
    cache: HashMap<String, String>,
}

impl Cacher {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn fetch_font(&mut self, url: &str) -> Result<String> {
        if let Some(content) = self.cache.get(url) {
            Ok(content.to_string())
        } else {
            let resp = reqwest::blocking::get(url)?.bytes()?;
            let encoded = BASE64_STANDARD.encode(resp);

            self.cache.insert(url.to_string(), encoded);
            Ok(self.cache.get(url).unwrap().to_string())
        }
    }
}

lazy_static! {
    static ref URL_RE: Regex =
        Regex::new(r"https://(www\.)?[a-zA-Z]+.[a-z]+/([a-zA-Z]+).[a-z\d]+").unwrap();
    static ref CACHE: Mutex<Cacher> = Mutex::new(Cacher::new());
}

#[tracing::instrument(level = tracing::Level::DEBUG)]
pub fn embed_font<P: AsRef<Path> + Debug>(path: P) -> Result<()> {
    let mut file = File::open(&path)?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let mut root = Element::parse(contents.as_bytes())?;

    let style = root
        .get_mut_child("defs")
        .and_then(|d| d.get_mut_child("style"));

    if let Some(s) = style {
        let text = s.get_text();

        if let Some(t) = text {
            if let Some(mat) = URL_RE.find(&t) {
                let url = mat.as_str();
                let font = CACHE.lock().unwrap().fetch_font(url)?;
                let replacement = format!(
                    "
                @font-face {{
                    font-family: \"Virgil\";
                    src: url(data:font/font;base64,{font});
                }}
                "
                );

                s.children.push(XMLNode::Text(replacement));
            }
        }
    }

    root.write(File::create(path)?)?;

    Ok(())
}
