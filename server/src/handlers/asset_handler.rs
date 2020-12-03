use crate::auth;
use crate::diesel::QueryDsl;
use crate::diesel::RunQueryDsl;
use crate::handlers::paginate::*;
use crate::handlers::types::*;
use crate::model::{Asset, NewAsset, Space, SpaceUser, User};
use crate::schema::asset_contents::dsl::{asset_contents, asset_id};
use crate::schema::assets::dsl::created_at as asset_create_at;
use crate::schema::assets::dsl::space_id as asset_space_id;
use crate::schema::assets::dsl::*;
use crate::schema::spaces::dsl::*;
use crate::schema::spaces_users::dsl::space_id;
use crate::schema::spaces_users::dsl::*;
use crate::schema::users::dsl::*;
use crate::Pool;

use actix_web::{web, Error, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use diesel::dsl::{delete, insert_into};
use diesel::prelude::*;

//http calls
pub async fn create_asset_folder(
    db: web::Data<Pool>,
    auth: BearerAuth,
    space_name: web::Path<PathInfo>,
    item: web::Json<AddAssetFolder>,
) -> Result<HttpResponse, Error> {
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(web::block(move || {
                    create_asset_folder_db(db, auth.token().to_string(), space_name, item)
                })
                .await
                .map(|response| HttpResponse::Ok().json(response))
                .map_err(|_| {
                    HttpResponse::Ok()
                        .json(Response::new(false, "Error creating folder".to_string()))
                })?)
            } else {
                Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
            }
        }
        Err(_) => Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))),
    }
}

pub async fn update_folder_name(
    db: web::Data<Pool>,
    auth: BearerAuth,
    space_name: web::Path<AddUserToFolderPath>,
    item: web::Json<AddAssetFolder>,
) -> Result<HttpResponse, Error> {
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(web::block(move || {
                    update_folder_name_db(db, auth.token().to_string(), space_name, item)
                })
                .await
                .map(|response| HttpResponse::Ok().json(response))
                .map_err(|_| {
                    HttpResponse::Ok()
                        .json(Response::new(false, "Error updating folder".to_string()))
                })?)
            } else {
                Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
            }
        }
        Err(_) => Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))),
    }
}

pub async fn get_asset_folder(
    db: web::Data<Pool>,
    auth: BearerAuth,
    space_name: web::Path<PathInfo>,
    item: web::Query<PaginateQuery>,
) -> Result<HttpResponse, Error> {
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(web::block(move || {
                    get_asset_folder_db(db, auth.token().to_string(), space_name, item)
                })
                .await
                .map(|response| HttpResponse::Ok().json(response))
                .map_err(|_| {
                    HttpResponse::Ok()
                        .json(Response::new(false, "Error getting folder".to_string()))
                })?)
            } else {
                Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
            }
        }
        Err(_) => Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))),
    }
}

pub async fn search_asset_folder(
    db: web::Data<Pool>,
    auth: BearerAuth,
    space_name: web::Path<MailChannelPathInfo>,
    item: web::Query<PaginateQuery>,
) -> Result<HttpResponse, Error> {
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(web::block(move || {
                    search_asset_folder_db(db, auth.token().to_string(), space_name, item)
                })
                .await
                .map(|response| HttpResponse::Ok().json(response))
                .map_err(|_| {
                    HttpResponse::Ok()
                        .json(Response::new(false, "Error searching folder".to_string()))
                })?)
            } else {
                Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
            }
        }
        Err(_) => Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))),
    }
}

pub async fn delete_asset_folder(
    db: web::Data<Pool>,
    auth: BearerAuth,
    space_name: web::Path<AddUserToFolderPath>,
) -> Result<HttpResponse, Error> {
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(web::block(move || {
                    delete_asset_folder_db(db, auth.token().to_string(), space_name)
                })
                .await
                .map(|response| HttpResponse::Ok().json(response))
                .map_err(|_| {
                    HttpResponse::Ok()
                        .json(Response::new(false, "Error deleting folder".to_string()))
                })?)
            } else {
                Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
            }
        }
        Err(_) => Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))),
    }
}

//db calls
fn delete_asset_folder_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<AddUserToFolderPath>,
) -> Result<Response<String>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let decoded_token = auth::decode_token(&token);
    let user = users
        .find(decoded_token.parse::<i32>().unwrap())
        .first::<User>(&conn)?;
    let space = spaces
        .filter(spaces_name.ilike(&space_name.info))
        .first::<Space>(&conn)?;
    let spaces_user: SpaceUser = spaces_users
        .filter(space_id.eq(space.id))
        .filter(user_id.eq(user.id))
        .first::<SpaceUser>(&conn)?;

    if !spaces_user.admin_status {
        return Ok(Response::new(
            false,
            "only admin allowed to delete asset folder".to_string(),
        ));
    }
    let asset_folder: Asset = assets.find(&space_name.id).first::<Asset>(&conn)?;
    //delete all asset content relationship
    let _count = delete(asset_contents.filter(asset_id.eq(asset_folder.id))).execute(&conn)?;
    //delet asset folder
    let _count2 = delete(assets.find(asset_folder.id)).execute(&conn)?;
    Ok(Response::new(
        true,
        "folder deleted successfully".to_string(),
    ))
}

