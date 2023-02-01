CREATE TABLE jwt_blacklist (
    token_id UUID,
    expiry TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (token_id)
);
