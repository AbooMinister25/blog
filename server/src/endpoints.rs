use crate::connection::DbConn;
use crate::post::posts_from_database;
use rocket::{fs::NamedFile, response::content::RawHtml, State};
use std::path::Path;
use tera::{Context, Tera};

#[get("/")]
pub async fn index(tera: &State<Tera>, conn: DbConn) -> RawHtml<String> {
    let posts = posts_from_database(&conn);

    let mut context = Context::new();
    context.insert("posts", &posts);

    let rendered = tera
        .render("index.html.tera", &context)
        .expect("Error while rendering template");

    RawHtml(rendered)
}

#[get("/posts?<page>")]
pub async fn get_posts(tera: &State<Tera>, conn: DbConn, page: Option<i32>) -> RawHtml<String> {
    let posts = posts_from_database(&conn);

    let mut context = Context::default();
    context.insert("posts", &posts);
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
