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