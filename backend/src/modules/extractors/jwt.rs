use crate::config::JwtSettings;
use secrecy::Secret;

#[derive(Clone)]
pub struct JwtAccessSecret(pub Secret<String>);

#[derive(Clone)]
pub struct JwtRefreshSecret(pub Secret<String>);

#[derive(Clone)]
pub struct TokenExtractors {
    pub access: JwtAccessSecret,
    pub refresh: JwtRefreshSecret,
}

impl TokenExtractors {
    pub fn new(access: JwtAccessSecret, refresh: JwtRefreshSecret) -> Self {
        Self {
            access,
            refresh,
        }
    }

    pub fn from_settings(settings: JwtSettings) -> Self {
        Self {
            access: JwtAccessSecret(settings.access_secret),
            refresh: JwtRefreshSecret(settings.refresh_secret),
        }
    }
}
