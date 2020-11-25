use crate::auth;
use crate::diesel::QueryDsl;
use crate::diesel::RunQueryDsl;
use crate::handlers::types::*;
use crate::helpers::{aws, bcrypt, email, email_template};
use crate::model::{NewSpace, NewSpaceChannel, NewSpaceUser, NewUser, Space, SpaceUser, User};
use crate::schema::spaces::dsl::*;
#[allow(unused_imports)]
use crate::schema::spaces_channel::dsl::space_id as channel_space_id;
use crate::schema::spaces_channel::dsl::*;
use crate::schema::spaces_users::dsl::space_id;
use crate::schema::spaces_users::dsl::*;
use crate::schema::users::dsl::*;
use crate::Pool;

use actix_files::NamedFile;
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

//db calls
fn get_users_in_space_db(
    db: web::Data<Pool>,
    space_name: web::Path<PathInfo>,
) -> Result<Response<Vec<(SpaceUser, User)>>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let space = spaces
        .filter(spaces_name.ilike(&space_name.info))
        .first::<Space>(&conn)?;
    let user_spaces: Vec<_> = SpaceUser::belonging_to(&space)
        .inner_join(users)
        .load::<(SpaceUser, User)>(&conn)?;
    Ok(Response::new(true, user_spaces))
}

fn change_user_priviledge_status_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<PathInfo>,
    item: web::Json<PriviledgeStruct>,
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
            "Only admin is permitted to change a user priviledge status".to_string(),
        ));
    }
    //get details of user to be kicked out
    let update_user: SpaceUser = spaces_users
        .filter(space_id.eq(space.id))
        .filter(user_id.eq(item.user))
        .first::<SpaceUser>(&conn)?;

    let _space_details = diesel::update(spaces_users.find(update_user.id))
        .set(admin_status.eq(&item.admin))
        .execute(&conn)?;
    Ok(Response::new(
        true,
        "priviledge status changed successfully".to_string(),
    ))
}

fn leave_space_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<PathInfo>,
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
    let _delete_user = delete(spaces_users.find(spaces_user.id)).execute(&conn)?;
    Ok(Response::new(
        true,
        "Removed from Space successfully".to_string(),
    ))
}

fn remove_user_from_space_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<PathInfo>,
    item: web::Json<UserIdStruct>,
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
            "Only admin is permitted to kick user out of this space".to_string(),
        ));
    }
    //get details of user to be kicked out
    let kicked_out_user: SpaceUser = spaces_users
        .filter(space_id.eq(space.id))
        .filter(user_id.eq(item.user))
        .first::<SpaceUser>(&conn)?;
    let _delete_user = delete(spaces_users.find(kicked_out_user.id)).execute(&conn)?;
    Ok(Response::new(
        true,
        "user kicked out successfully".to_string(),
    ))
}

fn get_user_space_status_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<PathInfo>,
) -> Result<Response<bool>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let decoded_token = auth::decode_token(&token);
    let user = users
        .find(decoded_token.parse::<i32>().unwrap())
        .first::<User>(&conn)?;
    let space = spaces
        .filter(spaces_name.ilike(&space_name.info))
        .first::<Space>(&conn)?;

    let space_user_status = spaces_users
        .filter(space_id.eq(space.id))
        .filter(user_id.eq(user.id))
        .select(admin_status)
        .first(&conn)?;
    Ok(Response::new(true, space_user_status))
}

fn add_invited_user_db(
    db: web::Data<Pool>,
    item: web::Json<CreateUser>,
    query: web::Query<QueryInfo>,
) -> Result<Response<String>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let decoded_token = auth::decode_token(&query.token);
    let space = spaces
        .find(decoded_token.parse::<i32>().unwrap())
        .first::<Space>(&conn)?;
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
        verified: &true,
        created_at: chrono::Local::now().naive_local(),
    };
    let res: User = insert_into(users).values(&new_user).get_result(&conn)?;
    let new_space_user = NewSpaceUser {
        user_id: &res.id,
        space_id: &space.id,
        admin_status: &false,
    };
    let _space_user: SpaceUser = insert_into(spaces_users)
        .values(&new_space_user)
        .get_result(&conn)?;
    return Ok(Response::new(
        true,
        "Space account created successfully".to_string(),
    ));
}

fn invite_user_db(
    db: web::Data<Pool>,
    item: web::Json<InviteToSpace>,
    token: String,
    space_name: web::Path<PathInfo>,
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
            "Only admin is permitted to invite users to this space".to_string(),
        ));
    };
    //loop through the vec of invite
    for user_email in item.email.iter() {
        //check if user already exist
        let user_details = users.filter(email.ilike(&user_email)).first::<User>(&conn);
        match user_details {
            Ok(user) => {
                //check if user is already in space
                let spaces_user_def = spaces_users
                    .filter(space_id.eq(space.id))
                    .filter(user_id.eq(user.id))
                    .first::<SpaceUser>(&conn);

                match spaces_user_def {
                    Ok(_spc_usr) => {
                        //do nothing if user is already a member of this space
                    }
                    _ => {
                        //if user exist, automatically add them
                        let new_space_user = NewSpaceUser {
                            user_id: &user.id,
                            space_id: &space.id,
                            admin_status: &false,
                        };
                        let _space_user: SpaceUser = insert_into(spaces_users)
                            .values(&new_space_user)
                            .get_result(&conn)?;
                        //send user email confirming the action
                        let email_body = email_template::added_user(&space.spaces_name);
                        email::send_email(
                            &user.email,
                            &user.username,
                            &"Added to new space".to_string(),
                            &email_body,
                        )
                    }
                }
            }
            Err(diesel::result::Error::NotFound) => {
                //send invite email if user is not found in spaces
                let mail_token = auth::create_token(&space.id.to_string(), 1).unwrap();
                let email_body = email_template::invite_user(&space.spaces_name, &mail_token);
                email::send_email(
                    &user_email,
                    &"Spacer".to_string(),
                    &"Invite to join a Space".to_string(),
                    &email_body,
                )
            }
            _ => println!("error"),
        }
    }

    Ok(Response::new(true, "invite sent successfully".to_string()))
}

