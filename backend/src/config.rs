use config::{Config, ConfigError};
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use std::net::SocketAddr;
use tracing::{error, info, warn};

const CONFIG_DIR: &str = "configuration";
const CONFIG_FILE_NAME: &str = "settings.toml";

#[derive(Deserialize, Clone)]
pub struct SettingsModel {
    pub app: Option<ApplicationSettings>,
    pub jwt: Option<JwtSettings>,
    pub postgres: Option<PostgresSettings>,
}

impl SettingsModel {
    fn parse() -> Result<Self, ConfigError> {
        let base_path = std::env::current_dir().expect("Failed to determine the current directory");
        let config_dir = base_path.join(CONFIG_DIR);
        let settings = Config::builder()
            .add_source(config::File::from(config_dir.join(CONFIG_FILE_NAME)))
            .add_source(
                config::Environment::with_prefix("APP")
                    .prefix_separator("_")
                    .separator("__"),
            );
        Ok(settings.build()?.try_deserialize()?)
    }
}

#[derive(Clone)]
pub struct Settings {
    pub app: ApplicationSettings,
    pub jwt: JwtSettings,
    pub postgres: PostgresSettings,
}

impl Settings {
    fn dev(model: SettingsModel) -> Self {
        let app = model.app.unwrap_or_else(|| {
            warn!("Using default `app` settings!");
            ApplicationSettings::default()
        });
        let jwt = model.jwt.unwrap_or_else(|| {
            warn!("Using default `jwt` settings!");
            JwtSettings::default()
        });
        let postgres = model.postgres.unwrap_or_else(|| {
            let settings = PostgresSettings::default();
            warn!("Using default `postgres` settings (env url)!");
            settings
        });
        return Self { app, jwt, postgres };
    }

    fn prod() -> Self {
        Self {
            app: ApplicationSettings::from_env(),
            jwt: JwtSettings::from_env(),
            postgres: PostgresSettings::from_env(),
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        let app = ApplicationSettings::default();
        let jwt = JwtSettings::default();
        let postgres = PostgresSettings::default();

        Self { app, jwt, postgres }
    }
}

#[derive(Deserialize, Clone)]
pub struct ApplicationSettings {
    pub host: String,
    pub port: u16,
    pub origin: String,
}

impl ApplicationSettings {
    pub fn get_addr(&self) -> SocketAddr {
        let addr = format!("{}:{}", self.host, self.port);
        addr.parse::<SocketAddr>()
            .expect(&format!("Failed to parse address: {addr} "))
    }

    pub fn from_env() -> Self {
        Self {
            host: "0.0.0.0".into(),
            port: get_env("PORT").parse::<u16>().expect("Invalid port number"),
            origin: get_env("WEBSITE_URL"),
        }
    }
}

impl Default for ApplicationSettings {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3001,
            origin: "http://127.0.0.1".to_string(),
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct JwtSettings {
    pub access_secret: Secret<String>,
    pub refresh_secret: Secret<String>,
}

impl JwtSettings {
    pub fn from_env() -> Self {
        Self {
            access_secret: get_secret_env("JWT_ACCESS_SECRET"),
            refresh_secret: get_secret_env("JWT_REFRESH_SECRET"),
        }
    }
}

impl Default for JwtSettings {
    fn default() -> Self {
        Self {
            access_secret: Secret::new("ACCESS_SECRET".to_string()),
            refresh_secret: Secret::new("REFRESH_SECRET".to_string()),
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct DatabaseFields {
    username: String,
    password: Secret<String>,
    port: u16,
    host: String,
    database_name: String,
}

impl DatabaseFields {
    fn compose(&self, db_name: String) -> String {
        format!(
            "{db_name}://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database_name
        )
    }
}

pub trait ConnectionPrep {
    fn compose_database_url(&self) -> Option<String>;
    fn get_database_url(&self) -> Option<String>;
    fn env_database_url() -> Option<String>;
    fn get_connection_string(&self) -> String
    where
        Self: ToString,
    {
        let info = format!("url for {}", self.to_string());
        if let Some(url) = self.compose_database_url() {
            info!("Using composed {info}");
            url
        } else {
            if let Some(url) = self.get_database_url() {
                info!("Using field {info}");
                url
            } else {
                let url = Self::env_database_url().expect("No connection info provided");
                info!("Using env {info}");
                url
            }
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct PostgresSettings {
    database_url: Option<String>,
    fields: Option<DatabaseFields>,
    is_migrating: Option<bool>,
}

impl PostgresSettings {
    pub fn is_migrating(&self) -> bool {
        self.is_migrating.unwrap_or(false)
    }
    fn from_env() -> Self {
        Self {
            database_url: Self::env_database_url(),
            fields: None,
            is_migrating: Some(true),
        }
    }
}

impl Default for PostgresSettings {
    fn default() -> Self {
        Self {
            database_url: Self::env_database_url(),
            fields: None,
            is_migrating: Some(false),
        }
    }
}

impl ToString for PostgresSettings {
    fn to_string(&self) -> String {
        String::from("postgresql")
    }
}

impl ConnectionPrep for PostgresSettings {
    fn compose_database_url(&self) -> Option<String> {
        Some(self.fields.clone()?.compose(self.to_string()))
    }
    fn get_database_url(&self) -> Option<String> {
        self.database_url.clone()
    }
    fn env_database_url() -> Option<String> {
        try_get_env("DATABASE_URL")
    }
}

enum Environment {
    Development,
    Production,
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "development" | "dev" | "local" => Ok(Self::Development),
            "production" | "prod" | "remote" => Ok(Self::Production),
            other => Err(format!(
                "{other} is not supported environment. Use either `local` or `production`"
            )),
        }
    }
}

pub fn get_config() -> Result<Settings, anyhow::Error> {
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .map_or(Environment::Development, |env| {
            env.try_into().expect("Failed to parse APP_ENVIRONMENT.")
        });

    return match environment {
        Environment::Development => {
            let res = SettingsModel::parse().map_err(|e| {
                error!("{e}\n - check {CONFIG_DIR}/{CONFIG_FILE_NAME}, reference at README.md")
            });
            if let Ok(model) = res {
                return Ok(Settings::dev(model));
            }
            let default = Settings::default();
            warn!("Using default configuration!");
            return Ok(default);
        }

        Environment::Production => Ok(Settings::prod()),
    };
}

fn try_get_env(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

fn try_get_secret_env(name: &str) -> Option<Secret<String>> {
    Some(Secret::from(try_get_env(name)?))
}

fn get_env(name: &str) -> String {
    std::env::var(name).expect(format!("Missing {name}").as_str())
}

fn get_secret_env(name: &str) -> Secret<String> {
    Secret::from(get_env(name))
}
