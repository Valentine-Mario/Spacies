-- Your SQL goes here
CREATE TABLE spaces_channel(
     id SERIAL NOT NULL PRIMARY KEY,
     channel_name TEXT NOT NULL,
     space_id INTEGER NOT NULL,

     foreign key (space_id) REFERENCES spaces(id)
)