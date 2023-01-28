## Backend

### CLI tools

#### Database

```bash
cargo install sqlx-cli --no-default-features --features native-tls, postgres
```

### Configuration

#### Directory: `backend/configuration/settings.toml`

Example settings:

```toml
[app]
host = "127.0.0.1"
port = 3001
origin = "http://localhost:3000"

[postgres]
database_url = "postgresql://postgres@localhost:5432/bimetable"
# determines whether database should migrate automatically, defaults to 'false'
is_migrating = false

[postgres.fields]
username = "postgres"
password = "leave_empty_if_you_wish"
port = 5432
host = "localhost"
database_name = "bimetable"
```

> **Note**
> Most fields have corresponding uppercase environment variables names.

##### Order of database url sourcing

`database_url -> fields -> environment variable`

---

#### Watch

For easier development Cargo Watch watches over your project's source for changes, and runs Cargo commands when they occur.

```bash
cargo install cargo-watch
```

## Development

To focus on coding rather than compiling after changes use:

`/backend`

```bash
cargo watch -x run
```