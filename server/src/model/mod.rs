
use crate::schema::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Identifiable, Deserialize, Queryable)]
#[table_name = "users"]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub user_password: String,
    pub user_image:String,
    pub verified:bool,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub email: &'a str,
    pub user_password: &'a str,
    pub verified: &'a bool,
    pub created_at: chrono::NaiveDateTime,
}