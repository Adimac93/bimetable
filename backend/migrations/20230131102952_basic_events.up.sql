CREATE TABLE events (
	id UUID DEFAULT gen_random_uuid(),
	starts_at TIMESTAMPTZ NOT NULL,
	ends_at TIMESTAMPTZ NOT NULL,
	name TEXT NOT NULL,
	PRIMARY KEY (id)
);
