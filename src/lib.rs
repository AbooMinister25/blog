#[macro_use]
extern crate diesel;

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate serde_json;

pub mod auth;
pub mod catchers;
pub mod crud;
pub mod endpoints;
pub mod errors;
pub mod markdown;
pub mod models;
pub mod response;
pub mod schema;

use diesel::pg::PgConnection;
use rocket::http::Method;
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};
use rocket_sync_db_pools::database;

#[database("blog_db")]
pub struct DBPool(PgConnection);

pub const DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

pub async fn app() -> rocket::Rocket<rocket::Build> {
    let allowed_origins = AllowedOrigins::all();

    let cors = CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get, Method::Post, Method::Put, Method::Delete]
            .into_iter()
            .map(From::from)
            .collect(),
        allowed_headers: AllowedHeaders::all(),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .expect("Failed building CORS fairing");

    rocket::build()
        .attach(DBPool::fairing())
        .attach(cors)
        .mount(
            "/api",
            routes![
                endpoints::posts::fetch_post,
                endpoints::posts::fetch_posts,
                endpoints::posts::create_post,
                endpoints::posts::delete_post,
                endpoints::users::create_user,
                endpoints::users::delete_user,
                endpoints::users::fetch_user,
                endpoints::users::validate_user,
            ],
        )
        .register(
            "/",
            catchers![catchers::not_found, catchers::internal_server_error],
        )
}
