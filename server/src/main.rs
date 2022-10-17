#[macro_use]
extern crate rocket;

mod connection;
mod endpoints;

use crate::connection::init_pool;
use crate::endpoints::{get_post, index};
use rocket::fs::FileServer;

#[rocket::launch]
fn rocket() -> _ {
    rocket::build()
        .manage(init_pool())
        .mount("/", routes![index, get_post])
        .mount("/styles", FileServer::from("./styles"))
        .mount("/static", FileServer::from("./static"))
}
