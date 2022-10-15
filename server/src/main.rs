#[macro_use]
extern crate rocket;

mod endpoints;

use crate::endpoints::index;
use rocket::fs::FileServer;

#[rocket::launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .mount("/styles", FileServer::from("./styles"))
        .mount("/static", FileServer::from("./static"))
}
