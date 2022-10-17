use r2d2_sqlite::SqliteConnectionManager;
use rocket::{
    http::Status,
    outcome::Outcome,
    request::{self, FromRequest, Request},
};
use rusqlite::Connection;
use std::ops::Deref;

type Pool = r2d2::Pool<SqliteConnectionManager>;

pub fn init_pool() -> Pool {
    let manager = SqliteConnectionManager::file("./blog.db");
    r2d2::Pool::new(manager).expect("Problem while creating database pool")
}

pub struct DbConn(r2d2::PooledConnection<SqliteConnectionManager>);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for DbConn {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let pool = req.rocket().state::<Pool>();

        match pool {
            Some(state) => match state.get() {
                Ok(conn) => Outcome::Success(DbConn(conn)),
                Err(_) => Outcome::Failure((Status::ServiceUnavailable, ())),
            },
            None => Outcome::Failure((Status::InternalServerError, ())),
        }
    }
}

impl Deref for DbConn {
    type Target = Connection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
