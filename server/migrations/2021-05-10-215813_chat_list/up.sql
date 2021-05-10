-- Your SQL goes here
CREATE TABLE unread_user_chat(
    id SERIAL NOT NULL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    other INTEGER NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    space_id INTEGER NOT NULL,


    foreign key (user_id) REFERENCES users(id),
    foreign key (space_id) REFERENCES spaces(id)
);
