use crate::auth;
use crate::diesel::QueryDsl;
use crate::diesel::RunQueryDsl;
use crate::handlers::types::*;
use crate::model::{ChannelUser, NewChannelUser, Space, SpaceChannel, SpaceUser, User};
use crate::schema::channel_users::dsl::space_id as channel_user_space_id;
use crate::schema::channel_users::dsl::user_id as channel_user_user_id;
use crate::schema::channel_users::dsl::*;
use crate::schema::spaces::dsl::*;
use crate::schema::spaces_channel::dsl::space_id as channel_space_id;
use crate::schema::spaces_channel::dsl::*;
use crate::schema::spaces_users::dsl::space_id;
use crate::schema::spaces_users::dsl::user_id as spaces_user_id;
use crate::schema::spaces_users::dsl::*;
use crate::schema::users::dsl::*;
use crate::Pool;

use actix_web::{web, Error, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use diesel::dsl::{delete, insert_into};
use diesel::prelude::*;

//http calls
pub async fn add_user_to_channel(
    db: web::Data<Pool>,
    auth: BearerAuth,
    space_name: web::Path<ChannelPathInfo>,
    item: web::Json<AddUserToFoldr>,
)
-> Result<HttpResponse, Error>{
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(web::block(move || {
                    add_user_to_channel_db(db, auth.token().to_string(), space_name, item)
                })
                .await
                .map(|response| HttpResponse::Ok().json(response))
                .map_err(|_| {
                    HttpResponse::Ok()
                        .json(Response::new(false, "Error adding user to channel".to_string()))
                })?)
            } else {
                Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
            }
        }
        Err(_) => Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))),
    }
}

pub async fn remove_user_from_channel(
    db: web::Data<Pool>,
    auth: BearerAuth,
    space_name: web::Path<ChannelPathInfo>,
    item: web::Json<DeleteMailList>,
)->Result<HttpResponse, Error>{
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(web::block(move || {
                    remove_user_from_channel_db(db, auth.token().to_string(), space_name, item)
                })
                .await
                .map(|response| HttpResponse::Ok().json(response))
                .map_err(|_| {
                    HttpResponse::Ok()
                        .json(Response::new(false, "Error removing user from channel".to_string()))
                })?)
            } else {
                Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
            }
        }
        Err(_) => Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))),
    }
}

pub async fn change_user_admin_status(
    db: web::Data<Pool>,
    auth: BearerAuth,
    space_name: web::Path<ChannelPathInfo>,
    item: web::Json<DeleteMailList>,
)->Result<HttpResponse, Error>{
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(web::block(move || {
                    change_user_admin_status_db(db, auth.token().to_string(), space_name, item)
                })
                .await
                .map(|response| HttpResponse::Ok().json(response))
                .map_err(|_| {
                    HttpResponse::Ok()
                        .json(Response::new(false, "Error changing admin status".to_string()))
                })?)
            } else {
                Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
            }
        }
        Err(_) => Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))),
    }
}

pub async fn leave_channel(
    db: web::Data<Pool>,
    auth: BearerAuth,
    space_name: web::Path<ChannelPathInfo>,
)->Result<HttpResponse, Error>{
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(web::block(move || {
                    leave_channel_db(db, auth.token().to_string(), space_name)
                })
                .await
                .map(|response| HttpResponse::Ok().json(response))
                .map_err(|_| {
                    HttpResponse::Ok()
                        .json(Response::new(false, "Error leaving channel".to_string()))
                })?)
            } else {
                Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
            }
        }
        Err(_) => Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))),
    }
}

pub async fn get_channel_admin_status(
    db: web::Data<Pool>,
    auth: BearerAuth,
    space_name: web::Path<ChannelPathInfo>,
)->Result<HttpResponse, Error>{
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(web::block(move || {
                    get_channel_admin_status_db(db, auth.token().to_string(), space_name)
                })
                .await
                .map(|response| HttpResponse::Ok().json(response))
                .map_err(|_| {
                    HttpResponse::Ok()
                        .json(Response::new(false, "Error lgetting admin status".to_string()))
                })?)
            } else {
                Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
            }
        }
        Err(_) => Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))),
    }
}

pub async fn get_user_channel_in_space(
    db: web::Data<Pool>,
    auth: BearerAuth,
    space_name: web::Path<PathInfo>,
)->Result<HttpResponse, Error>{
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(web::block(move || {
                    get_user_channel_in_space_db(db, auth.token().to_string(), space_name)
                })
                .await
                .map(|response| HttpResponse::Ok().json(response))
                .map_err(|_| {
                    HttpResponse::Ok()
                        .json(Response::new(false, "Error getting channels".to_string()))
                })?)
            } else {
                Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
            }
        }
        Err(_) => Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))),
    }
}

