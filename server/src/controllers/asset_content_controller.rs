use crate::auth;
use crate::diesel::QueryDsl;
use crate::diesel::RunQueryDsl;
use crate::handlers::types::*;
use crate::model::{Asset, AssetContent, Space, SpaceUser, User};
use crate::schema::asset_contents::dsl::*;
use crate::schema::assets::dsl::space_id as asset_space_id;
use crate::schema::assets::dsl::*;
use crate::schema::spaces::dsl::*;
use crate::schema::spaces_users::dsl::space_id;
use crate::schema::spaces_users::dsl::*;
use crate::schema::users::dsl::*;
use crate::Pool;

use actix_web::web;
use diesel::dsl::delete;
use diesel::prelude::*;

pub fn get_files_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<MailChannelPathInfo>,
) -> Result<Response<(Asset, Vec<AssetContent>)>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let decoded_token = auth::decode_token(&token);
    let user = users
        .find(decoded_token.parse::<i32>().unwrap())
        .first::<User>(&conn)?;
    let space = spaces
        .filter(spaces_name.ilike(&space_name.info))
        .first::<Space>(&conn)?;
    let _spaces_user: SpaceUser = spaces_users
        .filter(space_id.eq(space.id))
        .filter(user_id.eq(user.id))
        .first::<SpaceUser>(&conn)?;
    let asset_folder: Asset = assets
        .filter(folder_name.ilike(&space_name.name))
        .filter(asset_space_id.eq(space.id))
        .first::<Asset>(&conn)?;
    let asset_conents: Vec<AssetContent> = asset_contents
        .filter(asset_id.eq(asset_folder.id))
        .load::<AssetContent>(&conn)?;

    Ok(Response::new(true, (asset_folder, asset_conents)))
}

pub fn delete_upload_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<AddUserToFolderPath>,
) -> Result<Response<String>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let decoded_token = auth::decode_token(&token);
    let user = users
        .find(decoded_token.parse::<i32>().unwrap())
        .first::<User>(&conn)?;
    let space = spaces
        .filter(spaces_name.ilike(&space_name.info))
        .first::<Space>(&conn)?;
    let _spaces_user: SpaceUser = spaces_users
        .filter(space_id.eq(space.id))
        .filter(user_id.eq(user.id))
        .first::<SpaceUser>(&conn)?;

    let _count = delete(asset_contents.find(space_name.id)).execute(&conn)?;
    Ok(Response::new(true, "File deleted successfully".to_string()))
}
