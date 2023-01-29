use crate::utils::auth::errors::AuthError;
use axum::{
    async_trait,
    extract::{self, FromRequestParts},
};
use http::{request::Parts, Request};
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

// #[async_trait]
// impl<S> FromRequestParts<S> for TokenExtractors
// {
//     type Rejection = AuthError;

//     async fn from_request_parts(req: &mut Parts, state: S) -> Result<Self, Self::Rejection> {
//
//     }
// }

#[async_trait]
impl<S> FromRequestParts<S> for TokenExtractors {
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Ok(parts
            .extensions
            .get::<Self>()
            .expect("Failed to get jwt secret extension")
            .clone())
    }
}
