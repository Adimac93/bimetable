pub mod additions;
pub mod errors;
pub mod models;
use crate::modules::{
    extractors::jwt::TokenExtractors,
};
use anyhow::Context;
use argon2::verify_encoded;
use axum_extra::extract::{cookie::Cookie, CookieJar};
use errors::*;
use models::*;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use sqlx::{query, Acquire, PgPool, Postgres};
use time::OffsetDateTime;
use tracing::{debug, trace};
use uuid::Uuid;
use validator::Validate;

#[derive(sqlx::Type, Debug, Serialize, Deserialize)]
#[sqlx(type_name = "status", rename_all = "snake_case")]
pub enum ActivityStatus {
    Online,
    Offline,
    Idle,
}

// todo: make as transaction with Acquire
pub async fn try_register_user<'c>(
    pool: &PgPool,
    email: &str,
    password: SecretString,
    username: &str,
) -> Result<Uuid, AuthError> {
    let mut transaction = pool.begin().await?;

    let user = query!(
        r#"
            select user_id from credentials where login = $1
        "#,
        email
    )
    .fetch_optional(&mut transaction)
    .await?;

    if user.is_some() {
        return Err(AuthError::UserAlreadyExists);
    }

    if email.trim().is_empty() || password.expose_secret().trim().is_empty() {
        return Err(AuthError::MissingCredential);
    }

    if !additions::pass_is_strong(password.expose_secret(), &[&email]) {
        return Err(AuthError::WeakPassword);
    }

    let hashed_pass = additions::hash_pass(password)
        .context("Failed to hash password with argon2")
        .map_err(AuthError::Unexpected)?;

    let mut username = username.trim();
    if username.is_empty() {
        // TODO: Generate random username
        username = "I am definitely not a chad"
    }

    let user_id = query!(
        r#"
            insert into users (username)
            values ($1)
            returning (id)
        "#,
        username,
    )
    .fetch_one(&mut transaction)
    .await?
    .id;

    query!(
        r#"
            insert into credentials (user_id, login, password)
            values ($1, $2, $3)
        "#,
        user_id,
        username,
        hashed_pass
    )
    .execute(&mut transaction)
    .await?;

    // query!(
    //     r#"
    //         insert into user_networks (ip, user_id, is_trusted)
    //         values ($1, $2, true)
    //     "#,
    //     ip,
    //     user_id
    // )
    // .execute(&mut transaction)
    // .await?;

    transaction.commit().await?;

    Ok(user_id)
}

pub async fn verify_user_credentials(
    pool: &PgPool,
    username: String,
    password: SecretString,
) -> Result<Uuid, AuthError> {
    debug!("Verifying credentials");
    if username.trim().is_empty() || password.expose_secret().trim().is_empty() {
        return Err(AuthError::MissingCredential)?;
    }

    let res = query!(
        r#"
            select users.id, password from credentials
            join users on credentials.user_id = users.id
            where username = $1
        "#,
        username
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AuthError::WrongEmailOrPassword)?;

    match verify_encoded(&res.password, password.expose_secret().as_bytes())
        .context("Failed to verify credentials")
        .map_err(AuthError::Unexpected)?
    {
        true => Ok(res.id),
        false => Err(AuthError::WrongEmailOrPassword),
    }
}

pub async fn generate_token_cookies(
    user_id: Uuid,
    login: &str,
    ext: &TokenExtractors,
    jar: CookieJar,
) -> Result<CookieJar, AuthError> {
    let access_cookie = generate_jwt_in_cookie::<Claims>(user_id, login, ext).await?;

    trace!("Access JWT: {access_cookie:#?}");

    let refresh_cookie = generate_jwt_in_cookie::<RefreshClaims>(user_id, login, ext).await?;

    trace!("Refresh JWT: {refresh_cookie:#?}");

    Ok(jar.add(access_cookie).add(refresh_cookie))
}

async fn generate_jwt_in_cookie<'a, T>(
    user_id: Uuid,
    login: &str,
    ext: &TokenExtractors,
) -> Result<Cookie<'a>, AuthError>
where
    T: AuthToken,
{
    let access_token = T::generate_jwt(
        user_id,
        login,
        T::JWT_EXPIRATION,
        &T::get_jwt_key(ext).await,
    )
    .await?;

    let access_cookie = T::generate_cookie(access_token).await;
    trace!("Access JWT: {access_cookie:#?}");

    Ok(access_cookie)
}
