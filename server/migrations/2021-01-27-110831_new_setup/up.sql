-- Your SQL goes here
CREATE TABLE users (
  id SERIAL NOT NULL PRIMARY KEY,
  username TEXT NOT NULL,
  email TEXT NOT NULL UNIQUE,
  user_password TEXT NOT NULL,
  user_image TEXT NOT NULL DEFAULT 'https://res.cloudinary.com/rchain/image/upload/v1605608882/1_W35QUSvGpcLuxPo3SRTH4w.png',
  verified BOOLEAN NOT NULL,
  created_at TIMESTAMP NOT NULL
);

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

CREATE TABLE spaces_channel(
     id SERIAL NOT NULL PRIMARY KEY,
     channel_name TEXT NOT NULL,
     space_id INTEGER NOT NULL,

     foreign key (space_id) REFERENCES spaces(id)
);

CREATE TABLE maillists(
     id SERIAL NOT NULL PRIMARY KEY,
     folder_name TEXT NOT NULL,
     space_id INTEGER NOT NULL,

     foreign key (space_id) REFERENCES spaces(id)
);

CREATE TABLE usermails(
     id SERIAL NOT NULL PRIMARY KEY,
     mail_list_id INTEGER NOT NULL,
     user_id INTEGER NOT NULL,

     foreign key (user_id) REFERENCES users(id),
     foreign key (mail_list_id) REFERENCES maillists(id)
);

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
    file_type TEXT NOT NULL,
    asset_id INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,

    foreign key (asset_id) REFERENCES assets(id)
);

CREATE TABLE events(
    id SERIAL NOT NULL PRIMARY KEY,
    event_name TEXT NOT NULL,
    event_description TEXT NOT NULL,
    reminded BOOLEAN NOT NULL,
    event_date TIMESTAMP NOT NULL,
    space_id INTEGER NOT NULL,

    foreign key (space_id) REFERENCES spaces(id)
);

CREATE TABLE projects(
    id SERIAL NOT NULL PRIMARY KEY,
    project_name TEXT NOT NULL,
    space_id INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,

    foreign key (space_id) REFERENCES spaces(id)
);

CREATE TABLE tasks(
     id SERIAL NOT NULL PRIMARY KEY,
     task_name TEXT NOT NULL,
     task_description TEXT NOT NULL,
     project_id INTEGER NOT NULL,
     task_status TEXT NOT NULL,
     due_date TIMESTAMP NOT NULL,

     foreign key (project_id) REFERENCES projects(id)
);

CREATE TABLE user_tasks(
    id SERIAL NOT NULL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    task_id INTEGER NOT NULL,

    foreign key (user_id) REFERENCES users(id),
    foreign key (task_id) REFERENCES tasks(id)
);

CREATE TABLE channel_chats(
    id SERIAL NOT NULL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    space_channel_id INTEGER NOT NULL,
    chat TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL,


    foreign key (user_id) REFERENCES users(id),
    foreign key (space_channel_id) REFERENCES spaces_channel(id)
);

CREATE TABLE user_chat(
    id SERIAL NOT NULL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    reciever INTEGER NOT NULL,
    chat TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL,
    space_id INTEGER NOT NULL,

    foreign key (space_id) REFERENCES spaces(id),
    foreign key (user_id) REFERENCES users(id)
);

CREATE TABLE channel_users(
     id SERIAL NOT NULL PRIMARY KEY,
     space_channel_id INTEGER NOT NULL,
     space_id INTEGER NOT NULL,
     user_id INTEGER NOT NULL,
     channel_admin BOOLEAN NOT NULL,

     foreign key (space_id) REFERENCES spaces(id),
     foreign key (user_id) REFERENCES users(id),
     foreign key (space_channel_id) REFERENCES spaces_channel(id)
);