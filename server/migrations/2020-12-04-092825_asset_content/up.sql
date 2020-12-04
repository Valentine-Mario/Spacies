-- Your SQL goes here
CREATE TABLE asset_contents(
    id SERIAL NOT NULL PRIMARY KEY,
    file_content TEXT NOT NULL,
    file_type TEXT NOT NULL,
    asset_id INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,

    foreign key (asset_id) REFERENCES assets(id)
);