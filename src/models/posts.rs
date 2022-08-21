use crate::schema::posts;
use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable};
use serde::{Deserialize, Serialize};

#[derive(Queryable, Deserialize, Serialize)]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub summary: String,
    pub published: bool,
    pub published_at: NaiveDateTime,
    pub tags: Option<Vec<String>>,
}

#[derive(Insertable)]
#[table_name = "posts"]
pub struct NewPost {
    title: String,
    body: String,
    summary: String,
    published: bool,
    published_at: NaiveDateTime,
    tags: Option<Vec<String>>,
}

impl NewPost {
    pub fn new(
        title: String,
        body: String,
        summary: String,
        published: bool,
        published_at: NaiveDateTime,
        tags: Option<Vec<String>>,
    ) -> Self {
        Self {
            title,
            body,
            summary,
            published,
            published_at,
            tags,
        }
    }
}
