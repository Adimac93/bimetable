CREATE TABLE events
(
    id              UUID DEFAULT gen_random_uuid(),
    owner_id        UUID        NOT NULL,
    name            TEXT        NOT NULL,
    description     TEXT,
    starts_at       TIMESTAMPTZ NOT NULL,
    ends_at         TIMESTAMPTZ NOT NULL,
    deleted_at      TIMESTAMPTZ,
    PRIMARY KEY (id),
    FOREIGN KEY (owner_id) REFERENCES users (id)
);

CREATE TABLE recurrence_rules
(
    event_id UUID NOT NULL,
    recurrence JSONB NOT NULL,
    until TIMESTAMPTZ,
    PRIMARY KEY (event_id),
    FOREIGN KEY (event_id) REFERENCES events(id) ON DELETE CASCADE
);

CREATE TABLE event_overrides
(
    id                 UUID                 DEFAULT gen_random_uuid(),
    event_id           UUID        NOT NULL,
    override_starts_at TIMESTAMPTZ NOT NULL,
    override_ends_at   TIMESTAMPTZ NOT NULL,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    name               TEXT,
    description        TEXT,
    starts_at          TIMESTAMPTZ,
    ends_at            TIMESTAMPTZ,
    deleted_at         TIMESTAMPTZ,
    PRIMARY KEY (id),
    FOREIGN KEY (event_id) REFERENCES events (id)
);

CREATE TABLE user_events
(
    user_id     UUID NOT NULL,
    event_id    UUID NOT NULL,
    can_edit    BOOL NOT NULL,
    is_accepted BOOL NOT NULL DEFAULT FALSE,
    PRIMARY KEY (user_id, event_id),
    FOREIGN KEY (user_id) REFERENCES users (id),
    FOREIGN KEY (event_id) REFERENCES events (id) ON DELETE CASCADE
);
