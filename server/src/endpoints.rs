use rocket::{fs::NamedFile, response::content::RawHtml};
use std::include_str;
use std::path::Path;

static INDEX_CONTENT: &str = include_str!("../../public/index.html");

#[get("/")]
pub async fn index() -> Option<NamedFile> {
    NamedFile::open(Path::new("./public/index.html")).await.ok()
}

#[get("/posts/<title>")]
pub async fn get_post(title: &str) -> Option<NamedFile> {
    NamedFile::open(Path::new("./public/").join(format!("{title}.html")))
        .await
        .ok()
}
