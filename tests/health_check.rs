#[macro_use]
extern crate rocket;

use rocket::http::Status;
use rocket::local::blocking::Client;

#[test]
fn health_check() {
    let client = Client::tracked(blog::build_app()).expect("valid rocket instance");
    let response = client.get(uri!(blog::health_check)).dispatch();
    assert_eq!(response.status(), Status::Ok);
}
