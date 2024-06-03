-- Add migration script here
CREATE TABLE urls (
    id SERIAL PRIMARY KEY,
    long_url TEXT NOT NULL UNIQUE,
    short_url VARCHAR(10) NOT NULL UNIQUE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    clicks INT NOT NULL DEFAULT 0
);
