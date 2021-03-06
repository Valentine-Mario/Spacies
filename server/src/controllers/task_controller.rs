use crate::auth;
use crate::diesel::QueryDsl;
use crate::diesel::RunQueryDsl;
use crate::handlers::types::*;
use crate::model::{NewTask, Project, Space, SpaceUser, Task, User, UserTask};
use crate::schema::projects::dsl::space_id as project_space_id;
use crate::schema::projects::dsl::*;
use crate::schema::spaces::dsl::*;
use crate::schema::spaces_users::dsl::space_id as space_space_id;
use crate::schema::spaces_users::dsl::user_id as space_user_id;
use crate::schema::spaces_users::dsl::*;
use crate::schema::tasks::dsl::project_id as task_project_id;
use crate::schema::tasks::dsl::*;
use crate::schema::user_tasks::dsl::*;
use crate::schema::users::dsl::*;
use crate::Pool;
use chrono::prelude::*;

use actix_web::web;
use diesel::dsl::{delete, insert_into};
use diesel::prelude::*;

pub fn create_task_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<AddUserToFolderPath>,
    item: web::Json<AddTask>,
) -> Result<OptionalResponse<String, Task>, diesel::result::Error> {
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
        return Ok(OptionalResponse::new(
            false,
            Some("Only admin allowed to add task".to_string()),
            None,
        ));
    }
    //yyyy/mm/dd
    let dt: NaiveDateTime =
        NaiveDate::from_ymd(item.year, item.due_date[0], item.due_date[1]).and_hms(0, 0, 0);
    let new_task = NewTask {
        task_name: &item.task_name,
        task_description: &item.task_description,
        project_id: &space_name.id,
        task_status: &"undone",
        due_date: dt,
    };

    let project_task: Task = insert_into(tasks).values(&new_task).get_result(&conn)?;

    Ok(OptionalResponse::new(
        true,
        Some("Task created successfully".to_string()),
        Some(project_task),
    ))
}

pub fn update_task_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<AddUserToFolderPath>,
    item: web::Json<UpdateTask>,
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
            "Only admin allowed to update task".to_string(),
        ));
    }

    let _task_details = diesel::update(tasks.find(space_name.id))
        .set((
            task_name.eq(&item.task_name),
            task_description.eq(&item.task_description),
        ))
        .execute(&conn)?;

    Ok(Response::new(true, "Task updated successfully".to_string()))
}

pub fn delete_task_db(
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
        .filter(space_space_id.eq(space.id))
        .filter(space_user_id.eq(user.id))
        .first::<SpaceUser>(&conn)?;
    if !spaces_user.admin_status {
        return Ok(Response::new(
            false,
            "Only admin allowed to delete task".to_string(),
        ));
    }
    //delete all user task linked to task
    let _count = delete(user_tasks.filter(task_id.eq(space_name.id))).execute(&conn)?;
    //delet task
    let _count2 = delete(tasks.find(space_name.id)).execute(&conn)?;

    Ok(Response::new(true, "task deleted successfully".to_string()))
}

pub fn update_task_status_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<AddUserToFolderPath>,
    item: web::Json<UpdateTaskStatus>,
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
        .filter(space_space_id.eq(space.id))
        .filter(space_user_id.eq(user.id))
        .first::<SpaceUser>(&conn)?;
    let _task_details = diesel::update(tasks.find(space_name.id))
        .set(task_status.eq(&item.task_status))
        .execute(&conn)?;
    Ok(Response::new(true, "task status updated".to_string()))
}

pub fn get_task_in_project_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<ChannelPathInfo>,
) -> Result<Response<(Vec<(UserTask, User)>, Vec<Task>)>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let decoded_token = auth::decode_token(&token);
    let user = users
        .find(decoded_token.parse::<i32>().unwrap())
        .first::<User>(&conn)?;
    let space = spaces
        .filter(spaces_name.ilike(&space_name.info))
        .first::<Space>(&conn)?;
    let _spaces_user: SpaceUser = spaces_users
        .filter(space_space_id.eq(space.id))
        .filter(space_user_id.eq(user.id))
        .first::<SpaceUser>(&conn)?;
    let project_details: Project = projects
        .filter(project_space_id.eq(&space.id))
        .filter(project_name.ilike(&space_name.channel))
        .first::<Project>(&conn)?;

    let project_task: Vec<Task> = tasks
        .filter(task_project_id.eq(project_details.id))
        .load::<Task>(&conn)?;

    //get all task in this project and link users
    let project_tasks: Vec<_> = UserTask::belonging_to(&project_task)
        .inner_join(users)
        .load::<(UserTask, User)>(&conn)?;

    Ok(Response::new(true, (project_tasks, project_task)))
}
