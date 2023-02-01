use crate::config::JwtSettings;
use axum::extract::FromRef;
use secrecy::Secret;

#[derive(Clone)]
pub struct JwtAccessSecret(pub Secret<String>);

#[derive(Clone)]
pub struct JwtRefreshSecret(pub Secret<String>);

#[derive(Clone, FromRef)]
pub struct TokenSecrets {
    pub access: JwtAccessSecret,
    pub refresh: JwtRefreshSecret,
}

impl TokenSecrets {
    pub fn new(access: JwtAccessSecret, refresh: JwtRefreshSecret) -> Self {
        Self { access, refresh }
    }

    pub fn from_settings(settings: JwtSettings) -> Self {
        Self {
            access: JwtAccessSecret(settings.access_secret),
            refresh: JwtRefreshSecret(settings.refresh_secret),
        }
    }
}
