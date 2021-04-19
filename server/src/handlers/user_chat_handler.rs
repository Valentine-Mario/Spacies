use crate::auth;
use crate::handlers::types::*;
use crate::Pool;

use actix_web::{web, Error, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;

use crate::controllers::user_chat_controller::*;
use crate::diesel::QueryDsl;
use crate::diesel::RunQueryDsl;
use crate::helpers::socket::push_user_message;
use crate::model::{NewUserChat, User, UserChat};
use crate::schema::user_chat::dsl::*;
use crate::schema::users::dsl::*;

use diesel::dsl::insert_into;
use diesel::prelude::*;

pub async fn send_message(
    db: web::Data<Pool>,
    token: BearerAuth,
    other_user_id: web::Path<IdPathInfo>,
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
                match user {
                    Ok(user) => {
                        let new_chat = NewUserChat {
                            user_id: &user.id,
                            reciever: &other_user_id.id,
                            chat: &item.chat,
                            created_at: chrono::Local::now().naive_local(),
                        };
                        let socket_channel = format!("user-chat-{}-{}", &user.id, other_user_id.id);
                        let response = insert_into(user_chat).values(&new_chat).get_result(&conn);
                        match response {
                            Ok(response) => {
                                let socket_message = UserMessage {
                                    message: response,
                                    user: user,
                                };
                                push_user_message(
                                    &socket_channel,
                                    &"user_chat_created".to_string(),
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
    other_user_id: web::Path<MultiIdPathInfo>,
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
                match user {
                    Ok(user) => {
                        let updated_chat = diesel::update(user_chat.find(other_user_id.user_id))
                            .set(chat.eq(&item.chat))
                            .execute(&conn);
                        match updated_chat {
                            Ok(_updated_chat) => {
                                let message = user_chat
                                    .find(other_user_id.chat_id)
                                    .first::<UserChat>(&conn)
                                    .unwrap();
                                let socket_channel =
                                    format!("user-chat-{}-{}", &user.id, other_user_id.user_id);
                                let socket_message = UserMessage {
                                    message: message,
                                    user: user,
                                };
                                push_user_message(
                                    &socket_channel,
                                    &"user_chat_update".to_string(),
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

pub async fn get_all_message(
    db: web::Data<Pool>,
    auth: BearerAuth,
    other_user_id: web::Path<IdPathInfo>,
    item: web::Query<PaginateQuery>,
) -> Result<HttpResponse, Error> {
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(web::block(move || {
                    get_all_message_db(db, auth.token().to_string(), other_user_id, item)
                })
                .await
                .map(|response| HttpResponse::Ok().json(response))
                .map_err(|_| {
                    HttpResponse::Ok().json(Response::new(false, "Error getting chat".to_string()))
                })?)
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
