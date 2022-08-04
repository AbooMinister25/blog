#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;

pub mod endpoints;
pub mod models;
pub mod schema;

pub use endpoints::health_check;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use std::env;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn build_app() -> rocket::Rocket<rocket::Build> {
    rocket::build().mount("/", routes![health_check::health_check])
}
