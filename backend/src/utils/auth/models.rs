use crate::{
    modules::{
        extractors::jwt::{JwtAccessSecret, JwtRefreshSecret},
        AuthState,
    },
    utils::auth::{errors::*, TokenExtractors},
};

use anyhow::Context;
use axum::{async_trait, extract::FromRequestParts, RequestPartsExt};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use http::request::Parts;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use secrecy::{ExposeSecret, Secret};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use time::Duration;
use uuid::Uuid;
use validator::Validate;

pub trait AuthToken<'s>
where
    Self: DeserializeOwned + Serialize + Send + Sized,
{
    const NAME: &'s str;
    const JWT_EXPIRATION: Duration;

    fn generate_cookie(token: String) -> Cookie<'s> {
        Cookie::build(Self::NAME, token)
            .http_only(true)
            .secure(true)
            .same_site(SameSite::Strict)
            .path("/")
            .finish()
    }
    fn generate_jwt(&self, key: &Secret<String>) -> Result<String, AuthError> {
        Ok(encode(
            &Header::default(),
            &self,
            &EncodingKey::from_secret(key.expose_secret().as_bytes()),
        )
        .context("Failed to encrypt token")?)
    }
    fn get_jwt_key(ext: &TokenExtractors) -> Secret<String>;
    fn get_jwt_cookie(jar: &CookieJar) -> Result<Cookie<'s>, AuthError> {
        jar.get(Self::NAME).ok_or(AuthError::InvalidToken).cloned()
    }
    fn decode_jwt(token: &str, key: Secret<String>) -> Result<Self, AuthError> {
        // decode token - validation setup
        let mut validation = Validation::default();
        validation.leeway = 5;

        // decode token - try to decode token with a provided jwt key
        let data = decode::<Self>(
            token,
            &DecodingKey::from_secret(key.expose_secret().as_bytes()),
            &validation,
        )
        .map_err(|_e| AuthError::InvalidToken)?;

        Ok(data.claims)
    }
}

#[async_trait]
impl<'s> AuthToken<'s> for Claims {
    const NAME: &'s str = "jwt";
    const JWT_EXPIRATION: Duration = Duration::seconds(15);

    fn get_jwt_key(ext: &TokenExtractors) -> Secret<String> {
        let JwtAccessSecret(jwt_key) = ext.access.clone();
        jwt_key
    }
}

#[async_trait]
impl<'s> AuthToken<'s> for RefreshClaims {
    const NAME: &'s str = "refresh-jwt";
    const JWT_EXPIRATION: Duration = Duration::days(7);

    fn get_jwt_key(ext: &TokenExtractors) -> Secret<String> {
        let JwtRefreshSecret(jwt_key) = ext.refresh.clone();
        jwt_key
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Claims {
    pub jti: Uuid,
    pub user_id: Uuid,
    pub login: String,
    pub exp: u64,
}

impl Claims {
    pub fn new(user_id: Uuid, login: &str, duration: Duration) -> Self {
        Self {
            jti: Uuid::new_v4(),
            user_id,
            login: login.to_string(),
            exp: jsonwebtoken::get_current_timestamp() + duration.whole_seconds().abs() as u64,
        }
    }
}

#[async_trait]
impl FromRequestParts<AuthState> for Claims {
    type Rejection = AuthError;

    async fn from_request_parts(
        req: &mut Parts,
        state: &AuthState,
    ) -> Result<Self, Self::Rejection> {
        verify_token::<Self>(req, state).await
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RefreshClaims {
    pub jti: Uuid,
    pub user_id: Uuid,
    pub login: String,
    pub exp: u64,
}

impl RefreshClaims {
    pub fn new(user_id: Uuid, login: &str, duration: Duration) -> Self {
        Self {
            jti: Uuid::new_v4(),
            user_id,
            login: login.to_string(),
            exp: jsonwebtoken::get_current_timestamp() + duration.whole_seconds().abs() as u64,
        }
    }
}

#[async_trait]
impl FromRequestParts<AuthState> for RefreshClaims {
    type Rejection = AuthError;

    async fn from_request_parts(
        req: &mut Parts,
        state: &AuthState,
    ) -> Result<Self, Self::Rejection> {
        verify_token::<Self>(req, state).await
    }
}

async fn verify_token<'t, T>(req: &mut Parts, state: &AuthState) -> Result<T, AuthError>
where
    T: AuthToken<'t>,
{
    let jwt_key = T::get_jwt_key(&state.jwt);

    // get extensions - CookieJar
    let jar = req
        .extract::<CookieJar>()
        .await
        .context("Failed to fetch cookie jar")?;

    let cookie = T::get_jwt_cookie(&jar)?;

    let claims = T::decode_jwt(cookie.value(), jwt_key)?;

    Ok(claims)
}

#[derive(Serialize, Deserialize)]
pub struct LoginCredentials {
    pub login: String,
    pub password: String,
}

impl LoginCredentials {
    pub fn new(login: &str, password: &str) -> Self {
        Self {
            login: login.into(),
            password: password.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Validate)]
pub struct RegisterCredentials {
    pub login: String,
    pub password: String,
    pub username: String,
}

impl RegisterCredentials {
    pub fn new(login: &str, password: &str, username: &str) -> Self {
        Self {
            login: login.into(),
            password: password.into(),
            username: username.into(),
        }
    }
}
