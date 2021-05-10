use crate::auth;
use crate::diesel::QueryDsl;
use crate::diesel::RunQueryDsl;
use crate::handlers::types::*;
use crate::model::{
    ChannelUser, NewChannelUser, NewSpaceChannel, Space, SpaceChannel, SpaceUser, User,
};
use crate::schema::channel_users::dsl::channel_users;
use crate::schema::channel_users::dsl::space_channel_id;
use crate::schema::channel_users::dsl::space_id as channel_user_space_id;
use crate::schema::channel_users::dsl::user_id as channel_user_user_id;
use crate::schema::spaces::dsl::*;
use crate::schema::spaces_channel::dsl::space_id as channel_space_id;
use crate::schema::spaces_channel::dsl::*;
use crate::schema::spaces_users::dsl::space_id;
use crate::schema::spaces_users::dsl::user_id as spaces_user_id;
use crate::schema::spaces_users::dsl::*;
use crate::schema::users::dsl::*;
use crate::Pool;

use actix_web::web;
use diesel::dsl::{delete, insert_into};
use diesel::prelude::*;

pub fn create_new_channel_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<PathInfo>,
    item: web::Json<ChannelField>,
) -> Result<OptionalResponse<String, SpaceChannel>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let decoded_token = auth::decode_token(&token);
    let user = users
        .find(decoded_token.parse::<i32>().unwrap())
        .first::<User>(&conn)?;
    let space = spaces
        .filter(spaces_name.ilike(&space_name.info))
        .first::<Space>(&conn)?;
    let spaces_user: SpaceUser = spaces_users
        .filter(space_id.eq(space.id))
        .filter(spaces_user_id.eq(user.id))
        .first::<SpaceUser>(&conn)?;

    if !spaces_user.admin_status {
        return Ok(OptionalResponse::new(
            false,
            Some("Only admin is permitted to create new channel".to_string()),
            None,
        ));
    };
    //get all channels in space
    let channels: Vec<String> = spaces_channel
        .filter(channel_space_id.eq(&space.id))
        .select(channel_name)
        .load::<String>(&conn)?;
    if channels
        .iter()
        .any(|i| &i.to_lowercase() == &item.channel_name.to_lowercase())
    {
        return Ok(OptionalResponse::new(
            false,
            Some("Channel name already taken".to_string()),
            None,
        ));
    }
    //create channel
    let new_space_channel = NewSpaceChannel {
        space_id: &space.id,
        channel_name: &item.channel_name,
    };

    let space_channel: SpaceChannel = insert_into(spaces_channel)
        .values(&new_space_channel)
        .get_result(&conn)?;

    //add new user details to struct
    let new_channel_user = NewChannelUser {
        space_channel_id: &space_channel.id,
        space_id: &space.id,
        user_id: &user.id,
        channel_admin: &true,
        viewed: &0,
    };
    //add user to new channel as admin
    let _new_space_channel = insert_into(channel_users)
        .values(&new_channel_user)
        .execute(&conn)?;

    Ok(OptionalResponse::new(
        true,
        Some("New Channel created successfully".to_string()),
        Some(space_channel),
    ))
}

pub fn delete_channel_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<ChannelPathInfo>,
) -> Result<Response<String>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let decoded_token = auth::decode_token(&token);
    let user = users
        .find(decoded_token.parse::<i32>().unwrap())
        .first::<User>(&conn)?;
    let space = spaces
        .filter(spaces_name.ilike(&space_name.info))
        .first::<Space>(&conn)?;
    let spaces_user: SpaceUser = spaces_users
        .filter(space_id.eq(space.id))
        .filter(spaces_user_id.eq(user.id))
        .first::<SpaceUser>(&conn)?;
    let channel_details = spaces_channel
        .filter(channel_space_id.eq(&space.id))
        .filter(channel_name.ilike(&space_name.channel))
        .first::<SpaceChannel>(&conn)?;
    //check if user is a channel admin
    let channel_user: ChannelUser = channel_users
        .filter(channel_user_space_id.eq(space.id))
        .filter(space_channel_id.eq(channel_details.id))
        .filter(channel_user_user_id.eq(user.id))
        .first::<ChannelUser>(&conn)?;

    if !channel_user.channel_admin || !spaces_user.admin_status {
        return Ok(Response::new(
            false,
            "only admin allowed to delete channel".to_string(),
        ));
    }

    let _count2 =
        delete(channel_users.filter(space_channel_id.eq(channel_details.id))).execute(&conn)?;
    let _count = delete(spaces_channel.find(channel_details.id)).execute(&conn)?;
    Ok(Response::new(
        true,
        "channel deleted successfully".to_string(),
    ))
}

pub fn get_channels_in_space_db(
    db: web::Data<Pool>,
    space_details: web::Path<PathInfo>,
) -> Result<Response<Vec<SpaceChannel>>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let space = spaces
        .filter(spaces_name.ilike(&space_details.info))
        .first::<Space>(&conn)?;
    let channels = spaces_channel
        .filter(channel_space_id.eq(&space.id))
        .load::<SpaceChannel>(&conn)?;
    Ok(Response::new(true, channels))
}

pub fn get_channel_details_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<ChannelPathInfo>,
) -> Result<Response<SpaceChannel>, diesel::result::Error> {
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
        .filter(spaces_user_id.eq(user.id))
        .first::<SpaceUser>(&conn)?;
    let channel_details = spaces_channel
        .filter(channel_space_id.eq(&space.id))
        .filter(channel_name.ilike(&space_name.channel))
        .first::<SpaceChannel>(&conn)?;
    let _channel_user: ChannelUser = channel_users
        .filter(channel_user_space_id.eq(space.id))
        .filter(space_channel_id.eq(channel_details.id))
        .filter(channel_user_user_id.eq(user.id))
        .first::<ChannelUser>(&conn)?;
    Ok(Response::new(true, channel_details))
}

pub fn edit_channel_name_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<ChannelPathInfo>,
    item: web::Json<ChannelField>,
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
        .filter(spaces_user_id.eq(user.id))
        .first::<SpaceUser>(&conn)?;
    let channels: Vec<String> = spaces_channel
        .filter(channel_space_id.eq(&space.id))
        .select(channel_name)
        .load::<String>(&conn)?;
    if channels
        .iter()
        .any(|i| &i.to_lowercase() == &item.channel_name.to_lowercase())
    {
        return Ok(Response::new(
            false,
            "Channel name already taken".to_string(),
        ));
    }

    //get channel
    let channel_details = spaces_channel
        .filter(channel_space_id.eq(&space.id))
        .filter(channel_name.ilike(&space_name.channel))
        .first::<SpaceChannel>(&conn)?;
    if channel_details.channel_name.to_lowercase() == "general" {
        return Ok(Response::new(
            false,
            "General channel can't be modified".to_string(),
        ));
    }
    let _space_details = diesel::update(spaces_channel.find(channel_details.id))
        .set(channel_name.eq(&item.channel_name))
        .execute(&conn)?;
    Ok(Response::new(
        true,
        "Channel name updated successfully".to_string(),
    ))
}