fn get_user_space_db(
    db: web::Data<Pool>,
    token: String,
) -> Result<Response<Vec<(SpaceUser, Space)>>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let decoded_token = auth::decode_token(&token);
    let user = users
        .find(decoded_token.parse::<i32>().unwrap())
        .first::<User>(&conn)?;
    let user_spaces: Vec<_> = SpaceUser::belonging_to(&user)
        .inner_join(spaces)
        .load::<(SpaceUser, Space)>(&conn)?;
    Ok(Response::new(true, user_spaces))
}

fn get_space_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<PathInfo>,
) -> Result<Response<Space>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let decoded_token = auth::decode_token(&token);
    let user: User = users
        .find(decoded_token.parse::<i32>().unwrap())
        .first(&conn)?;
    let space_details = spaces
        .filter(spaces_name.ilike(&space_name.info))
        .first::<Space>(&conn)?;
    let _spaces_user: SpaceUser = spaces_users
        .filter(space_id.eq(space_details.id))
        .filter(user_id.eq(user.id))
        .first::<SpaceUser>(&conn)?;
    Ok(Response::new(true, space_details))
}

fn update_space_db(
    db: web::Data<Pool>,
    item: web::Json<CreateSpace>,
    token: String,
    space_name: web::Path<PathInfo>,
) -> Result<Response<String>, diesel::result::Error> {
    let decoded_token = auth::decode_token(&token);
    let conn = db.get().unwrap();
    //get user details
    let user: User = users
        .find(decoded_token.parse::<i32>().unwrap())
        .first(&conn)?;
    let space = spaces
        .filter(spaces_name.ilike(&space_name.info))
        .first::<Space>(&conn);
    match space {
        Ok(space) => {
            let spaces_user: SpaceUser = spaces_users
                .filter(space_id.eq(space.id))
                .filter(user_id.eq(user.id))
                .first::<SpaceUser>(&conn)?;

            if !spaces_user.admin_status {
                return Ok(Response::new(
                    false,
                    "Only admin is permitted to update this space details".to_string(),
                ));
            };
            let _space_details = diesel::update(spaces.find(space.id))
                .set((
                    spaces_name.eq(&item.spaces_name),
                    spaces_desc.eq(&item.spaces_desc),
                ))
                .execute(&conn)?;
            return Ok(Response::new(
                true,
                "Space updated successfully".to_string(),
            ));
        }
        Err(diesel::result::Error::NotFound) => {
            Ok(Response::new(false, "space not found".to_string()))
        }
        _ => Ok(Response::new(false, "error getting space".to_string())),
    }
}

fn add_space_db(
    db: web::Data<Pool>,
    item: web::Json<CreateSpace>,
    token: String,
) -> Result<Response<String>, diesel::result::Error> {
    let decoded_token = auth::decode_token(&token);
    let conn = db.get().unwrap();
    //get user details
    let user: User = users
        .find(decoded_token.parse::<i32>().unwrap())
        .first(&conn)?;
    if !user.verified {
        return Ok(Response::new(false,
                 "only verified users are allowed to add space. Click on the verification link sent to your email or request a new verification link".to_string()));
    }
    //add space
    let new_space = NewSpace {
        spaces_name: &item.spaces_name,
        spaces_desc: &item.spaces_desc,
        created_at: chrono::Local::now().naive_local(),
    };
    //check if space name already exist
    let space_details = spaces
        .filter(spaces_name.ilike(&item.spaces_name))
        .first::<Space>(&conn);
    match space_details {
        Ok(_space) => {
            return Ok(Response::new(
                false,
                "space name is already taken. Please select a new name".to_string(),
            ))
        }
        Err(diesel::result::Error::NotFound) => {
            //if space does not exist create space
            let space: Space = insert_into(spaces).values(&new_space).get_result(&conn)?;
            //add user to space as an admin
            let new_space_user = NewSpaceUser {
                user_id: &user.id,
                space_id: &space.id,
                admin_status: &true,
            };
            let _space_user: SpaceUser = insert_into(spaces_users)
                .values(&new_space_user)
                .get_result(&conn)?;

            //add default space channel on creation
            let default_space_channel = NewSpaceChannel {
                space_id: &space.id,
                channel_name: &"General".to_string(),
            };
            let _space_channel = insert_into(spaces_channel)
                .values(&default_space_channel)
                .execute(&conn)?;

            return Ok(Response::new(
                true,
                "space created successfully".to_string(),
            ));
        }
        _ => return Ok(Response::new(false, "some error occured".to_string())),
    }
}
