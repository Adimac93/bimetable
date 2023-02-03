CREATE TABLE recurrence_rules
(
    id        UUID DEFAULT gen_random_uuid(),
    week_map  BIT,
    interval  INT,
    is_by_day BOOL,
    PRIMARY KEY (id)
);

CREATE TABLE events
(
    id                 UUID DEFAULT gen_random_uuid(),
    owner_id           UUID NOT NULL,
    name               TEXT NOT NULL,
    starts_at          TIMETZ,
    ends_at            TIMETZ,
    date               DATE NOT NULL,
    recurrence_rule_id UUID,
    PRIMARY KEY (id),
    FOREIGN KEY (recurrence_rule_id) REFERENCES recurrence_rules (id),
    FOREIGN KEY (owner_id) REFERENCES users (id)
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
