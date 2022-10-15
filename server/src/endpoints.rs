use rocket::response::content::RawHtml;
use std::include_str;

static INDEX_CONTENT: &str = include_str!("../../public/index.html");

#[get("/")]
pub async fn index() -> RawHtml<&'static str> {
    RawHtml(INDEX_CONTENT)
}
