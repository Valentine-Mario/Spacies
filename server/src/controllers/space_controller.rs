use crate::auth;
use crate::diesel::QueryDsl;
use crate::diesel::RunQueryDsl;
use crate::handlers::types::*;
use crate::helpers::{bcrypt, email, email_template};
use crate::model::{
    NewChannelUser, NewSpace, NewSpaceChannel, NewSpaceUser, NewUser, Space, SpaceChannel,
    SpaceUser, User,
};
use crate::schema::channel_users::dsl::channel_users;
use crate::schema::spaces::dsl::*;
#[allow(unused_imports)]
use crate::schema::spaces_channel::dsl::space_id as channel_space_id;
use crate::schema::spaces_channel::dsl::*;
use crate::schema::spaces_users::dsl::space_id;
use crate::schema::spaces_users::dsl::*;
use crate::schema::users::dsl::*;
use crate::Pool;

use actix_web::web;
use diesel::dsl::{delete, insert_into};
use diesel::prelude::*;
use tokio::task;

pub fn get_users_in_space_db(
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

pub fn change_user_priviledge_status_db(
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

pub fn leave_space_db(
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

pub fn remove_user_from_space_db(
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

pub fn get_user_space_status_db(
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

pub fn add_invited_user_db(
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
    //get general space
    let general_channel: SpaceChannel = spaces_channel
        .filter(channel_space_id.eq(space.id))
        .filter(channel_name.ilike("General"))
        .first::<SpaceChannel>(&conn)?;

    let new_channel_user = NewChannelUser {
        space_channel_id: &general_channel.id,
        space_id: &space.id,
        user_id: &res.id,
        channel_admin: &false,
    };

    //add new user to gnetal channel
    let _new_space_channel = insert_into(channel_users)
        .values(&new_channel_user)
        .execute(&conn)?;
    return Ok(Response::new(
        true,
        "Space account created successfully".to_string(),
    ));
}

pub fn invite_user_db(
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
    //get general space
    let general_channel: SpaceChannel = spaces_channel
        .filter(channel_space_id.eq(space.id))
        .filter(channel_name.ilike("General"))
        .first::<SpaceChannel>(&conn)?;

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

                        let new_channel_user = NewChannelUser {
                            space_channel_id: &general_channel.id,
                            space_id: &space.id,
                            user_id: &user.id,
                            channel_admin: &false,
                        };

                        //add new user to gnetal channel
                        let _new_space_channel = insert_into(channel_users)
                            .values(&new_channel_user)
                            .execute(&conn)?;

                        //send user email confirming the action
                        let email_body = email_template::added_user(&space.spaces_name);
                        let other_email_address =
                            std::env::var("EMAIL_ADDRESS").expect("EMAIL ADDRESS not set");
                        let other_email_password =
                            std::env::var("EMAIL_PASSWORD").expect("EMAIL PASSWORD not set");
                        let other_email_provider =
                            std::env::var("EMAIL_PROVIDER").expect("EMAIL PROVIDER not set");
                        task::spawn(async move {
                            email::send_email(
                                &user.email,
                                &user.username,
                                &"Added to new space".to_string(),
                                &email_body,
                                &other_email_address,
                                &other_email_password,
                                &other_email_provider,
                            )
                        });
                    }
                }
            }
            Err(diesel::result::Error::NotFound) => {
                //send invite email if user is not found in spaces
                let mail_token = auth::create_token(&space.id.to_string(), 1).unwrap();

                let email_body = email_template::invite_user(&space.spaces_name, &mail_token);
                let other_email_address =
                    std::env::var("EMAIL_ADDRESS").expect("EMAIL ADDRESS not set");
                let other_email_password =
                    std::env::var("EMAIL_PASSWORD").expect("EMAIL PASSWORD not set");
                let other_email_provider =
                    std::env::var("EMAIL_PROVIDER").expect("EMAIL PROVIDER not set");
                let new_user_email = user_email.clone();
                task::spawn(async move {
                    email::send_email(
                        &new_user_email,
                        &"Spacer".to_string(),
                        &"Invite to join a Space".to_string(),
                        &email_body,
                        &other_email_address,
                        &other_email_password,
                        &other_email_provider,
                    )
                });
            }
            _ => println!("error"),
        }
    }

    Ok(Response::new(true, "invite sent successfully".to_string()))
}

pub fn get_user_space_db(
    db: web::Data<Pool>,
    token: String,
) -> Result<Response<Vec<(SpaceUser, Space)>>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let decoded_token = auth::decode_token(&token);
    let user = users
        .find(decoded_token.parse::<i32>().unwrap())
        .first::<User>(&conn)?;
    let user_spaces: Vec<(SpaceUser, Space)> = SpaceUser::belonging_to(&user)
        .inner_join(spaces)
        .load::<(SpaceUser, Space)>(&conn)?;
    Ok(Response::new(true, user_spaces))
}

pub fn get_space_db(
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

pub fn update_space_db(
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

pub fn add_space_db(
    db: web::Data<Pool>,
    item: web::Json<CreateSpace>,
    token: String,
) -> Result<OptionalResponse<String, Space>, diesel::result::Error> {
    let decoded_token = auth::decode_token(&token);
    let conn = db.get().unwrap();
    //get user details
    let user: User = users
        .find(decoded_token.parse::<i32>().unwrap())
        .first(&conn)?;
    if !user.verified {
        return Ok(OptionalResponse::new(false,
                 Some("only verified users are allowed to add space. Click on the verification link sent to your email or request a new verification link".to_string()),
                 None));
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
            return Ok(OptionalResponse::new(
                false,
                Some("space name is already taken. Please select a new name".to_string()),
                None,
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

            //get general space
            let general_channel: SpaceChannel = spaces_channel
                .filter(channel_space_id.eq(space.id))
                .filter(channel_name.ilike("General"))
                .first::<SpaceChannel>(&conn)?;

            let new_channel_user = NewChannelUser {
                space_channel_id: &general_channel.id,
                space_id: &space.id,
                user_id: &user.id,
                channel_admin: &true,
            };
            //add user to gneral channel as an admin
            let _new_space_channel = insert_into(channel_users)
                .values(&new_channel_user)
                .execute(&conn)?;

            return Ok(OptionalResponse::new(
                true,
                Some("space created successfully".to_string()),
                Some(space),
            ));
        }
        _ => {
            return Ok(OptionalResponse::new(
                false,
                Some("some error occured".to_string()),
                None,
            ))
        }
    }
}
