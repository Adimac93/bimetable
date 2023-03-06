use crate::config::{get_env, try_get_env};
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use tracing::info;

#[derive(Deserialize, Clone)]
pub struct DatabaseFieldsModel {
    username: Option<String>,
    password: Option<String>,
    port: Option<u16>,
    host: Option<String>,
    database_name: Option<String>,
}
impl DatabaseFieldsModel {
    fn to_fields(self) -> DatabaseFields {
        let username = self.username.unwrap_or("postgres".to_string());
        let password = self.password.unwrap_or("".to_string());
        let port = self.port.unwrap_or(5432);
        let host = self.host.unwrap_or("localhost".to_string());
        let database_name = self.database_name.unwrap_or("postgres".to_string());

        DatabaseFields::new(username, password, port, host, database_name)
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
    fn new(
        username: String,
        password: String,
        port: u16,
        host: String,
        database_name: String,
    ) -> Self {
        Self {
            username,
            password: Secret::new(password),
            port,
            host,
            database_name,
        }
    }

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

impl ConnectionPrep for PostgresSettingsModel {
    fn compose_database_url(&self) -> Option<String> {
        if let Some(fields_model) = &self.fields {
            return Some(fields_model.clone().to_fields().compose(self.to_string()));
        }
        None
    }

    fn get_database_url(&self) -> Option<String> {
        self.database_url.clone()
    }

    fn env_database_url() -> Option<String> {
        try_get_env("DATABASE_URL")
    }
}

#[derive(Deserialize, Clone)]
pub struct PostgresSettingsModel {
    database_url: Option<String>,
    fields: Option<DatabaseFieldsModel>,
    is_migrating: Option<bool>,
}

impl PostgresSettingsModel {
    pub fn to_settings(self) -> PostgresSettings {
        let is_migrating = self.is_migrating.unwrap_or(false);
        let database_url = self.get_connection_string();
        PostgresSettings {
            database_url,
            is_migrating,
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct PostgresSettings {
    pub database_url: String,
    pub is_migrating: bool,
}

impl PostgresSettings {
    pub fn from_env() -> Self {
        Self {
            database_url: get_env("DATABASE_URL"),
            is_migrating: true,
        }
    }
}

impl Default for PostgresSettings {
    fn default() -> Self {
        Self {
            database_url: get_env("DATABASE_URL"),
            is_migrating: false,
        }
    }
}

impl ToString for PostgresSettingsModel {
    fn to_string(&self) -> String {
        String::from("postgresql")
    }
}
