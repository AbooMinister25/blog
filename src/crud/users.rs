use crate::errors::ApiError;
use crate::models::user::{NewUser, User};
use crate::schema::users::dsl::*;
use argon2::{self, Config};
use diesel::prelude::*;
use rand::{thread_rng, Rng};

pub fn new(conn: &PgConnection, uname: &str, password: &str) -> Result<(), ApiError> {
    let hashed = hash_password(password)?;
    let new_user = NewUser {
        username: uname.to_string(),
        passwd: hashed,
    };

    diesel::insert_into(users)
        .values(&new_user)
        .execute(conn)
        .map_err(ApiError::UserInsertionError)?;

    Ok(())
}

pub fn validate(conn: &PgConnection, password: &str, uname: &str) -> Result<bool, ApiError> {
    let user = users
        .filter(username.eq(uname))
        .limit(1)
        .first::<User>(conn)
        .optional()
        .map_err(ApiError::UserLoadError)?;

    if let Some(u) = user {
        let hash = u.passwd;
        return argon2::verify_encoded(&hash, password.as_bytes())
            .map_err(ApiError::AuthenticationError);
    }

    Err(ApiError::UserNotFound)
} 

pub fn delete(conn: &PgConnection, uname: &str) -> Result<(), ApiError> {
    diesel::delete(users.filter(username.eq(uname)))
        .execute(conn)
        .map_err(ApiError::UserDeleteError)?;

    Ok(())
}

fn hash_password(password: &str) -> Result<String, ApiError> {
    let config = Config::default();
    let mut salt_arr: [u8; 16] = [0u8; 16];
    thread_rng().fill(&mut salt_arr[..]);

    argon2::hash_encoded(password.as_bytes(), &salt_arr, &config)
        .map_err(ApiError::PasswordHashingError)
}
