table! {
    maillists (id) {
        id -> Int4,
        folder_name -> Text,
        space_id -> Int4,
    }
}

table! {
    spaces (id) {
        id -> Int4,
        spaces_name -> Text,
        spaces_desc -> Text,
        spaces_img -> Text,
        created_at -> Timestamp,
    }
}

table! {
    spaces_channel (id) {
        id -> Int4,
        channel_name -> Text,
        space_id -> Int4,
    }
}

table! {
    spaces_users (id) {
        id -> Int4,
        user_id -> Int4,
        space_id -> Int4,
        admin_status -> Bool,
    }
}

table! {
    usermails (id) {
        id -> Int4,
        mail_list_id -> Int4,
        user_id -> Int4,
    }
}

table! {
    users (id) {
        id -> Int4,
        username -> Text,
        email -> Text,
        user_password -> Text,
        user_image -> Text,
        verified -> Bool,
        created_at -> Timestamp,
    }
}

joinable!(maillists -> spaces (space_id));
joinable!(spaces_channel -> spaces (space_id));
joinable!(spaces_users -> spaces (space_id));
joinable!(spaces_users -> users (user_id));
joinable!(usermails -> maillists (mail_list_id));
joinable!(usermails -> users (user_id));

allow_tables_to_appear_in_same_query!(
    maillists,
    spaces,
    spaces_channel,
    spaces_users,
    usermails,
    users,
);
