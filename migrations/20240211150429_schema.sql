-- Add migration script here
DROP TABLE IF EXISTS todos;

CREATE TABLE IF NOT EXISTS TODO (
    id serial PRIMARY KEY,
    note TEXT NOT NULL
);