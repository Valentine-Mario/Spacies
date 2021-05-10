-- Your SQL goes here
CREATE TABLE channel_users(
     id SERIAL NOT NULL PRIMARY KEY,
     space_channel_id INTEGER NOT NULL,
     space_id INTEGER NOT NULL,
     user_id INTEGER NOT NULL,
     channel_admin BOOLEAN NOT NULL,
     viewed INTEGER NOT NULL,

     foreign key (space_id) REFERENCES spaces(id),
     foreign key (user_id) REFERENCES users(id),
     foreign key (space_channel_id) REFERENCES spaces_channel(id)
);