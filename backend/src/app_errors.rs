use crate::utils::auth::errors::AuthError;
use axum::response::IntoResponse;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    AuthError(#[from] AuthError),
}

// TODO: server error backtrace
impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AppError::AuthError(e) => return e.into_response(),
        };
    }
}
