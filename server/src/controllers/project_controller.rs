use crate::auth;
use crate::diesel::QueryDsl;
use crate::diesel::RunQueryDsl;
use crate::handlers::paginate::*;
use crate::handlers::types::*;
use crate::model::{NewProject, Project, Space, SpaceUser, Task, User};
use crate::schema::projects::dsl::id as project_id;
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

use actix_web::web;
use diesel::dsl::{delete, insert_into};
use diesel::prelude::*;

pub fn create_project_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<PathInfo>,
    item: web::Json<AddProject>,
) -> Result<OptionalResponse<String, Project>, diesel::result::Error> {
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
            Some("Only admin authorized to create a project".to_string()),
            None,
        ));
    }

    let all_projects: Vec<String> = projects
        .filter(project_space_id.eq(&space.id))
        .select(project_name)
        .load::<String>(&conn)?;
    if all_projects
        .iter()
        .any(|i| &i.to_lowercase() == &item.project_name.to_lowercase())
    {
        return Ok(OptionalResponse::new(
            false,
            Some("A similar Project name alrady exist".to_string()),
            None,
        ));
    }

    let new_project = NewProject {
        project_name: &item.project_name,
        space_id: &space.id,
        created_at: chrono::Local::now().naive_local(),
    };

    let space_project: Project = insert_into(projects)
        .values(&new_project)
        .get_result(&conn)?;

    Ok(OptionalResponse::new(
        true,
        Some("Project created successfully".to_string()),
        Some(space_project),
    ))
}

pub fn update_project_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<AddUserToFolderPath>,
    item: web::Json<AddProject>,
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
            "Only admin authorized to update a project".to_string(),
        ));
    }

    let all_projects: Vec<String> = projects
        .filter(project_space_id.eq(&space.id))
        .select(project_name)
        .load::<String>(&conn)?;
    if all_projects
        .iter()
        .any(|i| &i.to_lowercase() == &item.project_name.to_lowercase())
    {
        return Ok(Response::new(
            false,
            "A similar project name alrady exist".to_string(),
        ));
    }

    let _projct_details = diesel::update(projects.find(space_name.id))
        .set(project_name.eq(&item.project_name))
        .execute(&conn)?;

    Ok(Response::new(
        true,
        "Project details updated successfullt".to_string(),
    ))
}

pub fn get_all_projects_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<PathInfo>,
    item: web::Query<PaginateQuery>,
) -> Result<Response<(i64, Vec<Project>)>, diesel::result::Error> {
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
    let all_project = projects
        .filter(project_space_id.eq(&space.id))
        .order(project_id.desc())
        .paginate(item.page)
        .per_page(item.per_page)
        .load::<(Project, i64)>(&conn)?;

    let total = all_project.get(0).map(|x| x.1).unwrap_or(0);
    let list: Vec<Project> = all_project.into_iter().map(|x| x.0).collect();
    Ok(Response::new(true, (total, list)))
}

pub fn get_project_details_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<ChannelPathInfo>,
) -> Result<Response<Project>, diesel::result::Error> {
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
    let project_details = projects
        .filter(project_space_id.eq(&space.id))
        .filter(project_name.ilike(&space_name.channel))
        .first::<Project>(&conn)?;
    Ok(Response::new(true, project_details))
}

pub fn search_project_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<MailChannelPathInfo>,
    item: web::Query<PaginateQuery>,
) -> Result<Response<(i64, Vec<Project>)>, diesel::result::Error> {
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

    let a = format!("%{}%", space_name.name);
    let searched = projects
        .filter(project_space_id.eq(&space.id))
        .filter(project_name.ilike(&a))
        .order(project_id.desc())
        .paginate(item.page)
        .per_page(item.per_page)
        .load::<(Project, i64)>(&conn)?;
    let total = searched.get(0).map(|x| x.1).unwrap_or(0);
    let list: Vec<Project> = searched.into_iter().map(|x| x.0).collect();

    Ok(Response::new(true, (total, list)))
}

pub fn delete_project_db(
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
            "Only admin authorized to delte a project".to_string(),
        ));
    }

    //get all task linked with project
    let all_task: Vec<Task> = tasks
        .filter(task_project_id.eq(&space_name.id))
        .load::<Task>(&conn)?;

    //loop through task and delete all user assigned to task
    for a in all_task.iter() {
        let _count = delete(user_tasks.filter(task_id.eq(a.id))).execute(&conn)?;
    }
    //delete all task in project
    let _count2 = delete(tasks.filter(task_project_id.eq(&space_name.id))).execute(&conn)?;
    //delete project
    let _count3 = delete(projects.find(&space_name.id)).execute(&conn)?;

    Ok(Response::new(
        true,
        "project deleted successfully".to_string(),
    ))
}
