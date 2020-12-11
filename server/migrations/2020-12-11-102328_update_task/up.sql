-- Your SQL goes here
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
)