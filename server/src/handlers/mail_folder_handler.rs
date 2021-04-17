use crate::auth;
use crate::handlers::types::*;
use crate::Pool;

use crate::controllers::mail_folder_controller::*;
use actix_web::{web, Error, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;

pub async fn add_mail_folder(
    db: web::Data<Pool>,
    auth: BearerAuth,
    space_name: web::Path<PathInfo>,
    item: web::Json<CreateMailList>,
) -> Result<HttpResponse, Error> {
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(web::block(move || {
                    add_mail_folder_db(db, auth.token().to_string(), space_name, item)
                })
                .await
                .map(|response| HttpResponse::Ok().json(response))
                .map_err(|_| {
                    HttpResponse::Ok().json(Response::new(
                        false,
                        "Error creating mail folder".to_string(),
                    ))
                })?)
            } else {
                Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
            }
        }
        Err(_) => Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))),
    }
}

pub async fn delete_mail_folder(
    db: web::Data<Pool>,
    auth: BearerAuth,
    space_name: web::Path<AddUserToFolderPath>,
) -> Result<HttpResponse, Error> {
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(web::block(move || {
                    delete_mail_folder_db(db, auth.token().to_string(), space_name)
                })
                .await
                .map(|response| HttpResponse::Ok().json(response))
                .map_err(|_| {
                    HttpResponse::Ok().json(Response::new(
                        false,
                        "Error deleting mail folder".to_string(),
                    ))
                })?)
            } else {
                Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
            }
        }
        Err(_) => Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))),
    }
}

pub async fn get_space_mail_folder(
    db: web::Data<Pool>,
    space_name: web::Path<PathInfo>,
) -> Result<HttpResponse, Error> {
    Ok(web::block(move || get_space_mail_folder_db(db, space_name))
        .await
        .map(|response| HttpResponse::Ok().json(response))
        .map_err(|_| {
            HttpResponse::Ok().json(Response::new(
                false,
                "Error retrieving mail folder".to_string(),
            ))
        })?)
}

pub async fn get_mail_folder_details(
    db: web::Data<Pool>,
    space_name: web::Path<MailChannelPathInfo>,
) -> Result<HttpResponse, Error> {
    Ok(
        web::block(move || get_mail_folder_details_db(db, space_name))
            .await
            .map(|response| HttpResponse::Ok().json(response))
            .map_err(|_| {
                HttpResponse::Ok().json(Response::new(
                    false,
                    "Error retrieving mail folder details".to_string(),
                ))
            })?,
    )
}
