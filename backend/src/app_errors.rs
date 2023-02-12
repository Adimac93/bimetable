use crate::utils::auth::errors::AuthError;
use crate::utils::events::errors::EventError;
use anyhow::Context;
use axum::response::IntoResponse;
use thiserror::Error;
use tracing::error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    AuthError(#[from] AuthError),
    #[error(transparent)]
    EventError(#[from] EventError),
}

// TODO: server error backtrace
impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AppError::AuthError(e) => e.into_response(),
            AppError::EventError(e) => e.into_response(),
        }
    }
}

pub trait DefaultContext<C, T, E>: Context<T, E> {
    fn dc(self) -> anyhow::Result<T>
    where
        Self: Sized,
    {
        self.context("No context provided")
    }
}

impl<C, T, E> DefaultContext<C, T, E> for C
where
    C: Context<T, E> {}
