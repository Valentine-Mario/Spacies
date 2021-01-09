use crate::auth;
use crate::diesel::QueryDsl;
use crate::diesel::RunQueryDsl;
use crate::handlers::types::*;
use crate::model::{NewChannelUser, NewSpaceChannel, Space, SpaceChannel, SpaceUser, User};
use crate::schema::channel_users::dsl::channel_users;
use crate::schema::spaces::dsl::*;
use crate::schema::spaces_channel::dsl::space_id as channel_space_id;
use crate::schema::spaces_channel::dsl::*;
use crate::schema::spaces_users::dsl::space_id;
use crate::schema::spaces_users::dsl::*;
use crate::schema::users::dsl::*;
use crate::Pool;

use actix_web::{web, Error, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use diesel::dsl::{delete, insert_into};
use diesel::prelude::*;

//http respose
pub async fn create_new_channel(
    db: web::Data<Pool>,
    auth: BearerAuth,
    space_name: web::Path<PathInfo>,
    item: web::Json<ChannelField>,
) -> Result<HttpResponse, Error> {
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(web::block(move || {
                    create_new_channel_db(db, auth.token().to_string(), space_name, item)
                })
                .await
                .map(|response| HttpResponse::Ok().json(response))
                .map_err(|_| {
                    HttpResponse::Ok()
                        .json(Response::new(false, "Error creating channel".to_string()))
                })?)
            } else {
                Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
            }
        }
        Err(_) => Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))),
    }
}

pub async fn delete_channel(
    db: web::Data<Pool>,
    auth: BearerAuth,
    space_name: web::Path<ChannelPathInfo>,
) -> Result<HttpResponse, Error> {
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(
                    web::block(move || delete_channel_db(db, auth.token().to_string(), space_name))
                        .await
                        .map(|response| HttpResponse::Ok().json(response))
                        .map_err(|_| {
                            HttpResponse::Ok()
                                .json(Response::new(false, "Error deleting channel".to_string()))
                        })?,
                )
            } else {
                Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
            }
        }
        Err(_) => Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))),
    }
}

pub async fn get_channels_in_space(
    db: web::Data<Pool>,
    space_details: web::Path<PathInfo>,
) -> Result<HttpResponse, Error> {
    Ok(
        web::block(move || get_channels_in_space_db(db, space_details))
            .await
            .map(|response| HttpResponse::Ok().json(response))
            .map_err(|_| {
                HttpResponse::Ok().json(Response::new(false, "Error getting channel".to_string()))
            })?,
    )
}

pub async fn edit_channel_name(
    db: web::Data<Pool>,
    auth: BearerAuth,
    space_name: web::Path<ChannelPathInfo>,
    item: web::Json<ChannelField>,
) -> Result<HttpResponse, Error> {
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(web::block(move || {
                    edit_channel_name_db(db, auth.token().to_string(), space_name, item)
                })
                .await
                .map(|response| HttpResponse::Ok().json(response))
                .map_err(|_| {
                    HttpResponse::Ok()
                        .json(Response::new(false, "Error updating channel".to_string()))
                })?)
            } else {
                Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
            }
        }
        Err(_) => Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))),
    }
}

pub async fn get_channel_details(
    db: web::Data<Pool>,
    auth: BearerAuth,
    space_name: web::Path<ChannelPathInfo>,
) -> Result<HttpResponse, Error> {
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(web::block(move || {
                    get_channel_details_db(db, auth.token().to_string(), space_name)
                })
                .await
                .map(|response| HttpResponse::Ok().json(response))
                .map_err(|_| {
                    HttpResponse::Ok()
                        .json(Response::new(false, "Error getting channel".to_string()))
                })?)
            } else {
                Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
            }
        }
        Err(_) => Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))),
    }
}

//db calls
fn create_new_channel_db(
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
        .filter(user_id.eq(user.id))
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

fn delete_channel_db(
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
    let _spaces_user: SpaceUser = spaces_users
        .filter(space_id.eq(space.id))
        .filter(user_id.eq(user.id))
        .first::<SpaceUser>(&conn)?;
    let channel_details = spaces_channel
        .filter(channel_space_id.eq(&space.id))
        .filter(channel_name.ilike(&space_name.channel))
        .first::<SpaceChannel>(&conn)?;
    let _count = delete(spaces_channel.find(channel_details.id)).execute(&conn)?;
    Ok(Response::new(
        true,
        "channel deleted successfully".to_string(),
    ))
}

fn get_channels_in_space_db(
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

fn get_channel_details_db(
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
        .filter(user_id.eq(user.id))
        .first::<SpaceUser>(&conn)?;
    let channel_details = spaces_channel
        .filter(channel_space_id.eq(&space.id))
        .filter(channel_name.ilike(&space_name.channel))
        .first::<SpaceChannel>(&conn)?;
    Ok(Response::new(true, channel_details))
}

fn edit_channel_name_db(
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
        .filter(user_id.eq(user.id))
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
    if channel_details.channel_name == "General" {
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
