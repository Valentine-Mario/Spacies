use crate::auth;
use crate::handlers::types::*;
use crate::Pool;

use actix_web::web;

use crate::diesel::QueryDsl;
use crate::diesel::RunQueryDsl;
use crate::model::{ChannelChat, ChatThread, User};
use crate::schema::channel_chats::dsl::*;
use crate::schema::chat_thread::dsl::id as thread_chat_id;
use crate::schema::chat_thread::dsl::user_id as thread_user_id;
use crate::schema::chat_thread::dsl::*;
use crate::schema::users::dsl::*;

use diesel::dsl::delete;
use diesel::prelude::*;

pub fn get_chat_thread_db(
    db: web::Data<Pool>,
    token: String,
    chat_id: web::Path<IdPathInfo>,
) -> Result<Response<(ChannelChat, Vec<(ChatThread, User)>)>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let decoded_token = auth::decode_token(&token);
    let _user: User = users
        .find(decoded_token.parse::<i32>().unwrap())
        .first::<User>(&conn)?;
    let channel_chat: ChannelChat = channel_chats.find(chat_id.id).first::<ChannelChat>(&conn)?;

    let all_chats: Vec<(ChatThread, User)> = ChatThread::belonging_to(&channel_chat)
        .inner_join(users)
        .order(thread_chat_id.desc())
        .load::<(ChatThread, User)>(&conn)?;

    Ok(Response::new(true, (channel_chat, all_chats)))
}

pub fn delete_message_db(
    db: web::Data<Pool>,
    token: String,
    chat_id: web::Path<IdPathInfo>,
) -> Result<Response<String>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let decoded_token = auth::decode_token(&token);
    let user: User = users
        .find(decoded_token.parse::<i32>().unwrap())
        .first::<User>(&conn)?;
    let _count2 = delete(
        chat_thread
            .filter(thread_chat_id.eq(chat_id.id))
            .filter(thread_user_id.eq(user.id)),
    )
    .execute(&conn)?;

    Ok(Response::new(
        true,
        "message deleted successfully".to_string(),
    ))
}
