CREATE TABLE users
(
    id       UUID DEFAULT gen_random_uuid(),
    username TEXT NOT NULL,
    PRIMARY KEY (id)
);

CREATE TABLE credentials
(
    login    TEXT,
    password TEXT NOT NULL,
    user_id  UUID NOT NULL UNIQUE,
    PRIMARY KEY (login),
    FOREIGN KEY (user_id) REFERENCES users (id)
);

CREATE TABLE jwt_blacklist
(
    token_id UUID,
    expiry   TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (token_id)
);
