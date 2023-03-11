pub mod additions;
pub mod errors;
pub mod models;
use self::additions::validate_usernames;
use crate::config::tokens::JwtSettings;
use crate::modules::database::PgQuery;
use crate::utils::auth::additions::{hash_pass, random_username_tag, verify_pass};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use errors::*;
use models::*;
use secrecy::{ExposeSecret, SecretString};
use sqlx::{query, Acquire, PgConnection, Postgres};
use std::collections::HashSet;
use tracing::{debug, trace};
use uuid::Uuid;

pub async fn try_register_user<'c>(
    acq: impl Acquire<'c, Database = Postgres>,
    login: &str,
    password: SecretString,
    username: &str,
) -> Result<Uuid, AuthError> {
    let mut transaction = acq.begin().await?;

    let mut user = PgQuery::new(AuthUser::new(&login), &mut transaction);

    if !user.is_new().await? {
        trace!("User with a specified name already exists");

        return Err(AuthError::UserAlreadyExists);
    }

    if login.trim().is_empty() {
        trace!("Attempted to register with empty login");
        return Err(AuthError::MissingCredential);
    }

    if password.expose_secret().trim().is_empty() {
        trace!("Attempted to register with empty password");
        return Err(AuthError::MissingCredential);
    }

    if username.trim().is_empty() {
        trace!("Attempted to register with empty username");
        return Err(AuthError::MissingCredential);
    }

    validate_usernames(&login, &username)?;

    let tag = random_username_tag(user.get_username_tags(&username).await?)
        .ok_or(AuthError::TagOverflow)?;

    if !additions::pass_is_strong(password.expose_secret(), &[&login]) {
        trace!("Attempted to register with weak password");
        return Err(AuthError::WeakPassword);
    }

    let hashed_pass = hash_pass(password.expose_secret().to_owned())?;

    let user_id = user.create_account(hashed_pass, &username, tag).await?;

    transaction.commit().await?;

    Ok(user_id)
}

pub async fn verify_user_credentials<'c>(
    conn: &mut PgConnection,
    login: &str,
    password: SecretString,
) -> Result<Uuid, AuthError> {
    debug!("Verifying credentials");
    if login.trim().is_empty() {
        trace!("Attempted to login the user with empty login");
        return Err(AuthError::MissingCredential)?;
    }

    if password.expose_secret().trim().is_empty() {
        trace!("Attempted to login the user with empty password");
        return Err(AuthError::MissingCredential)?;
    }

    let mut q = PgQuery::new(AuthUser::new(login), conn);
    let user_id = q.verify_credentials(password).await?;

    Ok(user_id)
}

pub fn generate_token_cookies(
    user_id: Uuid,
    login: &str,
    secrets: JwtSettings,
    jar: CookieJar,
) -> Result<CookieJar, AuthError> {
    let access_cookie = generate_jwt_in_cookie(
        Claims::new(user_id, login, secrets.access.0.expiration),
        &secrets.access.0.token,
    )?;

    let refresh_cookie = generate_jwt_in_cookie(
        RefreshClaims::new(user_id, login, secrets.refresh.0.expiration),
        &secrets.refresh.0.token,
    )?;

    trace!("JWT cookies generated successfully");
    Ok(jar.add(access_cookie).add(refresh_cookie))
}

fn generate_jwt_in_cookie<'a, T: AuthToken<'a>>(
    payload: T,
    secret: &SecretString,
) -> Result<Cookie<'a>, AuthError> {
    let token = payload.generate_jwt(secret)?;
    let access_cookie = T::generate_cookie(token);
    trace!("JWT: {access_cookie}");

    Ok(access_cookie)
}

pub struct AuthUser<'c> {
    login: &'c str,
}

impl<'c> AuthUser<'c> {
    fn new(login: &'c str) -> Self {
        Self { login }
    }
}

impl<'c> PgQuery<'c, AuthUser<'c>> {
    async fn create_user(&mut self, username: &str, tag: i32) -> Result<Uuid, AuthError> {
        let user_id = query!(
            r#"
            insert into users (username, tag)
            values ($1, $2)
            returning (id)
        "#,
            username,
            tag
        )
        .fetch_one(&mut *self.conn)
        .await?
        .id;
        trace!("Created user");
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
            self.payload.login,
            hashed_password
        )
        .execute(&mut *self.conn)
        .await?;
        trace!("Created credentials");
        Ok(())
    }

    async fn create_account(
        &mut self,
        hashed_password: String,
        username: &str,
        tag: i32,
    ) -> Result<Uuid, AuthError> {
        let user_id = self.create_user(username, tag).await?;
        self.create_credentials(&user_id, hashed_password).await?;
        trace!("Created user account successfully");
        Ok(user_id)
    }

    async fn is_new(&mut self) -> Result<bool, AuthError> {
        let is_new = query!(
            r#"
                select * from credentials where login = $1
            "#,
            self.payload.login
        )
        .fetch_optional(&mut *self.conn)
        .await?
        .is_none();
        if is_new {
            trace!("User with this login does not exist");
        } else {
            trace!("User with this login exists");
        }
        Ok(is_new)
    }

    async fn verify_credentials(&mut self, password: SecretString) -> Result<Uuid, AuthError> {
        let res = query!(
            r#"
            select users.id, password from credentials
            join users on credentials.user_id = users.id
            where login = $1
        "#,
            self.payload.login
        )
        .fetch_optional(&mut *self.conn)
        .await?
        .ok_or_else(|| {
            trace!("Wrong login or password");
            AuthError::WrongLoginOrPassword
        })?;

        let is_verified = verify_pass(password.expose_secret().to_owned(), res.password)?;

        if is_verified {
            trace!("Login and password verified");
            return Ok(res.id);
        }
        trace!("Wrong login or password");
        Err(AuthError::WrongLoginOrPassword)
    }

    async fn get_username_tags(&mut self, username: &str) -> Result<HashSet<i32>, AuthError> {
        let res = query!(
            r#"
            SELECT tag
            FROM users
            WHERE username = $1
        "#,
            username
        )
        .fetch_all(&mut *self.conn)
        .await?;

        Ok(res.iter().map(|rec| rec.tag).collect())
    }
}
