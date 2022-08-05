#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;

pub mod crud;
pub mod endpoints;
pub mod models;
pub mod schema;

pub use endpoints::health_check;

use diesel::pg::PgConnection;
use dotenv::dotenv;
use rocket::figment::{
    util::map,
    value::{Map, Value},
};
use rocket_sync_db_pools::database;
use std::env;

#[database("blog_dev")]
pub struct DBPool(PgConnection);

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
        .mount("/", routes![health_check::health_check])
}
