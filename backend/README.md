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
is_super_user = true
[jwt.access]
token = "JWT_ACCESS_TOKEN"
expiration = "600.0" # 10 minutes
[jwt.refresh]
token = "JWT_REFRESH_TOKEN"
expiration = "604800.0" # 7 days

[postgres]
database_url = "postgresql://postgres@localhost:5432/postgres"
is_migrating = false

[postgres.fields]
username = "postgres"
password = ""
port = 5432
host = "localhost"
database_name = "postgres"
```

> **Warning**
> Default configuration settings might be loaded automatically during `development` if config is partially or fully incompatible.
> All settings will default to example above.
> _Read server logs!_
#### 

#### Order of database url sourcing

`database_url -> fields -> environment variable`
