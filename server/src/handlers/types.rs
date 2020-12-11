use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub email: String,
    pub user_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryInfo {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginateQuery {
    pub page: i64,
    pub per_page: i64,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct InviteToSpace {
    pub email: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserIdStruct {
    pub user: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriviledgeStruct {
    pub admin: bool,
    pub user: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelField {
    pub channel_name: String,
}

#[derive(Deserialize)]
pub struct ChannelPathInfo {
    pub info: String,
    pub channel: String,
}

#[derive(Deserialize)]
pub struct MailChannelPathInfo {
    pub info: String,
    pub name: String,
}

#[derive(Deserialize)]
pub struct IdPathInfo {
    pub id: i32,
}

#[derive(Deserialize)]
pub struct AddUserToFolderPath {
    pub info: String,
    pub id: i32,
}

#[derive(Deserialize)]
pub struct AddFileContent {
    pub info: String,
    pub id: i32,
    pub file_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateMailList {
    pub folder_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteMailList {
    pub id: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddUserToFoldr {
    pub id: Vec<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendMail {
    pub title: String,
    pub body: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddAssetFolder {
    pub folder_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddEvent {
    pub event_name: String,
    pub event_description: String,
    pub year: i32,
    pub event_date: Vec<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EditEvent {
    pub event_name: String,
    pub event_description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddProject {
    pub project_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response<T> {
    pub success: bool,
    pub message: T,
}

impl<T> Response<T> {
    pub fn new(success: bool, message: T) -> Self {
        Self { success, message }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OptionalResponse<T, U> {
    success: bool,
    message: Option<T>,
    value: Option<U>,
}

impl<T, U> OptionalResponse<T, U> {
    pub fn new(success: bool, message: Option<T>, value: Option<U>) -> Self {
        OptionalResponse {
            success,
            message,
            value,
        }
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
