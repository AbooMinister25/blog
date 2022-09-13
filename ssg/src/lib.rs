#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_panics_doc)]

#[macro_use]
extern crate diesel;

pub mod schema;
pub mod models;

use diesel::{Connection, SqliteConnection};
use dotenv::dotenv;
use rocket_sync_db_pools::database;
use std::env;

#[database("blog")]
pub struct DBPool(SqliteConnection);

pub const DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {database_url}"))
}
