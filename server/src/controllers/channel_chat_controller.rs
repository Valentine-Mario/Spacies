use crate::auth;
use crate::handlers::types::*;
use crate::Pool;

use actix_web::web;

use crate::diesel::QueryDsl;
use crate::diesel::RunQueryDsl;
use crate::handlers::paginate::*;
use crate::model::{ChannelChat, Space, SpaceChannel, User};
use crate::schema::channel_chats::dsl::id as channel_chat_id;
use crate::schema::channel_chats::dsl::user_id as channel_user_chat_id;
use crate::schema::channel_chats::dsl::*;
use crate::schema::spaces::dsl::*;
use crate::schema::spaces_channel::dsl::*;
use crate::schema::users::dsl::*;

use diesel::dsl::delete;
use diesel::prelude::*;

pub fn get_all_message_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<ChannelPathInfo>,
    item: web::Query<PaginateQuery>,
) -> Result<Response<(Vec<(ChannelChat, User)>, i64)>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let decoded_token = auth::decode_token(&token);
    let _user: User = users
        .find(&decoded_token.parse::<i32>().unwrap())
        .first::<User>(&conn)?;
    let space: Space = spaces
        .filter(spaces_name.ilike(&space_name.info))
        .first::<Space>(&conn)?;
    let channel: SpaceChannel = spaces_channel
        .filter(space_id.eq(&space.id))
        .filter(channel_name.ilike(&space_name.channel))
        .first::<SpaceChannel>(&conn)?;
    let all_chats: (Vec<(ChannelChat, User)>, i64) = ChannelChat::belonging_to(&channel)
        .inner_join(users)
        .order(channel_chat_id.desc())
        .paginate(item.page)
        .per_page(item.per_page)
        .load_and_count_pages::<(ChannelChat, User)>(&conn)?;

    Ok(Response::new(true, all_chats))
}

pub fn delete_message_db(
    db: web::Data<Pool>,
    token: String,
    channel_details: web::Path<IdPathInfo>,
) -> Result<Response<String>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let decoded_token = auth::decode_token(&token);
    let user: User = users
        .find(decoded_token.parse::<i32>().unwrap())
        .first::<User>(&conn)?;
    let _count2 = delete(
        channel_chats
            .filter(channel_chat_id.eq(channel_details.id))
            .filter(channel_user_chat_id.eq(user.id)),
    )
    .execute(&conn)?;

    Ok(Response::new(
        true,
        "message deleted successfully".to_string(),
    ))
}
