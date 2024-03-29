use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use thiserror::Error;
use validator::ValidationErrors;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("Missing credential")]
    MissingCredential,
    #[error("Password is too weak")]
    WeakPassword,
    #[error("Incorrect email or password")]
    WrongLoginOrPassword,
    #[error("Invalid or expired token")]
    InvalidToken,
    #[error("Invalid login or username")]
    InvalidUsername(#[from] ValidationErrors),
    #[error("To many users named like you")]
    TagOverflow,
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        let status_code = match &self {
            AuthError::UserAlreadyExists => StatusCode::BAD_REQUEST,
            AuthError::MissingCredential => StatusCode::BAD_REQUEST,
            AuthError::WeakPassword => StatusCode::BAD_REQUEST,
            AuthError::WrongLoginOrPassword => StatusCode::UNAUTHORIZED,
            AuthError::InvalidToken => StatusCode::UNAUTHORIZED,
            AuthError::InvalidUsername(_e) => StatusCode::BAD_REQUEST,
            AuthError::TagOverflow => StatusCode::BAD_REQUEST,
            AuthError::Unexpected(e) => {
                tracing::error!("Internal server error: {e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        let info = match self {
            AuthError::InvalidUsername(_) => "Invalid username".to_string(),
            AuthError::Unexpected(_) => "Unexpected server error".to_string(),
            _ => self.to_string(),
        };

        (status_code, Json(json!({ "error_info": info }))).into_response()
    }
}

impl From<sqlx::Error> for AuthError {
    fn from(e: sqlx::Error) -> Self {
        Self::Unexpected(anyhow::Error::from(e))
    }
}
