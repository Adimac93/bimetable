CREATE TABLE users (
    id UUID DEFAULT gen_random_uuid(),
    username TEXT NOT NULL,
    PRIMARY KEY (id)
);

CREATE TABLE credentials (
    login TEXT,
    password TEXT NOT NULL,
    user_id UUID NOT NULL UNIQUE,
    PRIMARY KEY (login),
    FOREIGN KEY (user_id) REFERENCES users(id)
);
