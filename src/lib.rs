#[macro_use]
extern crate diesel;

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate serde_json;

pub mod crud;
pub mod endpoints;
pub mod errors;
pub mod markdown;
pub mod models;
pub mod response;
pub mod schema;

use diesel::pg::PgConnection;
use rocket_sync_db_pools::database;

#[database("blog_db")]
pub struct DBPool(PgConnection);

pub const DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

pub async fn app() -> rocket::Rocket<rocket::Build> {
    rocket::build().attach(DBPool::fairing()).mount(
        "/",
        routes![
            endpoints::posts::fetch_post,
            endpoints::posts::fetch_posts,
            endpoints::posts::create_post,
            endpoints::posts::delete_post,
        ],
    )
}
