use crate::config::app::{ApplicationSettings, ApplicationSettingsModel};
use crate::config::database::{PostgresSettings, PostgresSettingsModel};
use crate::config::environment::Environment;
use crate::config::tokens::{JwtSettings, JwtSettingsModel};
use config::{Config, ConfigError};
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use std::fmt::{Display, Formatter};
use tracing::{error, info, warn};

pub mod app;
pub mod database;
pub mod environment;
pub mod tokens;

const CONFIG_DIR: &str = "configuration";
const CONFIG_FILE_NAME: &str = "settings.toml";

#[derive(Deserialize)]
pub struct SettingsModel {
    pub app: Option<ApplicationSettingsModel>,
    pub jwt: Option<JwtSettingsModel>,
    pub postgres: Option<PostgresSettingsModel>,
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
    pub environment: Environment,
}

impl Settings {
    fn dev(model: SettingsModel) -> Self {
        let app = model.app.map_or_else(
            || {
                warn!("Using default `app` settings!");
                ApplicationSettings::default()
            },
            |x| x.to_settings(),
        );

        let jwt = model.jwt.map_or_else(
            || {
                warn!("Using default `jwt` settings!");
                JwtSettings::default()
            },
            |x| x.to_settings(),
        );

        let postgres = model.postgres.map_or_else(
            || {
                let settings = PostgresSettings::default();
                warn!("Using default `postgres` settings (env url)!");
                settings
            },
            |x| x.to_settings(),
        );

        return Self {
            app,
            jwt,
            postgres,
            environment: Environment::Development,
        };
    }

    fn prod() -> Self {
        Self {
            app: ApplicationSettings::from_env(),
            jwt: JwtSettings::from_env(),
            postgres: PostgresSettings::from_env(),
            environment: Environment::Production,
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        let app = ApplicationSettings::default();
        let jwt = JwtSettings::default();
        let postgres = PostgresSettings::default();
        let environment = Environment::default();

        Self {
            app,
            jwt,
            postgres,
            environment,
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

pub fn try_get_env(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

pub fn try_get_secret_env(name: &str) -> Option<Secret<String>> {
    Some(Secret::from(try_get_env(name)?))
}

pub fn get_env(name: &str) -> String {
    std::env::var(name).expect(format!("Missing {name}").as_str())
}

pub fn get_secret_env(name: &str) -> Secret<String> {
    Secret::from(get_env(name))
}
