use crate::auth;
use crate::diesel::QueryDsl;
use crate::diesel::RunQueryDsl;
use crate::handlers::types::*;
use crate::helpers::{email, email_template, encrypt::decrypt};
use crate::model::{MailList, NewUserMail, Space, SpaceEmail, SpaceUser, User, UserMail};
use crate::schema::maillists::dsl::*;
use crate::schema::spaces::dsl::*;
use crate::schema::spaces_email::dsl::space_id as space_email_id;
use crate::schema::spaces_email::dsl::*;
use crate::schema::spaces_users::dsl::space_id;
use crate::schema::spaces_users::dsl::user_id as space_user_id;
use crate::schema::spaces_users::dsl::*;
use crate::schema::usermails::dsl::user_id as mail_user_id;
use crate::schema::usermails::dsl::*;
use crate::schema::users::dsl::*;
use crate::Pool;

use actix_web::web;
use diesel::dsl::{delete, insert_into};
use diesel::prelude::*;

pub fn send_email_to_general_db(
    db: web::Data<Pool>,
    space_name: web::Path<PathInfo>,
    token: String,
    item: web::Json<SendMail>,
) -> Result<Response<String>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let decoded_token = auth::decode_token(&token);

    let space: Space = spaces
        .filter(spaces_name.ilike(&space_name.info))
        .first::<Space>(&conn)?;
    let user = users
        .find(decoded_token.parse::<i32>().unwrap())
        .first::<User>(&conn)?;

    let _spaces_user: SpaceUser = spaces_users
        .filter(space_id.eq(space.id))
        .filter(space_user_id.eq(user.id))
        .first::<SpaceUser>(&conn)?;

    //get space email cred
    let email_cred = spaces_email
        .filter(space_email_id.eq(space.id))
        .first::<SpaceEmail>(&conn);
    match email_cred {
        Ok(cred_details) => {
            let user_spaces: Vec<(SpaceUser, User)> =
                SpaceUser::belonging_to(&space)
                    .inner_join(users)
                    .load::<(SpaceUser, User)>(&conn)?;
            let pass = decrypt(&cred_details.email_password);
            for a in user_spaces.iter() {
                let template = email_template::notify_folder(&"General".to_string(), &item.body);
                //decrypt password
                email::send_email(
                    &a.1.email,
                    &a.1.username,
                    &item.title,
                    &template,
                    &cred_details.email_address,
                    &pass,
                    &cred_details.email_provider,
                );
            }
            Ok(Response::new(
                true,
                "Email sent successfully to all members".to_string(),
            ))
        }
        Err(diesel::result::Error::NotFound) => {
            return Ok(Response::new(
                false,
                "Please provide email credentials before you use this service".to_string(),
            ))
        }
        _ => return Ok(Response::new(false, "error sending email".to_string())),
    }
}

pub fn send_mail_to_folder_db(
    db: web::Data<Pool>,
    folder_id: web::Path<AddUserToFolderPath>,
    token: String,
    item: web::Json<SendMail>,
) -> Result<Response<String>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let decoded_token = auth::decode_token(&token);

    let space: Space = spaces
        .filter(spaces_name.ilike(&folder_id.info))
        .first::<Space>(&conn)?;
    let user = users
        .find(decoded_token.parse::<i32>().unwrap())
        .first::<User>(&conn)?;

    let _spaces_user: SpaceUser = spaces_users
        .filter(space_id.eq(space.id))
        .filter(space_user_id.eq(user.id))
        .first::<SpaceUser>(&conn)?;

    let mail_list: MailList = maillists.find(folder_id.id).first::<MailList>(&conn)?;
    //get space email cred
    let email_cred = spaces_email
        .filter(space_email_id.eq(space.id))
        .first::<SpaceEmail>(&conn);

    match email_cred {
        Ok(cred_details) => {
            let user_mail: Vec<(UserMail, User)> = UserMail::belonging_to(&mail_list)
                .inner_join(users)
                .load::<(UserMail, User)>(&conn)?;
            let pass = decrypt(&cred_details.email_password);

            for send_user in user_mail.iter() {
                let template = email_template::notify_folder(&mail_list.folder_name, &item.body);
                email::send_email(
                    &send_user.1.email,
                    &send_user.1.username,
                    &item.title,
                    &template,
                    &cred_details.email_address,
                    &pass,
                    &cred_details.email_provider,
                );
            }
            Ok(Response::new(
                true,
                "Email sent successfully to all members".to_string(),
            ))
        }
        Err(diesel::result::Error::NotFound) => {
            return Ok(Response::new(
                false,
                "Please provide email credentials before you use this service".to_string(),
            ))
        }
        _ => return Ok(Response::new(false, "error sending email".to_string())),
    }
}

pub fn remove_user_folder_db(
    db: web::Data<Pool>,
    token: String,
    folder: web::Path<AddUserToFolderPath>,
    item: web::Json<DeleteMailList>,
) -> Result<Response<String>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let decoded_token = auth::decode_token(&token);
    let user = users
        .find(decoded_token.parse::<i32>().unwrap())
        .first::<User>(&conn)?;

    let space: Space = spaces
        .filter(spaces_name.ilike(&folder.info))
        .first::<Space>(&conn)?;

    let spaces_user: SpaceUser = spaces_users
        .filter(space_id.eq(space.id))
        .filter(space_user_id.eq(user.id))
        .first::<SpaceUser>(&conn)?;

    if !spaces_user.admin_status {
        return Ok(Response::new(
            false,
            "only admin allowed to add users to folder".to_string(),
        ));
    }
    let folder: MailList = maillists.find(folder.id).first::<MailList>(&conn)?;

    let _count = delete(
        usermails
            .filter(mail_list_id.eq(folder.id))
            .filter(mail_user_id.eq(&item.id)),
    )
    .execute(&conn)?;

    Ok(Response::new(true, "user removed successfully".to_string()))
}

pub fn add_user_folder_db(
    db: web::Data<Pool>,
    token: String,
    folder: web::Path<AddUserToFolderPath>,
    item: web::Json<AddUserToFoldr>,
) -> Result<Response<String>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let decoded_token = auth::decode_token(&token);
    let user = users
        .find(decoded_token.parse::<i32>().unwrap())
        .first::<User>(&conn)?;

    let space: Space = spaces
        .filter(spaces_name.ilike(&folder.info))
        .first::<Space>(&conn)?;

    let spaces_user: SpaceUser = spaces_users
        .filter(space_id.eq(space.id))
        .filter(space_user_id.eq(user.id))
        .first::<SpaceUser>(&conn)?;

    if !spaces_user.admin_status {
        return Ok(Response::new(
            false,
            "only admin allowed to add users to folder".to_string(),
        ));
    }

    let folder: MailList = maillists.find(folder.id).first::<MailList>(&conn)?;

    for new_user_id in item.id.iter() {
        let user_in_folder = usermails
            .filter(mail_user_id.eq(&new_user_id))
            .filter(mail_list_id.eq(folder.id))
            .first::<UserMail>(&conn);

        match user_in_folder {
            Ok(_user) => {
                //do nothing for user already in folder
            }
            Err(diesel::result::Error::NotFound) => {
                //if user not found
                let new_user = NewUserMail {
                    mail_list_id: &folder.id,
                    user_id: &new_user_id,
                };

                let _res = insert_into(usermails).values(&new_user).execute(&conn)?;
            }
            _ => {
                println!("An error occured");
            }
        }
    }

    Ok(Response::new(true, "Users added successfully".to_string()))
}
