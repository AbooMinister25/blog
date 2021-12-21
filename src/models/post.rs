use crate::schema::posts;
use crate::DATE_FORMAT;
use chrono::NaiveDateTime;
use serde::Serialize;

#[derive(Queryable)]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub summary: String,
    pub published: bool,
    pub published_date: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "posts"]
pub struct NewPost {
    pub title: String,
    pub body: String,
    pub summary: String,
    pub published_date: NaiveDateTime,
    pub published: bool,
}


impl Post {
    pub fn to_json(self) -> PostJson {
        PostJson {
            id: self.id,
            title: self.title,
            body: self.body,
            published: self.published,
            published_date: self.published_date.format(DATE_FORMAT).to_string(),
        }
    }
}

#[derive(Serialize)]
pub struct PostJson {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub published: bool,
    pub published_date: String,
}
