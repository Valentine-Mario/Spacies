-- Your SQL goes here
CREATE TABLE user_chat(
    id SERIAL NOT NULL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    reciever INTEGER NOT NULL,
    chat TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL,

    foreign key (user_id) REFERENCES users(id)
);