use rocket::{fs::NamedFile, response::content::RawHtml, State};
use std::path::Path;
use tera::{Context, Tera};

#[get("/")]
pub async fn index() -> Option<NamedFile> {
    NamedFile::open(Path::new("./public/index.html")).await.ok()
}

#[get("/posts")]
pub async fn get_posts(tera: &State<Tera>) -> RawHtml<String> {
    let context = Context::default();
    let rendered = tera
        .render("posts.html.tera", &context)
        .expect("Error while rendering template");

    RawHtml(rendered)
}

#[get("/posts/<title>")]
pub async fn get_post(title: &str) -> Option<NamedFile> {
    NamedFile::open(Path::new("./public/").join(format!("{title}.html")))
        .await
        .ok()
}
