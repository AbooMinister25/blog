CREATE TABLE posts (
    id SERIAL PRIMARY KEY,
    title VARCHAR NOT NULL,
    body TEXT NOT NULL,
    summary TEXT NOT NULL,
    published BOOLEAN NOT NULL DEFAULT 'f',
    published_at TIMESTAMP NOT NULL
);