-- Your SQL goes here
CREATE TABLE usermails(
     id SERIAL NOT NULL PRIMARY KEY,
     maillist_id INTEGER NOT NULL,
     user_id INTEGER NOT NULL,

     foreign key (user_id) REFERENCES users(id),
     foreign key (maillist_id) REFERENCES maillists(id)
)