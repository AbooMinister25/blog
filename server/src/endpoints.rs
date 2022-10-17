use rocket::fs::NamedFile;
use rocket_dyn_templates::{context, Template};
use std::path::Path;
use crate::connection::DbConn;

#[get("/")]
pub async fn index(conn: DbConn) -> Option<NamedFile> {
    NamedFile::open(Path::new("./public/index.html")).await.ok()
}

#[get("/posts")]
pub async fn get_posts() -> Template {
    Template::render("posts", context! {
        
    })
}

#[get("/posts/<title>")]
pub async fn get_post(title: &str) -> Option<NamedFile> {
    NamedFile::open(Path::new("./public/").join(format!("{title}.html")))
        .await
        .ok()
}
