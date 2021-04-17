use crate::auth;
use crate::diesel::QueryDsl;
use crate::diesel::RunQueryDsl;
use crate::handlers::types::*;
use crate::model::{NewUserTask, Space, SpaceUser, User, UserTask};
use crate::schema::spaces::dsl::*;
use crate::schema::spaces_users::dsl::space_id as space_space_id;
use crate::schema::spaces_users::dsl::user_id as space_user_id;
use crate::schema::spaces_users::dsl::*;
use crate::schema::user_tasks::dsl::user_id as task_user_id;
use crate::schema::user_tasks::dsl::*;
use crate::schema::users::dsl::*;
use crate::Pool;

use actix_web::web;
use diesel::dsl::{delete, insert_into};
use diesel::prelude::*;

pub fn add_user_to_task_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<AddUserToFolderPath>,
    item: web::Json<AddUserToFoldr>,
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
        .filter(space_space_id.eq(space.id))
        .filter(space_user_id.eq(user.id))
        .first::<SpaceUser>(&conn)?;
    if !spaces_user.admin_status {
        return Ok(Response::new(
            false,
            "Only admin allowed to assign task".to_string(),
        ));
    }
    for a in item.id.iter() {
        //check if user is already in task
        let user_task = user_tasks
            .filter(task_id.eq(&space_name.id))
            .filter(task_user_id.eq(&a))
            .first::<UserTask>(&conn);

        match user_task {
            Ok(_user_task_exist) => println!("user already assigned to task"),
            Err(diesel::result::Error::NotFound) => {
                //assign user to task if they haven't already ben
                let new_user_task = NewUserTask {
                    user_id: &a,
                    task_id: &space_name.id,
                };

                let _project_task = insert_into(user_tasks)
                    .values(&new_user_task)
                    .execute(&conn)?;
            }
            _ => println!("an error occured"),
        }
    }
    Ok(Response::new(true, "users assigned to task".to_string()))
}

pub fn remove_user_from_task_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<AddUserToFolderPath>,
    item: web::Json<DeleteMailList>,
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
        .filter(space_space_id.eq(space.id))
        .filter(space_user_id.eq(user.id))
        .first::<SpaceUser>(&conn)?;
    if !spaces_user.admin_status {
        return Ok(Response::new(
            false,
            "Only admin allowed to assign task".to_string(),
        ));
    }

    let _count = delete(
        user_tasks
            .filter(task_id.eq(space_name.id))
            .filter(task_user_id.eq(item.id)),
    )
    .execute(&conn)?;
    Ok(Response::new(true, "user removed successfully".to_string()))
}
