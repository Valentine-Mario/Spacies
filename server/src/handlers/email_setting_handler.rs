use crate::auth;
use crate::diesel::QueryDsl;
use crate::diesel::RunQueryDsl;
use crate::handlers::types::*;
use crate::helpers::encrypt;
use crate::model::{NewSpaceEmail, Space, SpaceEmail, SpaceUser, User};
use crate::schema::spaces::dsl::*;
use crate::schema::spaces_email::dsl::space_id as space_email_id;
use crate::schema::spaces_email::dsl::*;
use crate::schema::spaces_users::dsl::space_id;
use crate::schema::spaces_users::dsl::*;
use crate::schema::users::dsl::*;
use crate::Pool;
use actix_web::{web, Error, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use diesel::dsl::{delete, insert_into};
use diesel::prelude::*;

pub async fn update_email_details(
    db: web::Data<Pool>,
    auth: BearerAuth,
    space_name: web::Path<PathInfo>,
    item: web::Json<AddEmail>,
) -> Result<HttpResponse, Error> {
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(web::block(move || {
                    update_email_details_db(db, auth.token().to_string(), space_name, item)
                })
                .await
                .map(|response| HttpResponse::Ok().json(response))
                .map_err(|_| {
                    HttpResponse::Ok().json(Response::new(
                        false,
                        "Error updating email details".to_string(),
                    ))
                })?)
            } else {
                Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
            }
        }
        Err(_) => Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))),
    }
}

pub async fn delete_email_setting(
    db: web::Data<Pool>,
    auth: BearerAuth,
    space_name: web::Path<PathInfo>,
) -> Result<HttpResponse, Error> {
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(web::block(move || {
                    delete_email_setting_db(db, auth.token().to_string(), space_name)
                })
                .await
                .map(|response| HttpResponse::Ok().json(response))
                .map_err(|_| {
                    HttpResponse::Ok().json(Response::new(
                        false,
                        "Error deleting email details".to_string(),
                    ))
                })?)
            } else {
                Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
            }
        }
        Err(_) => Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))),
    }
}

pub async fn get_email_setting(
    db: web::Data<Pool>,
    auth: BearerAuth,
    space_name: web::Path<PathInfo>,
) -> Result<HttpResponse, Error> {
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(web::block(move || {
                    get_email_setting_db(db, auth.token().to_string(), space_name)
                })
                .await
                .map(|response| HttpResponse::Ok().json(response))
                .map_err(|_| {
                    HttpResponse::Ok().json(Response::new(
                        false,
                        "Error deleting email details".to_string(),
                    ))
                })?)
            } else {
                Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
            }
        }
        Err(_) => Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))),
    }
}
fn update_email_details_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<PathInfo>,
    item: web::Json<AddEmail>,
) -> Result<Response<String>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let decoded_token = auth::decode_token(&token);
    let user: User = users
        .find(decoded_token.parse::<i32>().unwrap())
        .first::<User>(&conn)?;
    let space: Space = spaces
        .filter(spaces_name.ilike(&space_name.info))
        .first::<Space>(&conn)?;
    let spaces_user: SpaceUser = spaces_users
        .filter(space_id.eq(space.id))
        .filter(user_id.eq(user.id))
        .first::<SpaceUser>(&conn)?;

    if !spaces_user.admin_status {
        return Ok(Response::new(
            false,
            "Only admin authorized to edit email settings".to_string(),
        ));
    }

    //get email details
    let email_details = spaces_email
        .filter(space_email_id.eq(space.id))
        .first::<SpaceEmail>(&conn);

    //encrypt email password
    let pass = encrypt::encrypt(&item.email_password);

    match email_details {
        Ok(_space_email_data) => {
            //if email cred exist update it
            diesel::update(spaces_email.find(space.id))
                .set((
                    email_address.eq(&item.email_address),
                    email_password.eq(&pass),
                    email_provider.eq(&item.email_provider)
                ))
                .execute(&conn)?;
            return Ok(Response::new(
                true,
                "email details updated successfully".to_string(),
            ));
        }
        Err(diesel::result::Error::NotFound) => {
            //if email cred does not exist, create it
            let new_space_email = NewSpaceEmail {
                email_address: &item.email_address,
                email_password: &pass,
                email_provider:&item.email_provider,
                space_id: &space.id,
            };

            insert_into(spaces_email)
                .values(&new_space_email)
                .execute(&conn)?;
            return Ok(Response::new(
                true,
                "email settings added successfully".to_string(),
            ));
        }
        _ => {
            return Ok(Response::new(
                false,
                "error setting email details".to_string(),
            ))
        }
    }
}

fn get_email_setting_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<PathInfo>,
) -> Result<Response<SpaceEmail>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let decoded_token = auth::decode_token(&token);
    let user = users
        .find(decoded_token.parse::<i32>().unwrap())
        .first::<User>(&conn)?;
    let space: Space = spaces
        .filter(spaces_name.ilike(&space_name.info))
        .first::<Space>(&conn)?;
    let _spaces_user: SpaceUser = spaces_users
        .filter(space_id.eq(space.id))
        .filter(user_id.eq(user.id))
        .first::<SpaceUser>(&conn)?;
    let space_email_details: SpaceEmail = spaces_email
        .filter(space_email_id.eq(space.id))
        .first::<SpaceEmail>(&conn)?;
    Ok(Response::new(true, space_email_details))
}

fn delete_email_setting_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<PathInfo>,
) -> Result<Response<String>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let decoded_token = auth::decode_token(&token);
    let user: User = users
        .find(decoded_token.parse::<i32>().unwrap())
        .first::<User>(&conn)?;
    let space: Space = spaces
        .filter(spaces_name.ilike(&space_name.info))
        .first::<Space>(&conn)?;
    let spaces_user: SpaceUser = spaces_users
        .filter(space_id.eq(space.id))
        .filter(user_id.eq(user.id))
        .first::<SpaceUser>(&conn)?;

    if !spaces_user.admin_status {
        return Ok(Response::new(
            false,
            "Only admin authorized to delete email settings".to_string(),
        ));
    }
    let _a = delete(spaces_email.filter(space_email_id.eq(space.id))).execute(&conn);
    Ok(Response::new(
        true,
        "email details updated successfully".to_string(),
    ))
}
