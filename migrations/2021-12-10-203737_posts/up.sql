CREATE TABLE posts (
    id SERIAL PRIMARY KEY,
    title VARCHAR NOT NULL,
    body TEXT NOT NULl,
    summary TEXT NOT NULl,
    published BOOLEAN NOT NULL,
    published_date TIMESTAMP NOT NULL
);
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR NOT NULL,
    passwd VARCHAR NOT NULL
);