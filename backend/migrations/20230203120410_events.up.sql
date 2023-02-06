CREATE TABLE events
(
    id                 UUID DEFAULT gen_random_uuid(),
    owner_id           UUID NOT NULL,
    name               TEXT NOT NULL,
    description        TEXT NOT NULL,
    starts_at          TIMESTAMPTZ,
    ends_at            TIMESTAMPTZ,
    recurrence_rule    JSONB,
    PRIMARY KEY (id),
    FOREIGN KEY (owner_id) REFERENCES users (id)
);

CREATE TABLE modified_events
(
    origin_id        UUID,
    occurrence_id    INT,
    name             TEXT NOT NULL,
    description      TEXT NOT NULL,
    starts_at        TIMESTAMPTZ,
    ends_at          TIMESTAMPTZ,
    PRIMARY KEY (origin_id, occurrence_id),
    FOREIGN KEY (origin_id) ON DELETE CASCADE
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
