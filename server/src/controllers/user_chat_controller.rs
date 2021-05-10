use crate::auth;
use crate::diesel::QueryDsl;
use crate::diesel::RunQueryDsl;
use crate::handlers::paginate::*;
use crate::handlers::types::*;
use crate::model::{ChatList, Space, User, UserChat};
use crate::schema::spaces::dsl::*;
use crate::schema::unread_user_chat::dsl::*;
use crate::schema::user_chat::dsl::id as user_chat_id;
use crate::schema::user_chat::dsl::user_id as sender_id;
use crate::schema::user_chat::dsl::*;
use crate::schema::users::dsl::id as user_id;
use crate::schema::users::dsl::*;
use crate::Pool;

use actix_web::web;
use diesel::dsl::delete;
use diesel::prelude::*;

pub fn get_chat_list_db(
    db: web::Data<Pool>,
    token: String,
    space: web::Path<PathInfo>,
) -> Result<Response<Vec<User>>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let decoded_token = auth::decode_token(&token);
    let space: Space = spaces
        .filter(spaces_name.ilike(&space.info))
        .first::<Space>(&conn)
        .unwrap();

    let chat_list: Vec<ChatList> = unread_user_chat
        .filter(other.eq(&decoded_token.parse::<i32>().unwrap()))
        .filter(space_id.eq(space.id))
        .order(updated_at.desc())
        .load::<ChatList>(&conn)?;
    //find a better way to do this
    //seems to be the only solution now due to diesel limitations
    let mut return_val: Vec<User> = vec![];
    for item in chat_list {
        let user = users.find(item.user_id).first::<User>(&conn)?;
        return_val.push(user)
    }
    Ok(Response::new(true, return_val))
}

pub fn get_all_message_db(
    db: web::Data<Pool>,
    token: String,
    other_user_id: web::Path<IdPathInfo>,
    item: web::Query<PaginateQuery>,
) -> Result<Response<(Vec<(UserChat, User)>, i64)>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let decoded_token = auth::decode_token(&token);
    let user: User = users
        .filter(user_id.eq_any(vec![
            &decoded_token.parse::<i32>().unwrap(),
            &other_user_id.id,
        ]))
        .first::<User>(&conn)?;

    let all_chats: (Vec<(UserChat, User)>, i64) = UserChat::belonging_to(&user)
        .inner_join(users)
        .filter(user_id.eq_any(vec![&user.id, &other_user_id.id]))
        .filter(reciever.eq_any(vec![&user.id, &other_user_id.id]))
        .order(user_chat_id.desc())
        .paginate(item.page)
        .per_page(item.per_page)
        .load_and_count_pages::<(UserChat, User)>(&conn)?;

    Ok(Response::new(true, all_chats))
}

pub fn delete_message_db(
    db: web::Data<Pool>,
    token: String,
    chat_id: web::Path<IdPathInfo>,
) -> Result<Response<String>, diesel::result::Error> {
    let conn = db.get().unwrap();
    let decoded_token = auth::decode_token(&token);
    let user: User = users
        .find(decoded_token.parse::<i32>().unwrap())
        .first::<User>(&conn)?;
    let _count2 = delete(
        user_chat
            .filter(user_chat_id.eq(chat_id.id))
            .filter(sender_id.eq(user.id)),
    )
    .execute(&conn)?;

    Ok(Response::new(
        true,
        "message deleted successfully".to_string(),
    ))
}
