CREATE TABLE events
(
    id              UUID DEFAULT gen_random_uuid(),
    owner_id        UUID NOT NULL,
    name            TEXT NOT NULL,
    description     TEXT NOT NULL,
    starts_at       TIMESTAMPTZ,
    ends_at         TIMESTAMPTZ,
    recurrence_rule JSONB,
    PRIMARY KEY (id),
    FOREIGN KEY (owner_id) REFERENCES users (id)
);

CREATE TABLE event_overrides
(
    id          UUID DEFAULT gen_random_uuid(),
    event_id    UUID NOT NULL,
    ord_start   INT NOT NULL, -- nullable for from start modification?
    ord_end     INT NOT NULL, -- nullable for to end modification?
    name        TEXT,
    description TEXT,
    starts_at   TIMESTAMPTZ,
    ends_at     TIMESTAMPTZ,
    PRIMARY KEY (id),
    FOREIGN KEY (event_id) REFERENCES events (id)
);

CREATE TABLE user_events
(
    user_id  UUID NOT NULL,
    event_id UUID NOT NULL,
    can_edit BOOL NOT NULL,
    PRIMARY KEY (user_id, event_id),
    FOREIGN KEY (user_id) REFERENCES users (id),
    FOREIGN KEY (event_id) REFERENCES events (id)
);
