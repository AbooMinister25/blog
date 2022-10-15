#[macro_use]
extern crate rocket;

mod endpoints;

use crate::endpoints::{get_post, index};
use rocket::fs::FileServer;

#[rocket::launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, get_post])
        .mount("/styles", FileServer::from("./styles"))
        .mount("/static", FileServer::from("./static"))
}
