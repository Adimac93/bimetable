pub mod additions;
pub mod errors;
pub mod models;
use crate::modules::extensions::jwt::TokenSecrets;
use crate::utils::auth::additions::{hash_pass, verify_pass};
use crate::utils::auth::errors::AuthError::WrongLoginOrPassword;
use axum_extra::extract::{cookie::Cookie, CookieJar};
use errors::*;
use models::*;
use secrecy::{ExposeSecret, SecretString};
use sqlx::{query, Acquire, PgConnection, Postgres};
use tracing::{debug, trace};
use uuid::Uuid;

use self::additions::validate_usernames;
// use validator::Validate;

pub async fn try_register_user<'c>(
    acq: impl Acquire<'c, Database = Postgres>,
    login: &str,
    password: SecretString,
    username: &str,
) -> Result<Uuid, AuthError> {
    let mut transaction = acq.begin().await?;

    let mut q = AuthUser::new(login, &mut transaction);

    if !q.is_new().await? {
        return Err(AuthError::UserAlreadyExists);
    }

    if login.trim().is_empty()
        || password.expose_secret().trim().is_empty()
        || username.trim().is_empty()
    {
        return Err(AuthError::MissingCredential);
    }

    validate_usernames(login, username)?;

    if !additions::pass_is_strong(password.expose_secret(), &[&login]) {
        return Err(AuthError::WeakPassword);
    }

    let hashed_pass = hash_pass(password.expose_secret().to_owned())?;

    let user_id = q.create_account(hashed_pass, username).await?;

    transaction.commit().await?;

    Ok(user_id)
}

pub async fn verify_user_credentials<'c>(
    conn: &mut PgConnection,
    login: &str,
    password: SecretString,
) -> Result<Uuid, AuthError> {
    debug!("Verifying credentials");
    if login.trim().is_empty() || password.expose_secret().trim().is_empty() {
        return Err(AuthError::MissingCredential)?;
    }

    let mut q = AuthUser::new(login, conn);
    let user_id = q.verify_credentials(password).await?;
    Ok(user_id)
}

pub async fn generate_token_cookies(
    user_id: Uuid,
    login: &str,
    secrets: TokenSecrets,
    jar: CookieJar,
) -> Result<CookieJar, AuthError> {
    let access_cookie = generate_jwt_in_cookie(
        Claims::new(user_id, login, Claims::JWT_EXPIRATION),
        &secrets.access.0,
    )
    .await?;

    trace!("Access JWT: {access_cookie:#?}");

    let refresh_cookie = generate_jwt_in_cookie(
        RefreshClaims::new(user_id, login, RefreshClaims::JWT_EXPIRATION),
        &secrets.refresh.0,
    )
    .await?;

    trace!("Refresh JWT: {refresh_cookie:#?}");

    Ok(jar.add(access_cookie).add(refresh_cookie))
}

async fn generate_jwt_in_cookie<'a, T: AuthToken<'a>>(
    payload: T,
    secret: &SecretString,
) -> Result<Cookie<'a>, AuthError> {
    let token = payload.generate_jwt(secret)?;
    let access_cookie = T::generate_cookie(token);
    trace!("Access JWT: {access_cookie:#?}");

    Ok(access_cookie)
}

pub struct AuthUser<'c> {
    login: &'c str,
    conn: &'c mut PgConnection,
}

impl<'c> AuthUser<'c> {
    fn new(login: &'c str, conn: &'c mut PgConnection) -> Self {
        Self { login, conn }
    }

    async fn create_user(&mut self, username: &'c str) -> Result<Uuid, AuthError> {
        let user_id = query!(
            r#"
            insert into users (username)
            values ($1)
            returning (id)
        "#,
            username,
        )
        .fetch_one(&mut *self.conn)
        .await?
        .id;
        Ok(user_id)
    }

    async fn create_credentials(
        &mut self,
        user_id: &Uuid,
        hashed_password: String,
    ) -> Result<(), AuthError> {
        query!(
            r#"
                insert into credentials (user_id, login, password)
                values ($1, $2, $3)
            "#,
            user_id,
            self.login,
            hashed_password
        )
        .execute(&mut *self.conn)
        .await?;
        Ok(())
    }

    async fn create_account(
        &mut self,
        hashed_password: String,
        username: &'c str,
    ) -> Result<Uuid, AuthError> {
        let user_id = self.create_user(username).await?;
        self.create_credentials(&user_id, hashed_password).await?;
        Ok(user_id)
    }

    async fn is_new(&mut self) -> Result<bool, AuthError> {
        let is_new = query!(
            r#"
                select * from credentials where login = $1
            "#,
            self.login
        )
        .fetch_optional(&mut *self.conn)
        .await?
        .is_none();
        Ok(is_new)
    }

    async fn verify_credentials(&mut self, password: SecretString) -> Result<Uuid, AuthError> {
        let res = query!(
            r#"
            select users.id, password from credentials
            join users on credentials.user_id = users.id
            where login = $1
        "#,
            self.login
        )
        .fetch_optional(&mut *self.conn)
        .await?
        .ok_or(WrongLoginOrPassword)?;

        let is_verified = verify_pass(password.expose_secret().to_owned(), res.password)?;

        if is_verified {
            return Ok(res.id);
        }
        Err(WrongLoginOrPassword)
    }
}
