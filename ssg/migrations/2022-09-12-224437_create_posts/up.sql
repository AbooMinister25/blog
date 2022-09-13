CREATE TABLE posts (
    id INTEGER PRIMARY KEY,
    title VARCHAR NOT NULL,
    content TEXT NOT NULL,
    summary TEXT NOT NULL,
    tags TEXT NOT NULL,
    published BOOLEAN NOT NULL DEFAULT 'f',
    published_at TIMESTAMP NOT NULL
);