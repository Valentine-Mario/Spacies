-- Your SQL goes here
-- Your SQL goes here
CREATE TABLE spaces (
  id SERIAL NOT NULL PRIMARY KEY,
  spaces_name TEXT NOT NULL UNIQUE,
  spaces_desc TEXT NOT NULL,
  spaces_img TEXT NOT NULL DEFAULT 'https://res.cloudinary.com/rchain/image/upload/v1601545130/default-logo.png',
  created_at TIMESTAMP NOT NULL
);

CREATE TABLE spaces_users(
    id SERIAL NOT NULL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    space_id INTEGER NOT NULL,
    admin_status BOOLEAN NOT NULL,

    foreign key (user_id) REFERENCES users(id),
    foreign key (space_id) REFERENCES spaces(id)
);