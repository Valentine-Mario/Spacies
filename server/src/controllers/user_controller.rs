use crate::auth;
use crate::diesel::QueryDsl;
use crate::diesel::RunQueryDsl;
use crate::handlers::types::*;
use crate::helpers::{bcrypt, email, email_template};
use crate::model::{NewUser, User};
use crate::schema::users::dsl::*;
use crate::Pool;

use actix_web::web;
use diesel::dsl::insert_into;
use diesel::prelude::*;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use tokio::task;

pub fn forgot_password_db(
    db: web::Data<Pool>,
    item: web::Json<ForgotPassword>,
) -> Result<Response<String>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let user_details = users.filter(email.ilike(&item.email)).first::<User>(&conn);
    match user_details {
        Ok(user) => {
            //run email sending as a background job
            task::spawn(async move {
                //generate random string
                let rand_string: String =
                    thread_rng().sample_iter(&Alphanumeric).take(30).collect();

                let email_template = email_template::forgot_password_email(&rand_string);
                let other_email_address =
                    std::env::var("EMAIL_ADDRESS").expect("EMAIL ADDRESS not set");
                let other_email_password =
                    std::env::var("EMAIL_PASSWORD").expect("EMAIL PASSWORD not set");
                let other_email_provider =
                    std::env::var("EMAIL_PROVIDER").expect("EMAIL PROVIDER not set");

                email::send_email(
                    &user.email,
                    &user.username,
                    &"Password Reset".to_string(),
                    &email_template,
                    &other_email_address,
                    &other_email_password,
                    &other_email_provider,
                );
                let hashed = bcrypt::encrypt_password(&rand_string);
                let _updates = diesel::update(users.find(user.id))
                    .set(user_password.eq(&hashed))
                    .execute(&conn);
            });
            return Ok(Response::new(
                true,
                "Reset email sent successfully".to_string(),
            ));
        }
        Err(diesel::result::Error::NotFound) => {
            Ok(Response::new(false, "email not found".to_string()))
        }
        _ => Ok(Response::new(false, "error geting data".to_string())),
    }
}

pub fn update_password_db(
    db: web::Data<Pool>,
    item: web::Json<UpdatePassword>,
    token: String,
) -> Result<Response<String>, diesel::result::Error> {
    let decoded_token = auth::decode_token(&token);
    let conn = db.get().unwrap();
    let user: User = users
        .find(decoded_token.parse::<i32>().unwrap())
        .first(&conn)?;
    let cmp_pwd = bcrypt::compare_password(&user.user_password, &item.old_password);
    if cmp_pwd {
        if &item.new_password.chars().count() < &6 {
            return Ok(Response::new(
                false,
                "password should be min of 6 characters".to_string(),
            ));
        }
        let hashed = bcrypt::encrypt_password(&item.new_password);
        let _updates = diesel::update(users.find(user.id))
            .set(user_password.eq(&hashed))
            .execute(&conn);
        return Ok(Response::new(
            true,
            "password updated successfully".to_string(),
        ));
    } else {
        return Ok(Response::new(false, "incorrect password".to_string()));
    }
}

pub fn updat_name_db(
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

pub fn get_profile_db(
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

pub fn login_db(
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

pub fn resend_verification_db(
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
        //send email as a background job
        task::spawn(async move {
            //send user verification email
            let mail_token = auth::create_token(&user.id.to_string(), 1).unwrap();
            let email_template = email_template::verification_email(&mail_token);
            let other_email_address =
                std::env::var("EMAIL_ADDRESS").expect("EMAIL ADDRESS not set");
            let other_email_password =
                std::env::var("EMAIL_PASSWORD").expect("EMAIL PASSWORD not set");
            let other_email_provider =
                std::env::var("EMAIL_PROVIDER").expect("EMAIL PROVIDER not set");

            email::send_email(
                &user.email,
                &user.username,
                &"Welcome To Spacies".to_string(),
                &email_template,
                &other_email_address,
                &other_email_password,
                &other_email_provider,
            );
        });

        return Ok(Response::new(
            true,
            "verification email sent successfully".to_string(),
        ));
    }
}

pub fn verify_user_db(
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

pub fn add_user_db(
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
    let new_user_id = res.id;
    //send email as a new background task
    task::spawn(async move {
        //send user verification email
        let mail_token = auth::create_token(&res.id.to_string(), 1).unwrap();
        let other_email_address = std::env::var("EMAIL_ADDRESS").expect("EMAIL ADDRESS not set");
        let other_email_password = std::env::var("EMAIL_PASSWORD").expect("EMAIL PASSWORD not set");
        let other_email_provider = std::env::var("EMAIL_PROVIDER").expect("EMAIL PROVIDER not set");

        let email_template = email_template::verification_email(&mail_token);
        email::send_email(
            &res.email,
            &res.username,
            &"Welcome To Spacies".to_string(),
            &email_template,
            &other_email_address,
            &other_email_password,
            &other_email_provider,
        );
    });

    //create user token
    match auth::create_token(&new_user_id.to_string(), 30) {
        Ok(token) => return Ok(Response::new(true, token.to_string())),
        _ => Ok(Response::new(false, "error creating token".to_string())),
    }
}
