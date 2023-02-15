# Backend

----

## CLI tools

### Database

```bash
cargo install sqlx-cli --no-default-features --features native-tls,postgres
```

### Development

For easier development Cargo Watch watches over your project's source for changes, and runs Cargo commands when they occur.

```bash
cargo install cargo-watch
```

To focus on coding rather than compiling after changes use:

`/backend`

```bash
cargo watch -x run
```

----

## Configuration

### Directory: `backend/configuration/settings.toml`

#### App environment

To choose app environment set `APP_ENVIRONMENT` environmental variable to one of the following options:
- `development` | `dev` | `local` (default)
- `production` | `prod` | `remote`

#### Example settings

```toml
[app]
host = "127.0.0.1"
port = 3001
origin = "http://localhost:3000"

[jwt]
access_secret = "YOUR_SECRET_HERE"
refresh_secret = "YOUR_DIFFERENT_SECRET_HERE"

[postgres]
database_url = "postgresql://postgres@localhost:5432/bimetable"
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

> **Warning**
> Particular default configuration settings might be loaded automatically during `development` if config is partially or fully incompatible. 
> _Read server logs!_

#### Order of database url sourcing

`database_url -> fields -> environment variable`
