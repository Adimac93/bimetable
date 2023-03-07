use crate::validation::ValidateContentError;
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EventError {
    #[error("Event data rejected with validation")]
    InvalidData(#[from] ValidateContentError),
    #[error("Not Found")]
    NotFound,
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl IntoResponse for EventError {
    fn into_response(self) -> axum::response::Response {
        let status_code = match &self {
            EventError::InvalidData(e) => StatusCode::from(e),
            EventError::NotFound => StatusCode::NOT_FOUND,
            EventError::Unexpected(e) => {
                tracing::error!("Internal server error: {e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        let info = match self {
            EventError::Unexpected(_) => "Unexpected server error".to_string(),
            EventError::InvalidData(e) => match &e {
                ValidateContentError::Expected(content) => {
                    format!("{}: {}", e.to_string(), content)
                }
                ValidateContentError::Unexpected(_) => "Unexpected server error".to_string(),
            },
            _ => self.to_string(),
        };

        (status_code, Json(json!({ "error_info": info }))).into_response()
    }
}

impl From<sqlx::Error> for EventError {
    fn from(e: sqlx::Error) -> Self {
        Self::Unexpected(anyhow::Error::from(e))
    }
}
