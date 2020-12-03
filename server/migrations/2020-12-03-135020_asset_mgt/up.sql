-- Your SQL goes here
CREATE TABLE assets(
  id SERIAL NOT NULL PRIMARY KEY,
  folder_name TEXT NOT NULL,
  space_id INTEGER NOT NULL,
  created_at TIMESTAMP NOT NULL,

  foreign key (space_id) REFERENCES spaces(id)
);

CREATE TABLE asset_contents(
    id SERIAL NOT NULL PRIMARY KEY,
    file_content TEXT NOT NULL,
    asset_id INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,

    foreign key (asset_id) REFERENCES assets(id)
);