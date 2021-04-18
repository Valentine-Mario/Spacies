use crate::auth;
use crate::handlers::types::*;
use crate::Pool;

use actix_web::{web, Error, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;

use crate::controllers::channel_chat_controller::*;
use crate::diesel::QueryDsl;
use crate::diesel::RunQueryDsl;
use crate::helpers::socket::push_channel_message;
use crate::model::{ChannelChat, NewChannelChat, Space, SpaceChannel, User};
use crate::schema::channel_chats::dsl::*;
use crate::schema::spaces::dsl::*;
use crate::schema::spaces_channel::dsl::*;
use crate::schema::users::dsl::*;

use diesel::dsl::insert_into;
use diesel::prelude::*;

pub async fn send_message(
    db: web::Data<Pool>,
    token: BearerAuth,
    space_details: web::Path<ChannelPathInfo>,
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
                let space: Space = spaces
                    .filter(spaces_name.ilike(&space_details.info))
                    .first::<Space>(&conn)
                    .unwrap();
                let channel: SpaceChannel = spaces_channel
                    .filter(space_id.eq(&space.id))
                    .filter(channel_name.ilike(&space_details.channel))
                    .first::<SpaceChannel>(&conn)
                    .unwrap();
                match user {
                    Ok(user) => {
                        let new_chat = NewChannelChat {
                            user_id: &user.id,
                            space_channel_id: &channel.id,
                            chat: &item.chat,
                            created_at: chrono::Local::now().naive_local(),
                        };
                        let socket_channel =
                            format!("{}-{}", &space.spaces_name, channel.channel_name);
                        let response: Result<ChannelChat, diesel::result::Error> =
                            insert_into(channel_chats)
                                .values(&new_chat)
                                .get_result(&conn);
                        match response {
                            Ok(response) => {
                                let socket_message = ChannelMessage {
                                    message: response,
                                    user: user,
                                };
                                push_channel_message(
                                    &socket_channel,
                                    &"chat_created".to_string(),
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
    space_details: web::Path<ChannelPathInfoWithId>,
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
                        let updated_chat = diesel::update(channel_chats.find(space_details.id))
                            .set(chat.eq(&item.chat))
                            .execute(&conn);
                        match updated_chat {
                            Ok(_updated_chat) => {
                                let message = channel_chats
                                    .find(space_details.id)
                                    .first::<ChannelChat>(&conn)
                                    .unwrap();
                                let socket_channel =
                                    format!("{}-{}", &space_details.info, space_details.channel);
                                let socket_message = ChannelMessage {
                                    message: message,
                                    user: user,
                                };
                                push_channel_message(
                                    &socket_channel,
                                    &"chat_update".to_string(),
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
    space_name_path: web::Path<ChannelPathInfo>,
    item: web::Query<PaginateQuery>,
) -> Result<HttpResponse, Error> {
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(web::block(move || {
                    get_all_message_db(db, auth.token().to_string(), space_name_path, item)
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
    channel_details: web::Path<MultiIdPathInfo>,
) -> Result<HttpResponse, Error> {
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(web::block(move || {
                    delete_message_db(db, auth.token().to_string(), channel_details)
                })
                .await
                .map(|response| HttpResponse::Ok().json(response))
                .map_err(|_| {
                    HttpResponse::Ok().json(Response::new(false, "Error deleting chat".to_string()))
                })?)
            } else {
                Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
            }
        }
        Err(_) => Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))),
    }
}
