use std::{collections::HashSet, default::Default, fs, path::Path};

use crate::Page;

use color_eyre::Result;
use serde::{Deserialize, Serialize};

/// An index of all the pages in the site.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Index {
    pub working_pages: HashSet<Page>,
}

impl Index {
    pub fn build_index(&self, root: &Path) -> Result<()> {
        // TODO: Possibly change this to a HashMap and use the entry API instead.
        let path = root.join("index.json");
        let serialized = if path.exists() {
            let content = fs::read_to_string(&path)?;
            let mut old_index = Self {
                working_pages: serde_json::from_str(&content)?,
            };

            for page in &self.working_pages {
                old_index.working_pages.replace(page.clone());
            }

            serde_json::to_string(&old_index.working_pages)?
        } else {
            serde_json::to_string(&self.working_pages)?
        };

        fs::write(&path, serialized)?;

        Ok(())
    }

    pub fn clear(&mut self) {
        self.working_pages.clear();
    }
}

impl From<HashSet<Page>> for Index {
    fn from(value: HashSet<Page>) -> Self {
        Self {
            working_pages: value,
        }
    }
}
