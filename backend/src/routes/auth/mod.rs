pub mod models;

use crate::modules::AppState;
use crate::routes::auth::models::{LoginCredentials, RegisterCredentials};
use crate::utils::auth::errors::AuthError;
use crate::utils::auth::models::*;
use crate::utils::auth::*;
use axum::extract::State;
use axum::{debug_handler, extract, http::StatusCode, Extension, Json};
use axum::{routing::post, Router};
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::CookieJar;
use jsonwebtoken::{decode, DecodingKey, Validation};
use secrecy::{ExposeSecret, SecretString};
use serde_json::{json, Value};
use sqlx::PgPool;

use crate::config::tokens::JwtSettings;
use time::Duration;
use tracing::debug;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(post_register_user))
        .route("/login", post(post_login_user))
        .route("/validate", post(protected_zone))
        .route("/logout", post(post_logout_user))
        .route("/refresh", post(post_refresh_user_token))
}

/// Register user
#[utoipa::path(post, path = "/auth/register", tag = "auth", request_body = RegisterCredentials, responses((status = 200, description = "User has successfully registered")))]
#[debug_handler]
async fn post_register_user(
    State(pool): State<PgPool>,
    Extension(secrets): Extension<JwtSettings>,
    jar: CookieJar,
    Json(register_credentials): Json<RegisterCredentials>,
) -> Result<CookieJar, AuthError> {
    let user_id = try_register_user(
        &pool,
        register_credentials.login.trim(),
        SecretString::new(register_credentials.password.trim().to_string()),
        &register_credentials.username,
    )
    .await?;

    let jar = generate_token_cookies(user_id, &register_credentials.login, secrets, jar)?;

    debug!(
        "User {} ({}) registered successfully",
        user_id, &register_credentials.username,
    );

    Ok(jar)
}

/// Login user
#[utoipa::path(post, path = "/auth/login", tag = "auth", request_body = LoginCredentials, responses((status = 200, description = "User has successfully logged in")))]
async fn post_login_user(
    State(pool): State<PgPool>,
    Extension(secrets): Extension<JwtSettings>,
    jar: CookieJar,
    Json(login_credentials): Json<LoginCredentials>,
) -> Result<CookieJar, AuthError> {
    // returns if credentials are wrong
    let mut conn = pool.acquire().await?;

    let user_id = verify_user_credentials(
        &mut conn,
        &login_credentials.login,
        SecretString::new(login_credentials.password.clone()),
    )
    .await?;

    let jar = generate_token_cookies(user_id, &login_credentials.login, secrets, jar)?;

    debug!("User {} logged in successfully", user_id);

    Ok(jar)
}

/// Validate tokens
#[utoipa::path(post, path = "/auth/validate", tag = "auth", responses((status = 200, description = "User has valid auth tokens")))]
async fn protected_zone(claims: Claims) -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({ "user_id": claims.user_id })))
}

/// Logout user
#[utoipa::path(post, path = "/auth/logout", tag = "auth")]
async fn post_logout_user(
    State(state): State<AppState>,
    Extension(secrets): Extension<JwtSettings>,
    jar: CookieJar,
) -> Result<CookieJar, AuthError> {
    let validation = Validation::default();

    if let Ok(Some(data)) = Claims::decode_jwt(&jar, Some(&validation), secrets.access.0.token) {
        let _ = &data.claims.add_token_to_blacklist(&state.pool).await?;
    }

    if let Ok(Some(data)) =
        RefreshClaims::decode_jwt(&jar, Some(&validation), secrets.refresh.0.token)
    {
        let _ = &data.claims.add_token_to_blacklist(&state.pool).await?;
    }

    debug!("User logged out successfully");

    Ok(jar
        .remove(get_remove_cookie(Claims::NAME))
        .remove(get_remove_cookie(RefreshClaims::NAME)))
}

fn get_remove_cookie(name: &str) -> Cookie {
    Cookie::build(name, "")
        .path("/")
        .max_age(Duration::seconds(0))
        .finish()
}

/// Refresh access token
#[utoipa::path(post, path = "/auth/refresh", tag = "auth", responses((status = 200, description = "Refreshed user access token")))]
async fn post_refresh_user_token(
    State(state): State<AppState>,
    Extension(secrets): Extension<JwtSettings>,
    jar: CookieJar,
    refresh_claims: RefreshClaims,
) -> Result<CookieJar, AuthError> {
    let jar = generate_token_cookies(refresh_claims.user_id, &refresh_claims.login, secrets, jar)?;

    refresh_claims.add_token_to_blacklist(&state.pool).await?;

    debug!(
        "Access token of user {} refreshed successfully",
        &refresh_claims.user_id,
    );

    Ok(jar)
}
