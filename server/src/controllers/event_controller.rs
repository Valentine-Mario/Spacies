use crate::auth;
use crate::diesel::RunQueryDsl;
use crate::handlers::paginate::*;
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

use actix_web::web;
use diesel::dsl::{delete, insert_into};
use diesel::prelude::*;

pub fn get_event_details_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<ChannelPathInfo>,
) -> Result<OptionalResponse<String, Event>, diesel::result::Error> {
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
    let space_event: Event = events
        .filter(event_space_id.eq(&space.id))
        .filter(event_name.ilike(&space_name.channel))
        .first::<Event>(&conn)?;

    Ok(OptionalResponse::new(
        true,
        Some("event details".to_string()),
        Some(space_event),
    ))
}

pub fn delete_event_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<AddUserToFolderPath>,
) -> Result<OptionalResponse<String, String>, diesel::result::Error> {
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
    let _count = delete(events.find(space_name.id)).execute(&conn)?;

    Ok(OptionalResponse::new(
        true,
        Some("Event deleted successfully".to_string()),
        None,
    ))
}

pub fn search_event_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<MailChannelPathInfo>,
    item: web::Query<PaginateQuery>,
) -> Result<OptionalResponse<String, (i64, Vec<Event>)>, diesel::result::Error> {
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

    let a = format!("%{}%", space_name.name);
    let searched_events = events
        .filter(event_space_id.eq(&space.id))
        .filter(event_name.ilike(&a).or(event_description.ilike(&a)))
        .order(event_date.desc())
        .paginate(item.page)
        .per_page(item.per_page)
        .load::<(Event, i64)>(&conn)?;
    let total = searched_events.get(0).map(|x| x.1).unwrap_or(0);
    let list: Vec<Event> = searched_events.into_iter().map(|x| x.0).collect();
    Ok(OptionalResponse::new(
        true,
        Some("Search results gotten sucssfully".to_string()),
        Some((total, list)),
    ))
}

pub fn edit_event_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<AddUserToFolderPath>,
    item: web::Json<EditEvent>,
) -> Result<OptionalResponse<String, String>, diesel::result::Error> {
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
        return Ok(OptionalResponse::new(
            false,
            Some("A similar event name already exist for this space".to_string()),
            None,
        ));
    }

    let _space_details = diesel::update(events.find(space_name.id))
        .set((
            event_name.eq(&item.event_name),
            event_description.eq(&item.event_description),
        ))
        .execute(&conn)?;

    Ok(OptionalResponse::new(
        true,
        Some("Event updated successfully".to_string()),
        None,
    ))
}

pub fn get_events_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<PathInfo>,
    item: web::Query<PaginateQuery>,
) -> Result<OptionalResponse<String, (i64, Vec<Event>)>, diesel::result::Error> {
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
    let space_event = events
        .filter(event_space_id.eq(&space.id))
        .order(event_date.desc())
        .paginate(item.page)
        .per_page(item.per_page)
        .load::<(Event, i64)>(&conn)?;
    let total = space_event.get(0).map(|x| x.1).unwrap_or(0);
    let list: Vec<Event> = space_event.into_iter().map(|x| x.0).collect();

    Ok(OptionalResponse::new(
        true,
        Some("Event gotten successfully".to_string()),
        Some((total, list)),
    ))
}

pub fn create_event_db(
    db: web::Data<Pool>,
    token: String,
    space_name: web::Path<PathInfo>,
    item: web::Json<AddEvent>,
) -> Result<OptionalResponse<String, Event>, diesel::result::Error> {
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
        return Ok(OptionalResponse::new(
            false,
            Some("A similar event name already exist for this space".to_string()),
            None,
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

    let space_event: Event = insert_into(events).values(&new_event).get_result(&conn)?;

    Ok(OptionalResponse::new(
        true,
        Some("Event created successfully. Space members would be notified on set date".to_string()),
        Some(space_event),
    ))
}
