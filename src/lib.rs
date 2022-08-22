#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::must_use_candidate)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;

pub mod crud;
pub mod endpoints;
pub mod models;
pub mod schema;
pub mod util;

pub use endpoints::health_check;

use diesel::pg::PgConnection;
use dotenv::dotenv;
use rocket::figment::{
    util::map,
    value::{Map, Value},
};
use rocket_okapi::settings::UrlObject;
use rocket_okapi::{openapi_get_routes, rapidoc::*, swagger_ui::*};
use rocket_sync_db_pools::database;
use std::env;

#[database("blog_dev")]
pub struct DBPool(PgConnection);

pub const DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

pub fn build_app() -> rocket::Rocket<rocket::Build> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db: Map<_, Value> = map! {
        "url" => database_url.into(),
        "pool_size" => 10.into(),
    };

    let figment = rocket::Config::figment().merge(("databases", map!["blog_dev" =>  db]));

    rocket::custom(figment)
        .attach(DBPool::fairing())
        .mount("/api", openapi_get_routes![health_check::health_check])
        .mount(
            "/api/swagger-ui/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "../openapi.json".to_owned(),
                ..Default::default()
            }),
        )
        .mount(
            "/api/rapidoc/",
            make_rapidoc(&RapiDocConfig {
                general: GeneralConfig {
                    spec_urls: vec![UrlObject::new("General", "../openapi.json")],
                    ..Default::default()
                },
                hide_show: HideShowConfig {
                    allow_spec_url_load: false,
                    allow_spec_file_load: false,
                    ..Default::default()
                },
                ..Default::default()
            }),
        )
}