pub async fn get_user_in_channel(
    db: web::Data<Pool>,
    auth: BearerAuth,
    space_name: web::Path<ChannelPathInfo>,
)->Result<HttpResponse, Error>{
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(web::block(move || {
                    get_user_in_channel_db(db, auth.token().to_string(), space_name)
                })
                .await
                .map(|response| HttpResponse::Ok().json(response))
                .map_err(|_| {
                    HttpResponse::Ok()
                        .json(Response::new(false, "Error getting users".to_string()))
                })?)
            } else {
                Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
            }
        }
        Err(_) => Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))),
    }
}

//db calls
fn add_user_to_channel_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<ChannelPathInfo>,
    item: web::Json<AddUserToFoldr>,
) -> Result<Response<String>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let decoded_token = auth::decode_token(&token);
    let user: User = users
        .find(decoded_token.parse::<i32>().unwrap())
        .first::<User>(&conn)?;
    let space: Space = spaces
        .filter(spaces_name.ilike(&space_name.info))
        .first::<Space>(&conn)?;
    let channel: SpaceChannel = spaces_channel
        .filter(channel_space_id.eq(space.id))
        .filter(channel_name.ilike(&space_name.channel))
        .first::<SpaceChannel>(&conn)?;
    //check if user is a channel admin
    let channel_user: ChannelUser = channel_users
        .filter(channel_user_space_id.eq(space.id))
        .filter(space_channel_id.eq(channel.id))
        .filter(channel_user_user_id.eq(user.id))
        .first::<ChannelUser>(&conn)?;

    if !channel_user.channel_admin {
        return Ok(Response::new(
            false,
            "only admin allowed to add users to channel".to_string(),
        ));
    }

    //loop through id vector
    for a in item.id.iter() {
        //if user is already in channel, do nothing
        let existing_channel_user = channel_users
            .filter(space_channel_id.eq(channel.id))
            .filter(channel_user_user_id.eq(a))
            .first::<ChannelUser>(&conn);

        match existing_channel_user {
            Ok(_user) => {
                //do nothing for existing user
            }
            Err(diesel::result::Error::NotFound) => {
                //add new user details to struct
                let new_channel_user = NewChannelUser {
                    space_channel_id: &channel.id,
                    space_id: &space.id,
                    user_id: &a,
                    channel_admin: &false,
                };
                //add user to new channel
                let _new_space_channel = insert_into(channel_users)
                    .values(&new_channel_user)
                    .execute(&conn)?;
            }
            _ => {
                //do nothing
            }
        }
    }

    Ok(Response::new(
        true,
        "new users added to channel successflly".to_string(),
    ))
}


fn remove_user_from_channel_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<ChannelPathInfo>,
    item: web::Json<DeleteMailList>,
) -> Result<Response<String>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let decoded_token = auth::decode_token(&token);
    let user: User = users
        .find(decoded_token.parse::<i32>().unwrap())
        .first::<User>(&conn)?;
    let space: Space = spaces
        .filter(spaces_name.ilike(&space_name.info))
        .first::<Space>(&conn)?;
    let channel: SpaceChannel = spaces_channel
        .filter(channel_space_id.eq(space.id))
        .filter(channel_name.ilike(&space_name.channel))
        .first::<SpaceChannel>(&conn)?;
    //check if user is a channel admin
    let channel_user: ChannelUser = channel_users
        .filter(channel_user_space_id.eq(space.id))
        .filter(space_channel_id.eq(channel.id))
        .filter(channel_user_user_id.eq(user.id))
        .first::<ChannelUser>(&conn)?;

    if !channel_user.channel_admin {
        return Ok(Response::new(
            false,
            "only admin allowed to kick out user from channel".to_string(),
        ));
    }
    let _count = delete(
        channel_users
            .filter(space_channel_id.eq(channel.id))
            .filter(channel_user_user_id.eq(item.id)),
    )
    .execute(&conn)?;

    Ok(Response::new(
        true,
        "user removed from channel successfully".to_string(),
    ))
}

