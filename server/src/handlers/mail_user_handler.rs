use crate::auth;
use crate::handlers::types::*;
use crate::Pool;

use crate::controllers::mail_user_controller::*;
use actix_web::{web, Error, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;

pub async fn add_user_folder(
    db: web::Data<Pool>,
    auth: BearerAuth,
    folder: web::Path<AddUserToFolderPath>,
    item: web::Json<AddUserToFoldr>,
) -> Result<HttpResponse, Error> {
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(web::block(move || {
                    add_user_folder_db(db, auth.token().to_string(), folder, item)
                })
                .await
                .map(|response| HttpResponse::Ok().json(response))
                .map_err(|_| HttpResponse::InternalServerError())?)
            } else {
                Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
            }
        }
        Err(_) => Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))),
    }
}

pub async fn remove_user_folder(
    db: web::Data<Pool>,
    auth: BearerAuth,
    folder: web::Path<AddUserToFolderPath>,
    item: web::Json<DeleteMailList>,
) -> Result<HttpResponse, Error> {
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(web::block(move || {
                    remove_user_folder_db(db, auth.token().to_string(), folder, item)
                })
                .await
                .map(|response| HttpResponse::Ok().json(response))
                .map_err(|_| HttpResponse::InternalServerError())?)
            } else {
                Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
            }
        }
        Err(_) => Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))),
    }
}

pub async fn send_mail_to_folder(
    db: web::Data<Pool>,
    folder_id: web::Path<AddUserToFolderPath>,
    item: web::Json<SendMail>,
    auth: BearerAuth,
) -> Result<HttpResponse, Error> {
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(web::block(move || {
                    send_mail_to_folder_db(db, folder_id, auth.token().to_string(), item)
                })
                .await
                .map(|response| HttpResponse::Ok().json(response))
                .map_err(|_| HttpResponse::InternalServerError())?)
            } else {
                Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
            }
        }
        Err(_) => Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))),
    }
}

pub async fn send_email_to_general(
    db: web::Data<Pool>,
    space_name: web::Path<PathInfo>,
    auth: BearerAuth,
    item: web::Json<SendMail>,
) -> Result<HttpResponse, Error> {
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(web::block(move || {
                    send_email_to_general_db(db, space_name, auth.token().to_string(), item)
                })
                .await
                .map(|response| HttpResponse::Ok().json(response))
                .map_err(|_| HttpResponse::InternalServerError())?)
            } else {
                Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
            }
        }
        Err(_) => Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))),
    }
}
