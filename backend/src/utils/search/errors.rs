use axum::response::IntoResponse;
use axum::Json;
use http::StatusCode;
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SearchError {
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl IntoResponse for SearchError {
    fn into_response(self) -> axum::response::Response {
        let status_code = match &self {
            SearchError::Unexpected(e) => {
                tracing::error!("Internal server error: {e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        let info = match self {
            SearchError::Unexpected(_) => "Unexpected server error".to_string(),
        };

        (status_code, Json(json!({ "error_info": info }))).into_response()
    }
}
