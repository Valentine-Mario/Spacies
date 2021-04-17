use crate::auth;
use crate::diesel::QueryDsl;
use crate::diesel::RunQueryDsl;
use crate::handlers::types::*;
use crate::helpers::aws;
use crate::model::{Space, SpaceUser, User};
use crate::schema::spaces::dsl::*;
#[allow(unused_imports)]
use crate::schema::spaces_channel::dsl::space_id as channel_space_id;
use crate::schema::spaces_users::dsl::space_id;
use crate::schema::spaces_users::dsl::*;
use crate::schema::users::dsl::*;
use crate::Pool;

use crate::controllers::space_controller::*;
use actix_files::NamedFile;
use actix_multipart::Multipart;
use actix_web::{web, Error, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use diesel::prelude::*;
use futures::{StreamExt, TryStreamExt};
use std::fs;
use std::fs::remove_file;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

//http responses
pub async fn add_space(
    db: web::Data<Pool>,
    item: web::Json<CreateSpace>,
    auth: BearerAuth,
) -> Result<HttpResponse, Error> {
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(
                    web::block(move || add_space_db(db, item, auth.token().to_string()))
                        .await
                        .map(|response| HttpResponse::Ok().json(response))
                        .map_err(|_| {
                            HttpResponse::Ok()
                                .json(Response::new(false, "some error ocuured".to_string()))
                        })?,
                )
            } else {
                Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
            }
        }
        Err(_) => Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))),
    }
}

pub async fn update_space_logo(
    db: web::Data<Pool>,
    mut payload: Multipart,
    token: BearerAuth,
    space_name: web::Path<PathInfo>,
) -> Result<HttpResponse, Error> {
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

                let space = spaces
                    .filter(spaces_name.ilike(&space_name.info))
                    .first::<Space>(&conn);

                match space {
                    Ok(space) => {
                        let spaces_user: SpaceUser = spaces_users
                            .filter(space_id.eq(space.id))
                            .filter(user_id.eq(user.id))
                            .first::<SpaceUser>(&conn)
                            .unwrap();

                        if !spaces_user.admin_status {
                            return Ok(HttpResponse::Ok().json(Response::new(
                                false,
                                "Only admin is permitted to update this space logo".to_string(),
                            )));
                        };

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
                            let filepath =
                                format!("./tmp/{}", sanitize_filename::sanitize(&filename));

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
                                        let _space_details = diesel::update(spaces.find(space.id))
                                            .set(spaces_img.eq(&file_link))
                                            .execute(&conn);
                                        return Ok(
                                            HttpResponse::Ok().json(Response::new(true, file_link))
                                        );
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
                    }
                    _ => {
                        return Ok(HttpResponse::Ok()
                            .json(Response::new(false, "Space not found".to_string())))
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

pub async fn update_space(
    db: web::Data<Pool>,
    item: web::Json<CreateSpace>,
    auth: BearerAuth,
    space_name: web::Path<PathInfo>,
) -> Result<HttpResponse, Error> {
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(web::block(move || {
                    update_space_db(db, item, auth.token().to_string(), space_name)
                })
                .await
                .map(|response| HttpResponse::Ok().json(response))
                .map_err(|_| {
                    HttpResponse::Ok()
                        .json(Response::new(false, "Space name already exist".to_string()))
                })?)
            } else {
                Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
            }
        }
        Err(_) => Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))),
    }
}

pub async fn get_space(
    db: web::Data<Pool>,
    auth: BearerAuth,
    space_name: web::Path<PathInfo>,
) -> Result<HttpResponse, Error> {
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(
                    web::block(move || get_space_db(db, auth.token().to_string(), space_name))
                        .await
                        .map(|response| HttpResponse::Ok().json(response))
                        .map_err(|_| {
                            HttpResponse::Ok().json(Response::new(
                                false,
                                "Unauthorized to view space".to_string(),
                            ))
                        })?,
                )
            } else {
                Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
            }
        }
        Err(_) => Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))),
    }
}

