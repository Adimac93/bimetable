use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EventError {
    #[error("Not Found")]
    NotFound,
    #[error("Wrong event bounds")]
    WrongEventBounds,
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl IntoResponse for EventError {
    fn into_response(self) -> axum::response::Response {
        let status_code = match &self {
            EventError::NotFound => StatusCode::NOT_FOUND,
            EventError::WrongEventBounds => StatusCode::BAD_REQUEST,
            EventError::Unexpected(e) => {
                tracing::error!("Internal server error: {e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        let info = match self {
            EventError::Unexpected(_) => "Unexpected server error".to_string(),
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
