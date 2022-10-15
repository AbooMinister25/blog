use rocket::fs::NamedFile;
use std::path::Path;

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
