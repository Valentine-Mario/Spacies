-- Your SQL goes here
-- Your SQL goes here
CREATE TABLE spaces_email(
     id SERIAL NOT NULL PRIMARY KEY,
     email_address TEXT NOT NULL,
     email_password TEXT NOT NULL,
     space_id INTEGER NOT NULL,

     foreign key (space_id) REFERENCES spaces(id)
);