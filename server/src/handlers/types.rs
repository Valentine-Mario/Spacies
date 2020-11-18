use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub email: String,
    pub user_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response<T> {
    pub success: bool,
    pub message: T,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryInfo {
    pub token: String,
}

impl<T> Response<T> {
    pub fn new(success: bool, message: T) -> Self {
        Self { success, message }
    }
}
