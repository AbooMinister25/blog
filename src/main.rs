#[macro_use]
extern crate rocket;

use rocket::http::Status;

#[get("/health_check")]
async fn health_check() -> Status {
    Status::Ok
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![health_check])
}
