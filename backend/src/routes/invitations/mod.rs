pub mod models;

use axum::{
    debug_handler,
    extract::State,
    routing::{delete, patch, post, put},
    Json, Router,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    modules::AppState,
    routes::invitations::models::EventInvitation,
    utils::invitations::{
        accept_event_invitation, errors::InvitationError, fetch_event_invitations,
        reject_event_invitation, try_create_event_invitation,
    },
};

use super::events::models::EventPayload;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/create", put(create_invitation))
        .route("/fetch", post(fetch_invitations))
        .route("/accept", patch(accept_invitation))
        .route("/reject", delete(reject_invitation))
}

#[debug_handler]
async fn create_invitation(
    State(pool): State<PgPool>,
    Json(invitation): Json<EventInvitation>,
) -> Result<(), InvitationError> {
    Ok(try_create_event_invitation(&pool, invitation).await?)
}

#[debug_handler]
async fn fetch_invitations(
    State(pool): State<PgPool>,
    Json(user_id): Json<Uuid>,
) -> Result<Json<Vec<EventPayload>>, InvitationError> {
    Ok(Json(fetch_event_invitations(&pool, user_id).await?))
}

#[debug_handler]
async fn accept_invitation(
    State(pool): State<PgPool>,
    Json(invitation): Json<EventInvitation>,
) -> Result<(), InvitationError> {
    Ok(accept_event_invitation(&pool, invitation).await?)
}

#[debug_handler]
async fn reject_invitation(
    State(pool): State<PgPool>,
    Json(invitation): Json<EventInvitation>,
) -> Result<(), InvitationError> {
    Ok(reject_event_invitation(&pool, invitation).await?)
}