fn change_user_admin_status_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<ChannelPathInfo>,
    item: web::Json<DeleteMailList>,
) -> Result<Response<String>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let decoded_token = auth::decode_token(&token);
    let user: User = users
        .find(decoded_token.parse::<i32>().unwrap())
        .first::<User>(&conn)?;
    let space: Space = spaces
        .filter(spaces_name.ilike(&space_name.info))
        .first::<Space>(&conn)?;
    let channel: SpaceChannel = spaces_channel
        .filter(channel_space_id.eq(space.id))
        .filter(channel_name.ilike(&space_name.channel))
        .first::<SpaceChannel>(&conn)?;
    //check if user is a channel admin
    let channel_user: ChannelUser = channel_users
        .filter(channel_user_space_id.eq(space.id))
        .filter(space_channel_id.eq(channel.id))
        .filter(channel_user_user_id.eq(user.id))
        .first::<ChannelUser>(&conn)?;

    if !channel_user.channel_admin {
        return Ok(Response::new(
            false,
            "only admin allowed to updated admin status".to_string(),
        ));
    }

    let channel_user2: ChannelUser = channel_users
        .filter(channel_user_space_id.eq(space.id))
        .filter(space_channel_id.eq(channel.id))
        .filter(channel_user_user_id.eq(user.id))
        .first::<ChannelUser>(&conn)?;
    let _channel_user_details_update = diesel::update(
        channel_users
            .filter(space_channel_id.eq(channel.id))
            .filter(channel_user_user_id.eq(item.id)),
    )
    .set(channel_admin.eq(&!channel_user2.channel_admin))
    .execute(&conn)?;
    Ok(Response::new(
        true,
        "Admin Status updated successfully".to_string(),
    ))
}

fn leave_channel_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<ChannelPathInfo>,
) -> Result<Response<String>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let decoded_token = auth::decode_token(&token);
    let user: User = users
        .find(decoded_token.parse::<i32>().unwrap())
        .first::<User>(&conn)?;
    let space: Space = spaces
        .filter(spaces_name.ilike(&space_name.info))
        .first::<Space>(&conn)?;
    let channel: SpaceChannel = spaces_channel
        .filter(channel_space_id.eq(space.id))
        .filter(channel_name.ilike(&space_name.channel))
        .first::<SpaceChannel>(&conn)?;
    let _count = delete(
        channel_users
            .filter(space_channel_id.eq(channel.id))
            .filter(channel_user_user_id.eq(user.id)),
    )
    .execute(&conn)?;
    Ok(Response::new(
        true,
        "Channel exited successfully".to_string(),
    ))
}

fn get_channel_admin_status_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<ChannelPathInfo>,
) -> Result<Response<bool>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let decoded_token = auth::decode_token(&token);
    let user: User = users
        .find(decoded_token.parse::<i32>().unwrap())
        .first::<User>(&conn)?;
    let space: Space = spaces
        .filter(spaces_name.ilike(&space_name.info))
        .first::<Space>(&conn)?;
    let channel: SpaceChannel = spaces_channel
        .filter(channel_space_id.eq(space.id))
        .filter(channel_name.ilike(&space_name.channel))
        .first::<SpaceChannel>(&conn)?;
    let channel_user: ChannelUser = channel_users
        .filter(channel_user_space_id.eq(space.id))
        .filter(space_channel_id.eq(channel.id))
        .filter(channel_user_user_id.eq(user.id))
        .first::<ChannelUser>(&conn)?;
    Ok(Response::new(true, channel_user.channel_admin))
}

fn get_user_channel_in_space_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<PathInfo>,
) -> Result<Response<Vec<(ChannelUser, SpaceChannel)>>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let decoded_token = auth::decode_token(&token);
    let user: User = users
        .find(decoded_token.parse::<i32>().unwrap())
        .first::<User>(&conn)?;
    let space: Space = spaces
        .filter(spaces_name.ilike(&space_name.info))
        .first::<Space>(&conn)?;

    let user_spaces: Vec<_> = ChannelUser::belonging_to(&user)
        .inner_join(spaces_channel)
        .filter(channel_user_space_id.eq(space.id))
        .load::<(ChannelUser, SpaceChannel)>(&conn)?;
    Ok(Response::new(true, user_spaces))
}

fn get_user_in_channel_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<ChannelPathInfo>,
) -> Result<Response<Vec<(ChannelUser, User)>>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let decoded_token = auth::decode_token(&token);
    let user: User = users
        .find(decoded_token.parse::<i32>().unwrap())
        .first::<User>(&conn)?;
    let space: Space = spaces
        .filter(spaces_name.ilike(&space_name.info))
        .first::<Space>(&conn)?;
    let _spaces_user: SpaceUser = spaces_users
        .filter(space_id.eq(space.id))
        .filter(spaces_user_id.eq(user.id))
        .first::<SpaceUser>(&conn)?;
    let channel: SpaceChannel = spaces_channel
        .filter(channel_space_id.eq(space.id))
        .filter(channel_name.ilike(&space_name.channel))
        .first::<SpaceChannel>(&conn)?;
    let user_spaces: Vec<_> = ChannelUser::belonging_to(&channel)
        .inner_join(users)
        .load::<(ChannelUser, User)>(&conn)?;
    Ok(Response::new(true, user_spaces))
}
