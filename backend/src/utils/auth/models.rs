use crate::utils::auth::errors::*;
use crate::{modules::AppState, utils::auth::additions::is_ascii_or_latin_extended};
use anyhow::Context;
use axum::{async_trait, extract::FromRequestParts, RequestPartsExt};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use http::request::Parts;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use secrecy::{ExposeSecret, Secret};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sqlx::{query, PgPool};
use time::{Duration, OffsetDateTime};
use tracing::trace;

use crate::config::tokens::JwtSettings;
use uuid::Uuid;
use validator::Validate;

#[async_trait]
pub trait AuthToken<'s>
where
    Self: DeserializeOwned + Serialize + Send + Sized,
{
    const NAME: &'s str;

    fn jti(&self) -> Uuid;
    fn exp(&self) -> u64;
    fn generate_cookie(token: String) -> Cookie<'s> {
        trace!("Generating cookie with token");
        Cookie::build(Self::NAME, token)
            .http_only(true)
            .secure(true)
            .same_site(SameSite::Strict)
            .path("/")
            .finish()
    }
    fn generate_jwt(&self, key: &Secret<String>) -> Result<String, AuthError> {
        trace!("Generating JWT token");
        Ok(encode(
            &Header::default(),
            &self,
            &EncodingKey::from_secret(key.expose_secret().as_bytes()),
        )
        .context("Failed to encrypt token")?)
    }

    fn get_jwt_cookie(jar: &CookieJar) -> Result<Cookie<'s>, AuthError> {
        jar.get(Self::NAME).ok_or(AuthError::InvalidToken).cloned()
    }

    fn decode_jwt(
        jar: &CookieJar,
        validation: Option<&Validation>,
        secret: Secret<String>,
    ) -> Result<Option<TokenData<Self>>, AuthError> {
        let token = Self::get_jwt_cookie(jar)?;
        Ok(decode::<Self>(
            token.value(),
            &DecodingKey::from_secret(secret.expose_secret().as_bytes()),
            validation.unwrap_or(&Validation::default()),
        )
        .ok())
    }

    async fn add_token_to_blacklist(&self, pool: &PgPool) -> Result<(), AuthError> {
        let exp = OffsetDateTime::from_unix_timestamp(self.exp() as i64)
            .context("Failed to convert timestamp to date and time with the timezone")
            .map_err(AuthError::Unexpected)?;

        let _res = query!(
            r#"
                insert into jwt_blacklist (token_id, expiry)
                values ($1, $2)
            "#,
            self.jti(),
            exp,
        )
        .execute(pool)
        .await?;

        trace!("Adding token to blacklist");
        Ok(())
    }
}

impl<'s> AuthToken<'s> for Claims {
    const NAME: &'s str = "access-jwt";

    fn jti(&self) -> Uuid {
        self.jti
    }
    fn exp(&self) -> u64 {
        self.exp
    }
}

impl<'s> AuthToken<'s> for RefreshClaims {
    const NAME: &'s str = "refresh-jwt";

    fn jti(&self) -> Uuid {
        self.jti
    }
    fn exp(&self) -> u64 {
        self.exp
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
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(req: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let secret = req
            .extensions
            .get::<JwtSettings>()
            .context("Failed to get JWT secrets")?
            .to_owned();
        verify_token::<Self>(req, &secret.access.0.token).await
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
impl<S> FromRequestParts<S> for RefreshClaims
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(req: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let secret = req
            .extensions
            .get::<JwtSettings>()
            .context("Failed to get JWT secrets")?
            .to_owned();
        verify_token::<Self>(req, &secret.refresh.0.token).await
    }
}

async fn verify_token<'t, T>(req: &mut Parts, secret: &Secret<String>) -> Result<T, AuthError>
where
    T: AuthToken<'t>,
{
    // get extensions - CookieJar
    let jar = req
        .extract::<CookieJar>()
        .await
        .context("Failed to fetch cookie jar")?;

    trace!("Verifying tokens");

    let decoded = T::decode_jwt(&jar, None, secret.to_owned())?;
    let payload = decoded.ok_or(AuthError::InvalidToken)?;

    trace!("Tokens passed the verification step");

    Ok(payload.claims)
}

#[derive(Validate)]
pub struct ValidatedUserData {
    #[validate(
        non_control_character,
        custom = "is_ascii_or_latin_extended",
        does_not_contain = " ",
        length(min = 4, max = 20)
    )]
    pub login: String,
    #[validate(
        non_control_character,
        custom = "is_ascii_or_latin_extended",
        length(min = 4, max = 20)
    )]
    pub username: String,
}
