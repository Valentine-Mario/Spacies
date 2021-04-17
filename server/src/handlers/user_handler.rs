use crate::auth;
use crate::diesel::QueryDsl;
use crate::diesel::RunQueryDsl;
use crate::handlers::types::*;
use crate::helpers::aws;

use crate::schema::users::dsl::*;
use crate::Pool;

use actix_files::NamedFile;
use actix_multipart::Multipart;
use actix_web::{web, Error, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use diesel::prelude::*;
use futures::{StreamExt, TryStreamExt};

use crate::controllers::user_controller::*;
use std::fs;
use std::fs::remove_file;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn add_user(
    db: web::Data<Pool>,
    item: web::Json<CreateUser>,
) -> Result<HttpResponse, Error> {
    Ok(web::block(move || add_user_db(db, item))
        .await
        .map(|user| HttpResponse::Created().json(user))
        .map_err(|_| {
            HttpResponse::Ok().json(Response::new(false, "email already in use".to_string()))
        })?)
}

pub async fn verify_user(
    db: web::Data<Pool>,
    item: web::Query<QueryInfo>,
) -> Result<NamedFile, ()> {
    let success_file = format!("./pages/welcome.html");
    let error_file = format!("./pages/jwt_user_error.html");
    match auth::validate_token(&item.token.to_string()) {
        Ok(res) => {
            if res == true {
                web::block(move || verify_user_db(db, item))
                    .await
                    .map(|_user| NamedFile::open(success_file).unwrap())
                    .map_err(|_| ())
            } else {
                Ok(NamedFile::open(error_file).unwrap())
            }
        }
        Err(_) => Ok(NamedFile::open(error_file).unwrap()),
    }
}

pub async fn resend_verification(
    db: web::Data<Pool>,
    auth: BearerAuth,
) -> Result<HttpResponse, Error> {
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(
                    web::block(move || resend_verification_db(db, auth.token().to_string()))
                        .await
                        .map(|response| HttpResponse::Ok().json(response))
                        .map_err(|_| HttpResponse::InternalServerError())?,
                )
            } else {
                Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
            }
        }
        Err(_) => Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))),
    }
}

pub async fn login(db: web::Data<Pool>, item: web::Json<LoginUser>) -> Result<HttpResponse, Error> {
    Ok(web::block(move || login_db(db, item))
        .await
        .map(|user| HttpResponse::Ok().json(user))
        .map_err(|_| HttpResponse::InternalServerError())?)
}

pub async fn get_profile(db: web::Data<Pool>, auth: BearerAuth) -> Result<HttpResponse, Error> {
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(
                    web::block(move || get_profile_db(db, auth.token().to_string()))
                        .await
                        .map(|user| HttpResponse::Ok().json(user))
                        .map_err(|_| HttpResponse::InternalServerError())?,
                )
            } else {
                Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
            }
        }
        Err(_) => Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))),
    }
}

pub async fn update_profile_img(
    db: web::Data<Pool>,
    mut payload: Multipart,
    token: BearerAuth,
) -> Result<HttpResponse, Error> {
    match auth::validate_token(&token.token().to_string()) {
        Ok(res) => {
            if res == true {
                while let Ok(Some(mut field)) = payload.try_next().await {
                    //get file content
                    let content_type = field.content_disposition().unwrap();
                    let filename = format!(
                        "{}-{}",
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_micros(),
                        content_type.get_filename().unwrap(),
                    );
                    let filepath = format!("./tmp/{}", sanitize_filename::sanitize(&filename));

                    // File::create is blocking operation, use threadpool
                    let mut f = web::block(|| std::fs::File::create(filepath))
                        .await
                        .unwrap();

                    // Field in turn is stream of *Bytes* object
                    while let Some(chunk) = field.next().await {
                        let data = chunk.unwrap();
                        // filesystem operations are blocking, we have to use threadpool
                        f = web::block(move || f.write_all(&data).map(|_| f))
                            .await
                            .unwrap();
                    }

                    //extract file vector
                    let file: Vec<u8> = fs::read(format!("./tmp/{}", &filename)).unwrap();
                    let metadata = fs::metadata(format!("./tmp/{}", &filename)).unwrap();
                    //validate if file is over 5 MB
                    if metadata.len() > 5000000 {
                        remove_file(format!("./tmp/{}", &filename)).unwrap();
                        return Ok(HttpResponse::Ok().json(Response::new(
                            false,
                            "file should not be larger than 5 MB".to_string(),
                        )));
                    } else {
                        remove_file(format!("./tmp/{}", &filename)).unwrap();
                        //upload to aws
                        let uploaded = web::block(|| aws::aws_func(filename, file)).await;
                        match uploaded {
                            Ok(file_link) => {
                                let conn = db.get().unwrap();
                                let decoded_token = auth::decode_token(&token.token().to_string());
                                let _user_details = diesel::update(
                                    users.find(decoded_token.parse::<i32>().unwrap()),
                                )
                                .set(user_image.eq(&file_link))
                                .execute(&conn);
                                return Ok(HttpResponse::Ok().json(Response::new(true, file_link)));
                            }
                            _ => {
                                return Ok(HttpResponse::Ok().json(Response::new(
                                    false,
                                    "error uploading file".to_string(),
                                )))
                            }
                        }
                    }
                }
            } else {
                return Ok(
                    HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))
                );
            }
        }
        Err(_) => {
            return Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
        }
    }
    Ok(HttpResponse::Ok().json(Response::new(true, "file upload successful".to_string())))
}

pub async fn update_name(
    db: web::Data<Pool>,
    item: web::Json<UpdateName>,
    auth: BearerAuth,
) -> Result<HttpResponse, Error> {
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(
                    web::block(move || updat_name_db(db, item, auth.token().to_string()))
                        .await
                        .map(|user| HttpResponse::Ok().json(user))
                        .map_err(|_| {
                            HttpResponse::Ok().json(Response::new(false, "error updating name"))
                        })?,
                )
            } else {
                Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
            }
        }
        Err(_) => Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))),
    }
}

pub async fn update_password(
    db: web::Data<Pool>,
    item: web::Json<UpdatePassword>,
    auth: BearerAuth,
) -> Result<HttpResponse, Error> {
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(
                    web::block(move || update_password_db(db, item, auth.token().to_string()))
                        .await
                        .map(|user| HttpResponse::Ok().json(user))
                        .map_err(|_| {
                            HttpResponse::Ok().json(Response::new(false, "error updating password"))
                        })?,
                )
            } else {
                Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
            }
        }
        Err(_) => Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))),
    }
}

pub async fn forgot_password(
    db: web::Data<Pool>,
    item: web::Json<ForgotPassword>,
) -> Result<HttpResponse, Error> {
    Ok(web::block(move || forgot_password_db(db, item))
        .await
        .map(|user| HttpResponse::Ok().json(user))
        .map_err(|_| HttpResponse::Ok().json(Response::new(false, "error sending mail")))?)
}
