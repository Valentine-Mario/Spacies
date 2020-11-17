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
