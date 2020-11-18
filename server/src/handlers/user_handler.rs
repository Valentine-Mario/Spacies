use crate::auth;
use crate::diesel::QueryDsl;
use crate::diesel::RunQueryDsl;
use crate::handlers::types::*;
use crate::helpers::{aws, bcrypt, email, email_template};
use crate::model::{NewUser, User};
use crate::schema::users::dsl::*;
use crate::Pool;

use actix_web::{web, Error, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use diesel::dsl::{delete, insert_into};
use diesel::prelude::*;

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







//db calls
fn add_user_db(
    db: web::Data<Pool>,
    item: web::Json<CreateUser>,
) -> Result<Response<String>, diesel::result::Error> {
    let conn = db.get().unwrap();

    //password should be min of 6 char
    if &item.user_password.chars().count()< &6{
        return Ok(Response::new(false, "password should be min of 6 characters".to_string()))
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
