use crate::auth;
use crate::diesel::QueryDsl;
use crate::diesel::RunQueryDsl;
use crate::handlers::types::*;
use crate::helpers::{aws, bcrypt, email, email_template};
use crate::model::{NewUser, User};
use crate::schema::users::dsl::*;
use crate::Pool;

use actix_files::NamedFile;
use actix_multipart::Multipart;
use actix_web::{web, Error, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use diesel::dsl::insert_into;
use diesel::prelude::*;
use futures::{StreamExt, TryStreamExt};
use std::fs;
use std::fs::remove_file;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

//http responses
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

//db calls
fn updat_name_db(
    db: web::Data<Pool>,
    item: web::Json<UpdateName>,
    token: String,
) -> Result<Response<String>, diesel::result::Error> {
    let decoded_token = auth::decode_token(&token);
    let conn = db.get().unwrap();
    let _user_verified = diesel::update(users.find(decoded_token.parse::<i32>().unwrap()))
        .set(username.eq(&item.username))
        .execute(&conn);
    Ok(Response::new(
        true,
        "username updated successfully".to_string(),
    ))
}

fn get_profile_db(
    db: web::Data<Pool>,
    token: String,
) -> Result<Response<User>, diesel::result::Error> {
    let decoded_token = auth::decode_token(&token);
    let conn = db.get().unwrap();
    let user: User = users
        .find(decoded_token.parse::<i32>().unwrap())
        .first(&conn)?;
    Ok(Response::new(true, user))
}

fn login_db(
    db: web::Data<Pool>,
    item: web::Json<LoginUser>,
) -> Result<Response<String>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let user_details = users.filter(email.ilike(&item.email)).first::<User>(&conn);
    match user_details {
        Ok(user) => {
            let cmp_pwd = bcrypt::compare_password(&user.user_password, &item.user_password);
            if cmp_pwd {
                let token = auth::create_token(&user.id.to_string(), 30).unwrap();
                return Ok(Response::new(true, token.to_string()));
            } else {
                return Ok(Response::new(false, "invalid password".to_string()));
            }
        }
        Err(diesel::result::Error::NotFound) => {
            Ok(Response::new(false, "email not found".to_string()))
        }
        _ => Ok(Response::new(false, "email not found".to_string())),
    }
}

fn resend_verification_db(
    db: web::Data<Pool>,
    token: String,
) -> Result<Response<String>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let decoded_token = auth::decode_token(&token);
    let user: User = users
        .find(decoded_token.parse::<i32>().unwrap())
        .first(&conn)?;
    if user.verified {
        return Ok(Response::new(
            false,
            "your account is already verified".to_string(),
        ));
    } else {
        //send user verification email
        let mail_token = auth::create_token(&user.id.to_string(), 1).unwrap();
        let email_template = email_template::verification_email(&mail_token);
        email::send_email(
            &user.email,
            &user.username,
            &"Welcome To Spacies".to_string(),
            &email_template,
        );
        return Ok(Response::new(
            true,
            "verification email sent successfully".to_string(),
        ));
    }
}

fn verify_user_db(
    db: web::Data<Pool>,
    item: web::Query<QueryInfo>,
) -> Result<(), diesel::result::Error> {
    let conn = db.get().unwrap();
    let decoded_token = auth::decode_token(&item.token);
    let _user_verified = diesel::update(users.find(decoded_token.parse::<i32>().unwrap()))
        .set(verified.eq(true))
        .execute(&conn);
    Ok(())
}

fn add_user_db(
    db: web::Data<Pool>,
    item: web::Json<CreateUser>,
) -> Result<Response<String>, diesel::result::Error> {
    let conn = db.get().unwrap();

    //password should be min of 6 char
    if &item.user_password.chars().count() < &6 {
        return Ok(Response::new(
            false,
            "password should be min of 6 characters".to_string(),
        ));
    }
    let hashed = bcrypt::encrypt_password(&item.user_password.to_string());
    let new_user = NewUser {
        username: &item.username,
        email: &item.email,
        user_password: &hashed,
        verified: &false,
        created_at: chrono::Local::now().naive_local(),
    };
    let res: User = insert_into(users).values(&new_user).get_result(&conn)?;

    //send user verification email
    let mail_token = auth::create_token(&res.id.to_string(), 1).unwrap();
    let email_template = email_template::verification_email(&mail_token);
    email::send_email(
        &res.email,
        &res.username,
        &"Welcome To Spacies".to_string(),
        &email_template,
    );

    //create user token
    match auth::create_token(&res.id.to_string(), 30) {
        Ok(token) => return Ok(Response::new(true, token.to_string())),
        _ => Ok(Response::new(false, "error creating token".to_string())),
    }
}
