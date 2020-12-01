use crate::auth;
use crate::diesel::QueryDsl;
use crate::diesel::RunQueryDsl;
use crate::handlers::types::*;
use crate::model::{MailList, NewMailList, Space, SpaceUser, User, UserMail};
use crate::schema::maillists::dsl::space_id as mail_space_id;
use crate::schema::maillists::dsl::*;
use crate::schema::spaces::dsl::*;
use crate::schema::spaces_users::dsl::space_id;
use crate::schema::spaces_users::dsl::*;
use crate::schema::usermails::dsl::mail_list_id;
use crate::schema::usermails::dsl::usermails;
use crate::schema::users::dsl::*;
use crate::Pool;

use actix_web::{web, Error, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use diesel::dsl::{delete, insert_into};
use diesel::prelude::*;

//http request
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
    space_name: web::Path<PathInfo>,
    item: web::Json<DeleteMailList>,
) -> Result<HttpResponse, Error> {
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(web::block(move || {
                    delete_mail_folder_db(db, auth.token().to_string(), space_name, item)
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

//db calls
fn get_mail_folder_details_db(
    db: web::Data<Pool>,
    space_name: web::Path<MailChannelPathInfo>,
) -> Result<Response<(MailList, Vec<(UserMail, User)>)>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let space: Space = spaces
        .filter(spaces_name.ilike(&space_name.info))
        .first::<Space>(&conn)?;
    let mail_list: MailList = maillists
        .filter(mail_space_id.eq(space.id))
        .filter(folder_name.ilike(&space_name.name))
        .first::<MailList>(&conn)?;

    let user_mail: Vec<_> = UserMail::belonging_to(&mail_list)
        .inner_join(users)
        .load::<(UserMail, User)>(&conn)?;
    Ok(Response::new(true, (mail_list, user_mail)))
}

fn get_space_mail_folder_db(
    db: web::Data<Pool>,
    space_name: web::Path<PathInfo>,
) -> Result<Response<Vec<MailList>>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let space: Space = spaces
        .filter(spaces_name.ilike(&space_name.info))
        .first::<Space>(&conn)?;
    let mail_folder: Vec<MailList> = maillists
        .filter(mail_space_id.eq(space.id))
        .load::<MailList>(&conn)?;
    Ok(Response::new(true, mail_folder))
}

fn delete_mail_folder_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<PathInfo>,
    item: web::Json<DeleteMailList>,
) -> Result<Response<String>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let decoded_token = auth::decode_token(&token);
    let user = users
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
            "only admin allowed to delete mail folders".to_string(),
        ));
    };
    let mail_folder: MailList = maillists.find(&item.id).first::<MailList>(&conn)?;
    //delete all user email relationship
    let _count = delete(usermails.filter(mail_list_id.eq(mail_folder.id))).execute(&conn)?;
    //delet mail folder
    let _count2 = delete(maillists.find(mail_folder.id)).execute(&conn)?;

    Ok(Response::new(
        true,
        "mail folder deleted successfully".to_string(),
    ))
}

fn add_mail_folder_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<PathInfo>,
    item: web::Json<CreateMailList>,
) -> Result<Response<String>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let decoded_token = auth::decode_token(&token);
    let user = users
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
            "only admin allowed to add mail folders".to_string(),
        ));
    }
    let mail_list: Vec<String> = maillists
        .filter(mail_space_id.eq(&space.id))
        .select(folder_name)
        .load::<String>(&conn)?;
    if mail_list
        .iter()
        .any(|i| &i.to_lowercase() == &item.folder_name.to_lowercase())
    {
        return Ok(Response::new(
            false,
            "A similar folder name already exist".to_string(),
        ));
    }

    let new_mail_list = NewMailList {
        folder_name: &item.folder_name,
        space_id: &space.id,
    };
    let _res: MailList = insert_into(maillists)
        .values(&new_mail_list)
        .get_result(&conn)?;
    Ok(Response::new(
        true,
        "Mail folder created successfully".to_string(),
    ))
}
