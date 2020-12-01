-- Your SQL goes here
CREATE TABLE usermails(
     id SERIAL NOT NULL PRIMARY KEY,
     mail_list_id INTEGER NOT NULL,
     user_id INTEGER NOT NULL,

     foreign key (user_id) REFERENCES users(id),
     foreign key (mail_list_id) REFERENCES maillists(id)
)