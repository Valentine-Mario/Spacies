use crate::schema::*;
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Associations, PartialEq, Identifiable, Deserialize, Queryable)]
#[table_name = "users"]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub user_password: String,
    pub user_image: String,
    pub verified: bool,
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

#[derive(Debug, Serialize, Associations, PartialEq, Identifiable, Deserialize, Queryable)]
#[table_name = "spaces"]
pub struct Space {
    pub id: i32,
    pub spaces_name: String,
    pub spaces_desc: String,
    pub spaces_img: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[table_name = "spaces"]
pub struct NewSpace<'a> {
    pub spaces_name: &'a str,
    pub spaces_desc: &'a str,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Associations, PartialEq, Identifiable, Deserialize, Queryable)]
#[table_name = "spaces_users"]
#[belongs_to(User)]
#[belongs_to(Space)]
pub struct SpaceUser {
    pub id: i32,
    pub user_id: i32,
    pub space_id: i32,
    pub admin_status: bool,
}

#[derive(Insertable, Debug)]
#[table_name = "spaces_users"]
pub struct NewSpaceUser<'a> {
    pub user_id: &'a i32,
    pub space_id: &'a i32,
    pub admin_status: &'a bool,
}

#[derive(Debug, Serialize, Associations, PartialEq, Identifiable, Deserialize, Queryable)]
#[table_name = "spaces_channel"]
#[belongs_to(Space)]
pub struct SpaceChannel {
    pub id: i32,
    pub channel_name: String,
    pub space_id: i32,
}

#[derive(Insertable, Debug)]
#[table_name = "spaces_channel"]
pub struct NewSpaceChannel<'a> {
    pub space_id: &'a i32,
    pub channel_name: &'a str,
}

#[derive(Debug, Serialize, Associations, PartialEq, Identifiable, Deserialize, Queryable)]
#[table_name = "maillists"]
#[belongs_to(Space)]
pub struct MailList {
    pub id: i32,
    pub folder_name: String,
    pub space_id: i32,
}

#[derive(Insertable, Debug)]
#[table_name = "maillists"]
pub struct NewMailList<'a> {
    pub folder_name: &'a str,
    pub space_id: &'a i32,
}

#[derive(Debug, Serialize, Associations, PartialEq, Identifiable, Deserialize, Queryable)]
#[table_name = "usermails"]
#[belongs_to(MailList)]
#[belongs_to(User)]
pub struct UserMail {
    pub id: i32,
    pub mail_list_id: i32,
    pub user_id: i32,
}

#[derive(Insertable, Debug)]
#[table_name = "usermails"]
pub struct NewUserMail<'a> {
    pub mail_list_id: &'a i32,
    pub user_id: &'a i32,
}

#[derive(Debug, Serialize, Associations, PartialEq, Identifiable, Deserialize, Queryable)]
#[table_name = "assets"]
#[belongs_to(Space)]
pub struct Asset {
    pub id: i32,
    pub folder_name: String,
    pub space_id: i32,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[table_name = "assets"]
pub struct NewAsset<'a> {
    pub folder_name: &'a str,
    pub space_id: &'a i32,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Associations, PartialEq, Identifiable, Deserialize, Queryable)]
#[table_name = "asset_contents"]
#[belongs_to(Asset)]
pub struct AssetContent {
    pub id: i32,
    pub file_content: String,
    pub file_type: String,
    pub asset_id: i32,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[table_name = "asset_contents"]
pub struct NewAssetContent<'a> {
    pub file_content: &'a str,
    pub file_type: &'a str,
    pub asset_id: &'a i32,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Associations, PartialEq, Identifiable, Deserialize, Queryable)]
#[table_name = "events"]
#[belongs_to(Space)]
pub struct Event {
    pub id: i32,
    pub event_name: String,
    pub event_description: String,
    pub reminded: bool,
    pub event_date: chrono::NaiveDateTime,
    pub space_id: i32,
}

#[derive(Insertable, Debug)]
#[table_name = "events"]
pub struct NewEvent<'a> {
    pub event_name: &'a str,
    pub event_description: &'a str,
    pub reminded: &'a bool,
    pub event_date: chrono::NaiveDateTime,
    pub space_id: &'a i32,
}