fn search_asset_folder_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<MailChannelPathInfo>,
    item: web::Query<PaginateQuery>,
) -> Result<Response<(Vec<Asset>, i64)>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let decoded_token = auth::decode_token(&token);
    let mut return_list = vec![];
    let mut total_val = 0;
    let user = users
        .find(decoded_token.parse::<i32>().unwrap())
        .first::<User>(&conn)?;
    let space = spaces
        .filter(spaces_name.ilike(&space_name.info))
        .first::<Space>(&conn)?;
    let _spaces_user: SpaceUser = spaces_users
        .filter(space_id.eq(space.id))
        .filter(user_id.eq(user.id))
        .first::<SpaceUser>(&conn)?;

    let values: Vec<_> = space_name.name.split(' ').collect();
    for b in values.iter() {
        let a = format!("%{}%", b);
        let folders = assets
            .filter(asset_space_id.eq(&space.id))
            .filter(folder_name.ilike(&a))
            .order(asset_create_at.desc())
            .paginate(item.page)
            .per_page(item.per_page)
            .load::<(Asset, i64)>(&conn)?;
        total_val += folders.get(0).map(|x| x.1).unwrap_or(0);
        let list: Vec<Asset> = folders.into_iter().map(|x| x.0).collect();
        for c in list {
            return_list.push(c)
        }
    }

    Ok(Response::new(true, (return_list, total_val)))
}

fn get_asset_folder_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<PathInfo>,
    item: web::Query<PaginateQuery>,
) -> Result<Response<(Vec<Asset>, i64)>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let decoded_token = auth::decode_token(&token);
    let user = users
        .find(decoded_token.parse::<i32>().unwrap())
        .first::<User>(&conn)?;
    let space = spaces
        .filter(spaces_name.ilike(&space_name.info))
        .first::<Space>(&conn)?;
    let _spaces_user: SpaceUser = spaces_users
        .filter(space_id.eq(space.id))
        .filter(user_id.eq(user.id))
        .first::<SpaceUser>(&conn)?;

    let folders = assets
        .filter(asset_space_id.eq(&space.id))
        .order(asset_create_at.desc())
        .paginate(item.page)
        .per_page(item.per_page)
        .load::<(Asset, i64)>(&conn)?;

    let total = folders.get(0).map(|x| x.1).unwrap_or(0);
    let folder_list = folders.into_iter().map(|x| x.0).collect();

    Ok(Response::new(true, (folder_list, total)))
}

fn update_folder_name_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<AddUserToFolderPath>,
    item: web::Json<AddAssetFolder>,
) -> Result<Response<String>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let decoded_token = auth::decode_token(&token);
    let user = users
        .find(decoded_token.parse::<i32>().unwrap())
        .first::<User>(&conn)?;
    let space = spaces
        .filter(spaces_name.ilike(&space_name.info))
        .first::<Space>(&conn)?;
    let spaces_user: SpaceUser = spaces_users
        .filter(space_id.eq(space.id))
        .filter(user_id.eq(user.id))
        .first::<SpaceUser>(&conn)?;

    if !spaces_user.admin_status {
        return Ok(Response::new(
            false,
            "only admin allowed to update asset folder".to_string(),
        ));
    }

    let folders: Vec<String> = assets
        .filter(asset_space_id.eq(&space.id))
        .select(folder_name)
        .load::<String>(&conn)?;

    if folders
        .iter()
        .any(|i| &i.to_lowercase() == &item.folder_name.to_lowercase())
    {
        return Ok(Response::new(false, "Asset name already taken".to_string()));
    };

    let _asset_details_update = diesel::update(assets.find(space_name.id))
        .set(folder_name.eq(&item.folder_name.to_lowercase()))
        .execute(&conn)?;

    Ok(Response::new(
        true,
        "Folder name updated successfully".to_string(),
    ))
}

fn create_asset_folder_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<PathInfo>,
    item: web::Json<AddAssetFolder>,
) -> Result<Response<String>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let decoded_token = auth::decode_token(&token);
    let user = users
        .find(decoded_token.parse::<i32>().unwrap())
        .first::<User>(&conn)?;
    let space = spaces
        .filter(spaces_name.ilike(&space_name.info))
        .first::<Space>(&conn)?;
    let spaces_user: SpaceUser = spaces_users
        .filter(space_id.eq(space.id))
        .filter(user_id.eq(user.id))
        .first::<SpaceUser>(&conn)?;

    if !spaces_user.admin_status {
        return Ok(Response::new(
            false,
            "only admin allowed to add asset folder".to_string(),
        ));
    }
    let folders: Vec<String> = assets
        .filter(asset_space_id.eq(&space.id))
        .select(folder_name)
        .load::<String>(&conn)?;

    if folders
        .iter()
        .any(|i| &i.to_lowercase() == &item.folder_name.to_lowercase())
    {
        return Ok(Response::new(false, "Asset name already taken".to_string()));
    };

    let new_folder = NewAsset {
        folder_name: &item.folder_name.to_lowercase(),
        space_id: &space.id,
        created_at: chrono::Local::now().naive_local(),
    };

    let _asset_folder = insert_into(assets).values(&new_folder).execute(&conn)?;

    Ok(Response::new(
        true,
        "asset folder created successfully".to_string(),
    ))
}
