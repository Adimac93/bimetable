use super::get_secret_env;
use crate::config::get_env;
use secrecy::Secret;
use serde::Deserialize;
use time::Duration;
use tracing::log::warn;

const ACCESS_SECRET: &str = "JWT_ACCESS_SECRET";
const REFRESH_SECRET: &str = "JWT_REFRESH_SECRET";

const ACCESS_EXPIRATION: Duration = Duration::minutes(5);
const REFRESH_EXPIRATION: Duration = Duration::days(7);
const SUPER_EXPIRATION: Duration = Duration::days(2137);

#[derive(Deserialize)]
pub struct JwtSettingsModel {
    pub access: Option<TokenDataModel>,
    pub refresh: Option<TokenDataModel>,
    pub is_super_user: Option<bool>,
}

#[derive(Deserialize)]
pub struct TokenDataModel {
    pub token: Option<String>,
    pub expiration: Option<Duration>,
}

impl TokenDataModel {
    fn to_access(self) -> AccessTokenData {
        AccessTokenData(TokenData::new(
            &self.token.unwrap_or(ACCESS_SECRET.to_string()),
            self.expiration.map_or(ACCESS_EXPIRATION, |expiration| {
                warn!("Using custom access token expiration of {}", &expiration);
                expiration
            }),
        ))
    }

    fn to_refresh(self) -> RefreshTokenData {
        RefreshTokenData(TokenData::new(
            &self.token.unwrap_or(REFRESH_SECRET.to_string()),
            self.expiration.map_or(REFRESH_EXPIRATION, |expiration| {
                warn!("Using custom refresh token expiration of {}", &expiration);
                expiration
            }),
        ))
    }
}

#[derive(Clone)]
pub struct JwtSettings {
    pub access: AccessTokenData,
    pub refresh: RefreshTokenData,
}

impl JwtSettings {
    pub fn new(access: &str, refresh: &str) -> Self {
        Self {
            access: AccessTokenData(TokenData::new(access, SUPER_EXPIRATION)),
            refresh: RefreshTokenData(TokenData::new(refresh, SUPER_EXPIRATION)),
        }
    }

    fn super_user() -> Self {
        Self {
            access: AccessTokenData::super_token(),
            refresh: RefreshTokenData::super_token(),
        }
    }

    pub fn from_env() -> Self {
        Self {
            access: AccessTokenData::from_env(),
            refresh: RefreshTokenData::from_env(),
        }
    }
}

impl Default for JwtSettings {
    fn default() -> Self {
        Self {
            access: AccessTokenData::default(),
            refresh: RefreshTokenData::default(),
        }
    }
}

#[derive(Clone)]
pub struct TokenData {
    pub token: Secret<String>,
    pub expiration: Duration,
}

#[derive(Clone)]
pub struct AccessTokenData(pub TokenData);

impl AccessTokenData {
    fn super_token() -> Self {
        Self(TokenData::new(ACCESS_SECRET, SUPER_EXPIRATION))
    }
    fn from_env() -> Self {
        Self(TokenData::new(&get_env("ACCESS_SECRET"), ACCESS_EXPIRATION))
    }
}

impl Default for AccessTokenData {
    fn default() -> Self {
        Self(TokenData::new(ACCESS_SECRET, ACCESS_EXPIRATION))
    }
}

#[derive(Clone)]
pub struct RefreshTokenData(pub TokenData);

impl RefreshTokenData {
    fn super_token() -> Self {
        Self(TokenData::new(REFRESH_SECRET, SUPER_EXPIRATION))
    }
    fn from_env() -> Self {
        Self(TokenData::new(
            &get_env("REFRESH_SECRET"),
            REFRESH_EXPIRATION,
        ))
    }
}

impl Default for RefreshTokenData {
    fn default() -> Self {
        Self(TokenData::new(REFRESH_SECRET, REFRESH_EXPIRATION))
    }
}

impl TokenData {
    fn new(token: &str, expiration: Duration) -> Self {
        Self {
            token: Secret::new(token.to_owned()),
            expiration,
        }
    }
}

impl JwtSettingsModel {
    pub fn to_settings(self) -> JwtSettings {
        if self.is_super_user.unwrap_or(false) {
            warn!("Using super tokens");
            return JwtSettings::super_user();
        }

        let access = self.access.map_or_else(
            || {
                warn!("Using default access token");
                AccessTokenData::default()
            },
            |t| t.to_access(),
        );

        let refresh = self.refresh.map_or_else(
            || {
                warn!("Using default refresh token");
                RefreshTokenData::default()
            },
            |t| t.to_refresh(),
        );

        JwtSettings { access, refresh }
    }
}
