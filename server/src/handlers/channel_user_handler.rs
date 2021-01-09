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
use crate::schema::spaces_users::dsl::*;
use crate::schema::users::dsl::*;
use crate::Pool;

use actix_web::{web, Error, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use diesel::dsl::{delete, insert_into};
use diesel::prelude::*;

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
        .first::<ChannelUser>(&conn)?;

    if !channel_user.channel_admin {
        return Ok(Response::new(
            false,
            "only admin allowed to add users to channel".to_string(),
        ));
    }

    //loop through all user list
    for a in item.id.iter() {
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

    Ok(Response::new(
        true,
        "new users added to channel successflly".to_string(),
    ))
}

fn remove_user_from_channel_db() {}

fn make_user_channel_admin_db() {}

fn remove_user_channel_admin_db() {}

fn get_user_channel_in_space_db() {}

fn leave_channel_db() {}

fn get_channel_admin_status() {}
