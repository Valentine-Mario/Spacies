-- Your SQL goes here
-- Your SQL goes here
CREATE TABLE unread_user_chat(
    id SERIAL NOT NULL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    sender INTEGER NOT NULL,
    user_chat_id INTEGER NOT NULL,
    viewed BOOLEAN NOT NULL,
    created_at TIMESTAMP NOT NULL,


    foreign key (user_id) REFERENCES users(id),
    foreign key (user_chat_id) REFERENCES user_chat(id)
);
