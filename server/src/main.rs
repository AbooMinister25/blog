#[macro_use]
extern crate rocket;

mod connection;
mod endpoints;
mod templates;
mod post;

use crate::connection::init_pool;
use crate::endpoints::{get_post, get_posts, index};
use rocket::{fs::FileServer, response::content::RawHtml, Request};
use tera::Tera;

#[catch(500)]
fn internal_server_error() -> RawHtml<String> {
    RawHtml(
        r#"
    <head>
        <title>Rayyan Cyclegar</title>
        <link rel="stylesheet" href="/styles/errors.css" />
    </head>
    <div class="error error-500">
        <h1>Internal Server Error</h1>
        <p>Something went wrong. Try again later, or go back <a href="/">Home</a></p>
    </div>
    "#
        .to_string(),
    )
}

#[catch(404)]
fn not_found(req: &Request<'_>) -> RawHtml<String> {
    RawHtml(format!(
        r#"
    <head>
        <title>Rayyan Cyclegar</title>
        <link rel="stylesheet" href="/styles/errors.css" />
    </head>
    <div class="error error-404">
        <h1>Page Not Found</h1>
        <p> The page <code>{}</code> doesn't exist </p>
        <p> Consider heading back <a href="/">Home</a> </p>
    </div>
        "#,
        req.uri()
    ))
}

#[rocket::launch]
fn rocket() -> _ {
    let tera = Tera::new("templates/**/*.tera").expect("Error while discovering templates");

    rocket::build()
        .manage(init_pool())
        .manage(tera)
        .mount("/", routes![index, get_post, get_posts])
        .mount("/styles", FileServer::from("./styles"))
        .mount("/static", FileServer::from("./static"))
        .register("/", catchers![internal_server_error, not_found])
}
