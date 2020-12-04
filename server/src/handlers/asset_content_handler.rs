use crate::auth;
use crate::diesel::QueryDsl;
use crate::diesel::RunQueryDsl;
use crate::handlers::types::*;
use crate::helpers::aws;
use crate::model::{Asset, AssetContent, NewAssetContent, Space, SpaceUser, User};
use crate::schema::asset_contents::dsl::*;
use crate::schema::assets::dsl::space_id as asset_space_id;
use crate::schema::assets::dsl::*;
use crate::schema::spaces::dsl::*;
use crate::schema::spaces_users::dsl::space_id;
use crate::schema::spaces_users::dsl::*;
use crate::schema::users::dsl::*;
use crate::Pool;

use actix_multipart::Multipart;
use actix_web::{web, Error, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use diesel::dsl::{delete, insert_into};
use diesel::prelude::*;
use futures::{StreamExt, TryStreamExt};
use std::fs;
use std::fs::remove_file;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

//http calls
pub async fn upload_file_db(
    db: web::Data<Pool>,
    token: BearerAuth,
    space_name: web::Path<AddFileContent>,
    mut payload: Multipart,
) -> Result<HttpResponse, Error> {
    let mut result = vec![];
    match auth::validate_token(&token.token().to_string()) {
        Ok(res) => {
            if res == true {
                //check if user is an admin
                let decoded_token = auth::decode_token(&token.token().to_string());
                let conn = db.get().unwrap();

                let user: User = users
                    .find(decoded_token.parse::<i32>().unwrap())
                    .first(&conn)
                    .unwrap();

                let space: Space = spaces
                    .filter(spaces_name.ilike(&space_name.info))
                    .first::<Space>(&conn)
                    .unwrap();

                let _spaces_user: SpaceUser = spaces_users
                    .filter(space_id.eq(space.id))
                    .filter(user_id.eq(user.id))
                    .first::<SpaceUser>(&conn)
                    .unwrap();

                let asset_folder: Asset = assets.find(space_name.id).first::<Asset>(&conn).unwrap();

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
                    if metadata.len() > 10000000 {
                        remove_file(format!("./tmp/{}", &filename)).unwrap();
                        return Ok(HttpResponse::Ok().json(Response::new(
                            false,
                            "file should not be larger than 10 MB".to_string(),
                        )));
                    } else {
                        remove_file(format!("./tmp/{}", &filename)).unwrap();
                        //upload to aws
                        let uploaded = web::block(|| aws::aws_func(filename, file)).await;
                        match uploaded {
                            Ok(file_link) => {
                                let new_content = NewAssetContent {
                                    file_content: &file_link,
                                    file_type: &space_name.file_type,
                                    asset_id: &asset_folder.id,
                                    created_at: chrono::Local::now().naive_local(),
                                };
                                let res: AssetContent = insert_into(asset_contents)
                                    .values(&new_content)
                                    .get_result(&conn)
                                    .unwrap();
                                result.push(res);
                            }
                            _ => {
                                println!("Error uploading doc");
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
    Ok(HttpResponse::Ok().json(Response::new(true, result)))
}

pub async fn get_files(
    db: web::Data<Pool>,
    auth: BearerAuth,
    space_name: web::Path<MailChannelPathInfo>,
) -> Result<HttpResponse, Error> {
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(
                    web::block(move || get_files_db(db, auth.token().to_string(), space_name))
                        .await
                        .map(|response| HttpResponse::Ok().json(response))
                        .map_err(|_| {
                            HttpResponse::Ok()
                                .json(Response::new(false, "Error getting content".to_string()))
                        })?,
                )
            } else {
                Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
            }
        }
        Err(_) => Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))),
    }
}

pub async fn delete_upload(
    db: web::Data<Pool>,
    auth: BearerAuth,
    space_name: web::Path<AddUserToFolderPath>,
) -> Result<HttpResponse, Error> {
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(
                    web::block(move || delete_upload_db(db, auth.token().to_string(), space_name))
                        .await
                        .map(|response| HttpResponse::Ok().json(response))
                        .map_err(|_| {
                            HttpResponse::Ok()
                                .json(Response::new(false, "Error deleting content".to_string()))
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

fn get_files_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<MailChannelPathInfo>,
) -> Result<Response<(Asset, Vec<AssetContent>)>, diesel::result::Error> {
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
    let asset_folder: Asset = assets
        .filter(folder_name.ilike(&space_name.name))
        .filter(asset_space_id.eq(space.id))
        .first::<Asset>(&conn)?;
    let asset_conents: Vec<AssetContent> = asset_contents
        .filter(asset_id.eq(asset_folder.id))
        .load::<AssetContent>(&conn)?;

    Ok(Response::new(true, (asset_folder, asset_conents)))
}

fn delete_upload_db(
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
    let _spaces_user: SpaceUser = spaces_users
        .filter(space_id.eq(space.id))
        .filter(user_id.eq(user.id))
        .first::<SpaceUser>(&conn)?;

    let _count = delete(asset_contents.find(space_name.id)).execute(&conn)?;
    Ok(Response::new(true, "File deleted successfully".to_string()))
}
