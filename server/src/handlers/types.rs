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

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginUser {
    pub email: String,
    pub user_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateName {
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePassword {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ForgotPassword {
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSpace {
    pub spaces_name: String,
    pub spaces_desc: String,
}

#[derive(Deserialize)]
pub struct PathInfo {
    pub info: String,
}
impl<T> Response<T> {
    pub fn new(success: bool, message: T) -> Self {
        Self { success, message }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseError {
    pub success: bool,
    pub error: String,
}

impl ResponseError {
    pub fn new(success: bool, error: String) -> Self {
        Self { success, error }
    }
}
