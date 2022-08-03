#[macro_use]
extern crate rocket;

use blog::build_app;

#[launch]
fn rocket() -> _ {
    build_app()
}
