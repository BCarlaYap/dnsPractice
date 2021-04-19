CREATE TABLE IF NOT EXISTS providers (
    id SERIAL,
    name TEXT PRIMARY KEY,
    doh_url TEXT NOT NULL,
    vote_weight REAL NOT NULL
);