use crate::auth;
use crate::diesel::QueryDsl;
use crate::diesel::RunQueryDsl;
use crate::handlers::paginate::*;
use crate::handlers::types::*;
use crate::helpers::socket::push_user_message;
use crate::model::{NewUserChat, User, UserChat};
use crate::schema::user_chat::dsl::id as user_chat_id;
use crate::schema::user_chat::dsl::user_id as sender_id;
use crate::schema::user_chat::dsl::*;
use crate::schema::users::dsl::id as user_id;
use crate::schema::users::dsl::*;
use crate::Pool;

use actix_web::web;
use diesel::dsl::{delete, insert_into};
use diesel::prelude::*;

pub async fn send_message_db(
    db: web::Data<Pool>,
    token: String,
    other_user_id: web::Path<IdPathInfo>,
    item: web::Json<ChatMessage>,
) -> Result<Response<String>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let decoded_token = auth::decode_token(&token);
    let user: User = users
        .find(decoded_token.parse::<i32>().unwrap())
        .first::<User>(&conn)?;
    let new_chat = NewUserChat {
        user_id: &user.id,
        reciever: &other_user_id.id,
        chat: &item.chat,
        created_at: chrono::Local::now().naive_local(),
    };
    let socket_channel = format!("{}-{}", &user.id, other_user_id.id);
    let res: UserChat = insert_into(user_chat).values(&new_chat).get_result(&conn)?;
    let socket_message = UserMessage {
        message: res,
        user: user,
    };
    push_user_message(
        &socket_channel,
        &"chat_created".to_string(),
        &socket_message,
    )
    .await;

    Ok(Response::new(true, "message sent successfully".to_string()))
}

pub fn get_all_message_db(
    db: web::Data<Pool>,
    token: String,
    other_user_id: web::Path<IdPathInfo>,
    item: web::Query<PaginateQuery>,
) -> Result<Response<(Vec<(UserChat, User)>, i64)>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let decoded_token = auth::decode_token(&token);
    let user: User = users
        .filter(user_id.eq_any(vec![
            &decoded_token.parse::<i32>().unwrap(),
            &other_user_id.id,
        ]))
        .first::<User>(&conn)?;

    let all_chats: (Vec<(UserChat, User)>, i64) = UserChat::belonging_to(&user)
        .inner_join(users)
        .filter(user_id.eq_any(vec![&user.id, &other_user_id.id]))
        .filter(reciever.eq_any(vec![&user.id, &other_user_id.id]))
        .order(user_chat_id.desc())
        .paginate(item.page)
        .per_page(item.per_page)
        .load_and_count_pages::<(UserChat, User)>(&conn)?;

    Ok(Response::new(true, all_chats))
}

pub async fn update_message_db(
    db: web::Data<Pool>,
    token: String,
    other_user_id: web::Path<MultiIdPathInfo>,
    item: web::Json<ChatMessage>,
) -> Result<Response<String>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let decoded_token = auth::decode_token(&token);
    let user: User = users
        .find(decoded_token.parse::<i32>().unwrap())
        .first::<User>(&conn)?;
    diesel::update(user_chat.find(other_user_id.user_id))
        .set((chat.eq(&item.chat),))
        .execute(&conn)?;
    let message = user_chat
        .find(other_user_id.chat_id)
        .first::<UserChat>(&conn)?;
    let socket_channel = format!("{}-{}", &user.id, other_user_id.user_id);
    let socket_message = UserMessage {
        message: message,
        user: user,
    };
    push_user_message(&socket_channel, &"chat_update".to_string(), &socket_message).await;

    Ok(Response::new(true, "update successful".to_string()))
}

pub fn delete_message_db(
    db: web::Data<Pool>,
    token: String,
    other_user_id: web::Path<MultiIdPathInfo>,
) -> Result<Response<String>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let decoded_token = auth::decode_token(&token);
    let user: User = users
        .find(decoded_token.parse::<i32>().unwrap())
        .first::<User>(&conn)?;
    let _count2 = delete(
        user_chat
            .filter(user_chat_id.eq(other_user_id.chat_id))
            .filter(sender_id.eq(user.id)),
    )
    .execute(&conn)?;

    Ok(Response::new(
        true,
        "message deleted successfully".to_string(),
    ))
}