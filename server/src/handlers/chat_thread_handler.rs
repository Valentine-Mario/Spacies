use crate::auth;
use crate::handlers::types::*;
use crate::Pool;

use actix_web::{web, Error, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;

use crate::controllers::chat_thread_controller::*;
use crate::diesel::QueryDsl;
use crate::diesel::RunQueryDsl;
use crate::helpers::socket::push_thread_message;
use crate::model::{ChannelChat, ChatThread, NewChatThread, User};
use crate::schema::channel_chats::dsl::*;
use crate::schema::chat_thread::dsl::chat as thread_chat;
use crate::schema::chat_thread::dsl::*;
use crate::schema::users::dsl::*;

use diesel::dsl::insert_into;
use diesel::prelude::*;

pub async fn send_message(
    db: web::Data<Pool>,
    token: BearerAuth,
    chat_id: web::Path<IdPathInfo>,
    item: web::Json<ChatMessage>,
) -> Result<HttpResponse, Error> {
    match auth::validate_token(&token.token().to_string()) {
        Ok(res) => {
            if res == true {
                let conn = db.get().unwrap();
                let decoded_token = auth::decode_token(&token.token().to_string());
                let user = users
                    .find(decoded_token.parse::<i32>().unwrap())
                    .first::<User>(&conn);
                let channel_chat: ChannelChat = channel_chats
                    .find(chat_id.id)
                    .first::<ChannelChat>(&conn)
                    .unwrap();
                match user {
                    Ok(user) => {
                        let new_chat = NewChatThread {
                            user_id: &user.id,
                            space_channel_id: &channel_chat.space_channel_id,
                            channel_chat_id: &channel_chat.id,
                            chat: &item.chat,
                            created_at: chrono::Local::now().naive_local(),
                        };
                        //use chat id and channel_chat id as channel
                        let socket_channel = format!(
                            "thread-chat-{}-{}",
                            &channel_chat.id, channel_chat.space_channel_id
                        );
                        let response: Result<ChatThread, diesel::result::Error> =
                            insert_into(chat_thread).values(&new_chat).get_result(&conn);
                        match response {
                            Ok(response) => {
                                let socket_message = ThreadMessage {
                                    message: response,
                                    user: user,
                                };
                                push_thread_message(
                                    &socket_channel,
                                    &"thread_chat_created".to_string(),
                                    &socket_message,
                                )
                                .await;

                                Ok(HttpResponse::Ok().json(Response::new(
                                    true,
                                    "message sent successfully".to_string(),
                                )))
                            }
                            _ => Ok(HttpResponse::Ok()
                                .json(ResponseError::new(false, "error adding chat".to_string()))),
                        }
                    }
                    _ => Ok(HttpResponse::Ok().json(ResponseError::new(
                        false,
                        "error getting user details".to_string(),
                    ))),
                }
            } else {
                Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
            }
        }
        Err(_) => Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))),
    }
}

pub async fn update_message(
    db: web::Data<Pool>,
    token: BearerAuth,
    chat_id: web::Path<MultiId>,
    item: web::Json<ChatMessage>,
) -> Result<HttpResponse, Error> {
    match auth::validate_token(&token.token().to_string()) {
        Ok(res) => {
            if res == true {
                let conn = db.get().unwrap();
                let decoded_token = auth::decode_token(&token.token().to_string());
                let user = users
                    .find(decoded_token.parse::<i32>().unwrap())
                    .first::<User>(&conn);
                let channel_chat: ChannelChat = channel_chats
                    .find(chat_id.id2)
                    .first::<ChannelChat>(&conn)
                    .unwrap();
                match user {
                    Ok(user) => {
                        let updated_chat = diesel::update(chat_thread.find(chat_id.id))
                            .set(thread_chat.eq(&item.chat))
                            .execute(&conn);
                        match updated_chat {
                            Ok(_updated_chat) => {
                                let message: ChatThread = chat_thread
                                    .find(chat_id.id)
                                    .first::<ChatThread>(&conn)
                                    .unwrap();
                                let socket_channel = format!(
                                    "thread-chat-{}-{}",
                                    &channel_chat.id, channel_chat.space_channel_id
                                );
                                let socket_message = ThreadMessage {
                                    message: message,
                                    user: user,
                                };
                                push_thread_message(
                                    &socket_channel,
                                    &"thread_chat_update".to_string(),
                                    &socket_message,
                                )
                                .await;
                                Ok(HttpResponse::Ok().json(Response::new(
                                    true,
                                    "message updated successfully".to_string(),
                                )))
                            }
                            _ => Ok(HttpResponse::Ok()
                                .json(Response::new(false, "error updating chat".to_string()))),
                        }
                    }
                    _ => Ok(HttpResponse::Ok().json(ResponseError::new(
                        false,
                        "error getting user details".to_string(),
                    ))),
                }
            } else {
                Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
            }
        }
        Err(_) => Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))),
    }
}

pub async fn get_chat_thread(
    db: web::Data<Pool>,
    auth: BearerAuth,
    chat_id: web::Path<IdPathInfo>,
) -> Result<HttpResponse, Error> {
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(
                    web::block(move || get_chat_thread_db(db, auth.token().to_string(), chat_id))
                        .await
                        .map(|response| HttpResponse::Ok().json(response))
                        .map_err(|_| {
                            HttpResponse::Ok()
                                .json(Response::new(false, "Error getting chat".to_string()))
                        })?,
                )
            } else {
                Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
            }
        }
        Err(_) => Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))),
    }
}

pub async fn delete_message(
    db: web::Data<Pool>,
    auth: BearerAuth,
    chat_id: web::Path<IdPathInfo>,
) -> Result<HttpResponse, Error> {
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(
                    web::block(move || delete_message_db(db, auth.token().to_string(), chat_id))
                        .await
                        .map(|response| HttpResponse::Ok().json(response))
                        .map_err(|_| {
                            HttpResponse::Ok()
                                .json(Response::new(false, "Error deleting chat".to_string()))
                        })?,
                )
            } else {
                Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
            }
        }
        Err(_) => Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))),
    }
}
