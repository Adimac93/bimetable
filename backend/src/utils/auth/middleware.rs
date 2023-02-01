use axum::{
    http::{Request, StatusCode},
    response::Response,
    middleware::Next,
    extract::State,
};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use secrecy::Secret;
use sqlx::PgPool;
use tracing::{trace, debug};

use crate::{modules::AppState, utils::auth::models::{Claims, RefreshClaims}};

use super::{models::AuthToken, errors::AuthError};

pub async fn auth_access_middleware<B>(
    State(state): State<AppState>,
    jar: CookieJar,
    mut request: Request<B>,
    next: Next<B>,
) -> Result<Response, AuthError> {
    let claims = verify_token::<Claims>(&state.jwt.access.0, &state.pool, jar).await?;
    debug!("inserting");
    request.extensions_mut().insert(claims);

    let response = next.run(request).await;
    Ok(response)
}

pub async fn auth_refresh_middleware<B>(
    State(state): State<AppState>,
    jar: CookieJar,
    mut request: Request<B>,
    next: Next<B>,
) -> Result<Response, AuthError> {
    let claims = verify_token::<RefreshClaims>(&state.jwt.refresh.0, &state.pool, jar).await?;
    debug!("inserting");
    request.extensions_mut().insert(claims);

    let response = next.run(request).await;
    Ok(response)
}

async fn verify_token<'t, T>(secret: &Secret<String>, pool: &PgPool, jar: CookieJar) -> Result<T, AuthError>
where
    T: AuthToken<'t> + Send + Sync,
{
    debug!("getting cookie");
    let cookie = T::get_jwt_cookie(&jar)?;
    debug!("decoding jwt");
    debug!("Cookie value: {:#?}", cookie);
    let claims = T::decode_jwt(cookie.value(), secret.to_owned())?;
    debug!("checking if in blacklist");
    if claims.check_if_in_blacklist(pool).await? {
        debug!("in blacklist");
        return Err(AuthError::InvalidToken);
    }

    Ok(claims)
}