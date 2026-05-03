CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY NOT NULL,
    username TEXT NOT NULL UNIQUE
);
INSERT
    OR IGNORE INTO users (id, username)
VALUES (1, 'admin');
INSERT
    OR IGNORE INTO users (id, username)
VALUES (2, 'guest');