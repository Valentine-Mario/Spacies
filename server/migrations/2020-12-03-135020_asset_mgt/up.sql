-- Your SQL goes here
CREATE TABLE assets(
  id SERIAL NOT NULL PRIMARY KEY,
  folder_name TEXT NOT NULL,
  space_id INTEGER NOT NULL,
  created_at TIMESTAMP NOT NULL,

  foreign key (space_id) REFERENCES spaces(id)
);