pub async fn get_user_space(db: web::Data<Pool>, auth: BearerAuth) -> Result<HttpResponse, Error> {
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(
                    web::block(move || get_user_space_db(db, auth.token().to_string()))
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

pub async fn add_invited_user(
    db: web::Data<Pool>,
    item: web::Json<CreateUser>,
    query: web::Query<QueryInfo>,
) -> Result<HttpResponse, Error> {
    match auth::validate_token(&query.token.to_string()) {
        Ok(res) => {
            if res == true {
                Ok(web::block(move || add_invited_user_db(db, item, query))
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

pub async fn invite_user(
    db: web::Data<Pool>,
    item: web::Json<InviteToSpace>,
    auth: BearerAuth,
    space_name: web::Path<PathInfo>,
) -> Result<HttpResponse, Error> {
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(web::block(move || {
                    invite_user_db(db, item, auth.token().to_string(), space_name)
                })
                .await
                .map(|response| HttpResponse::Ok().json(response))
                .map_err(|_| {
                    HttpResponse::Ok().json(Response::new(false, "Space not found".to_string()))
                })?)
            } else {
                Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
            }
        }
        Err(_) => Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))),
    }
}

pub async fn invite_page(_item: web::Query<QueryInfo>) -> Result<NamedFile, ()> {
    let success_file = format!("./pages/invite.html");
    Ok(NamedFile::open(success_file).unwrap())
}

pub async fn get_user_space_status(
    db: web::Data<Pool>,
    auth: BearerAuth,
    space_name: web::Path<PathInfo>,
) -> Result<HttpResponse, Error> {
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(web::block(move || {
                    get_user_space_status_db(db, auth.token().to_string(), space_name)
                })
                .await
                .map(|response| HttpResponse::Ok().json(response))
                .map_err(|_| {
                    HttpResponse::Ok()
                        .json(Response::new(false, "Error getting status".to_string()))
                })?)
            } else {
                Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
            }
        }
        Err(_) => Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))),
    }
}

pub async fn remove_user_from_space(
    db: web::Data<Pool>,
    auth: BearerAuth,
    space_name: web::Path<PathInfo>,
    item: web::Json<UserIdStruct>,
) -> Result<HttpResponse, Error> {
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(web::block(move || {
                    remove_user_from_space_db(db, auth.token().to_string(), space_name, item)
                })
                .await
                .map(|response| HttpResponse::Ok().json(response))
                .map_err(|_| {
                    HttpResponse::Ok().json(Response::new(
                        false,
                        "Error getting space details".to_string(),
                    ))
                })?)
            } else {
                Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
            }
        }
        Err(_) => Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))),
    }
}

pub async fn leave_space(
    db: web::Data<Pool>,
    auth: BearerAuth,
    space_name: web::Path<PathInfo>,
) -> Result<HttpResponse, Error> {
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(
                    web::block(move || leave_space_db(db, auth.token().to_string(), space_name))
                        .await
                        .map(|response| HttpResponse::Ok().json(response))
                        .map_err(|_| {
                            HttpResponse::Ok().json(Response::new(
                                false,
                                "Error getting space details".to_string(),
                            ))
                        })?,
                )
            } else {
                Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
            }
        }
        Err(_) => Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))),
    }
}

pub async fn change_user_priviledge_status(
    db: web::Data<Pool>,
    auth: BearerAuth,
    space_name: web::Path<PathInfo>,
    item: web::Json<PriviledgeStruct>,
) -> Result<HttpResponse, Error> {
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(web::block(move || {
                    change_user_priviledge_status_db(db, auth.token().to_string(), space_name, item)
                })
                .await
                .map(|response| HttpResponse::Ok().json(response))
                .map_err(|_| {
                    HttpResponse::Ok().json(Response::new(
                        false,
                        "Error getting space details".to_string(),
                    ))
                })?)
            } else {
                Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
            }
        }
        Err(_) => Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))),
    }
}

pub async fn get_users_in_space(
    db: web::Data<Pool>,
    space_name: web::Path<PathInfo>,
) -> Result<HttpResponse, Error> {
    Ok(web::block(move || get_users_in_space_db(db, space_name))
        .await
        .map(|response| HttpResponse::Ok().json(response))
        .map_err(|_| {
            HttpResponse::Ok().json(Response::new(
                false,
                "Error getting space details".to_string(),
            ))
        })?)
}
