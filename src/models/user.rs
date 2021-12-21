use crate::schema::users;


#[derive(Queryable)]
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
