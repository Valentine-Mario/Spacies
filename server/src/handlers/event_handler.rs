use crate::auth;
use crate::diesel::QueryDsl;
use crate::diesel::RunQueryDsl;
use crate::handlers::types::*;
use crate::model::{Event, NewEvent, Space, SpaceUser, User};
use crate::schema::events::dsl::space_id as event_space_id;
use crate::schema::events::dsl::*;
use crate::schema::spaces::dsl::*;
use crate::schema::spaces_users::dsl::space_id as space_user_id;
use crate::schema::spaces_users::dsl::*;
use crate::schema::users::dsl::*;
use crate::Pool;
use chrono::prelude::*;

use actix_web::{web, Error, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use diesel::dsl::{delete, insert_into};
use diesel::prelude::*;

//http calls
pub async fn create_event(
    db: web::Data<Pool>,
    auth: BearerAuth,
    space_name: web::Path<PathInfo>,
    item: web::Json<AddEvent>,
) -> Result<HttpResponse, Error> {
    match auth::validate_token(&auth.token().to_string()) {
        Ok(res) => {
            if res == true {
                Ok(web::block(move || {
                    create_event_db(db, auth.token().to_string(), space_name, item)
                })
                .await
                .map(|response| HttpResponse::Ok().json(response))
                .map_err(|_| {
                    HttpResponse::Ok()
                        .json(Response::new(false, "Error creating event".to_string()))
                })?)
            } else {
                Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string())))
            }
        }
        Err(_) => Ok(HttpResponse::Ok().json(ResponseError::new(false, "jwt error".to_string()))),
    }
}

//db calls
fn get_event_details_db() {}

fn delete_event_db() {}

fn search_event_db() {}

fn edit_event_db() {}

fn get_events_db() {}

fn create_event_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<PathInfo>,
    item: web::Json<AddEvent>,
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
        .filter(space_user_id.eq(space.id))
        .filter(user_id.eq(user.id))
        .first::<SpaceUser>(&conn)?;

    let space_event: Vec<String> = events
        .filter(event_space_id.eq(&space.id))
        .select(event_name)
        .load::<String>(&conn)?;
    if space_event
        .iter()
        .any(|i| &i.to_lowercase() == &item.event_name.to_lowercase())
    {
        return Ok(Response::new(
            false,
            "A similar event name alreadt exist for this space".to_string(),
        ));
    }
    let dt: NaiveDateTime =
        NaiveDate::from_ymd(item.year, item.event_date[0], item.event_date[1]).and_hms(0, 0, 0);
    let new_event = NewEvent {
        event_name: &item.event_name,
        event_description: &item.event_description,
        reminded: &false,
        event_date: dt,
        space_id: &space.id,
    };

    let _space_event = insert_into(events).values(&new_event).execute(&conn)?;

    Ok(Response::new(
        true,
        "Event created successfully".to_string(),
    ))
}
