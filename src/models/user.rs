use crate::schema::users;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub passwd: String,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub username: String,
    pub passwd: String,
}

#[derive(Deserialize)]
pub struct NewUserJson {
    pub username: String,
    pub passwd: String,
}
