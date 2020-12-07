-- Your SQL goes here
CREATE TABLE events(
    id SERIAL NOT NULL PRIMARY KEY,
    event_name TEXT NOT NULL,
    event_description TEXT NOT NULL,
    reminded BOOLEAN NOT NULL,
    event_date TIMESTAMP NOT NULL,
    space_id INTEGER NOT NULL,

    foreign key (space_id) REFERENCES spaces(id)
)