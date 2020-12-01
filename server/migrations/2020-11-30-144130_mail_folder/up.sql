-- Your SQL goes here
-- Your SQL goes here
CREATE TABLE maillists(
     id SERIAL NOT NULL PRIMARY KEY,
     folder_name TEXT NOT NULL,
     space_id INTEGER NOT NULL,

     foreign key (space_id) REFERENCES spaces(id)
)