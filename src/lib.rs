#[macro_use]
extern crate rocket;

use rocket::http::Status;

#[get("/health_check")]
async fn health_check() -> Status {
    Status::Ok
}

pub fn build_app() -> rocket::Rocket<rocket::Build> {
    rocket::build().mount("/", routes![health_check])
}
