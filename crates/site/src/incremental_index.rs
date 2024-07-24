use std::{collections::HashSet, default::Default, fs};

use color_eyre::Result;
use serde::{Deserialize, Serialize};

use crate::{context::Context, page::Page};

/// An incrementally updated index of all the pages in the site.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct IncrementalIndex {
    pub posts: HashSet<Page>,
}

impl IncrementalIndex {
    pub fn build_index(&mut self, ctx: &Context) -> Result<()> {
        // TODO: Possibly change this to a HashMap and use the entry API instead.
        let path = ctx.config.original_output_path.join("index.json");
        let out_path = ctx.config.output_path.join("index.json");

        let serialized = if path.exists() {
            let content = fs::read_to_string(&path)?;
            let mut old_index = Self {
                posts: serde_json::from_str(&content)?,
            };

            for page in &self.posts {
                old_index.posts.replace(page.clone());
            }

            let ret = serde_json::to_string(&old_index.posts)?;
            self.posts = old_index.posts;

            ret
        } else {
            serde_json::to_string(&self.posts)?
        };

        fs::write(out_path, serialized)?;

        Ok(())
    }
}

impl From<Vec<Page>> for IncrementalIndex {
    fn from(value: Vec<Page>) -> Self {
        Self {
            posts: HashSet::from_iter(value),
        }
    }
}
