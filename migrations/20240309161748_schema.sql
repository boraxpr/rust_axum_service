-- Add migration script here
DROP TABLE IF EXISTS TODO;

CREATE TABLE IF NOT EXISTS todo (
    id serial PRIMARY KEY,
    note TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS post (
    id serial PRIMARY KEY,
    title TEXT NOT NULL,
    body TEXT NOT NULL
)

