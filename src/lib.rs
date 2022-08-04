#[macro_use]
extern crate rocket;

pub mod models;
pub mod schema;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use rocket::http::Status;
use std::env;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

#[get("/health_check")]
async fn health_check() -> Status {
    Status::Ok
}

pub fn build_app() -> rocket::Rocket<rocket::Build> {
    rocket::build().mount("/", routes![health_check])
}
