-- Your SQL goes here
CREATE TABLE unread_channel_chat(
    id SERIAL NOT NULL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    space_channel_id INTEGER NOT NULL,
    channel_chat_id INTEGER NOT NULL,
    viewed BOOLEAN NOT NULL,
    created_at TIMESTAMP NOT NULL,


    foreign key (user_id) REFERENCES users(id),
    foreign key (space_channel_id) REFERENCES spaces_channel(id),
    foreign key (channel_chat_id) REFERENCES channel_chats(id)
);
