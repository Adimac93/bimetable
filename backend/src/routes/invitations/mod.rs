pub mod models;

use axum::{
    debug_handler,
    extract::{Query, State},
    routing::{delete, get, patch, post, put},
    Json, Router,
};
use sqlx::PgPool;
use tracing::debug;
use uuid::Uuid;

use crate::{
    modules::AppState,
    routes::invitations::models::EventInvitation,
    utils::{
        auth::models::Claims,
        invitations::{
            accept_event_invitation, errors::InvitationError, fetch_event_invitations,
            reject_event_invitation, try_create_event_invitation,
        },
    },
};

use super::events::models::EventPayload;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/create", put(create_invitation))
        .route("/fetch", get(fetch_invitations))
        .route("/accept", patch(accept_invitation))
        .route("/reject", delete(reject_invitation))
}

/// Create user event invitation
#[debug_handler]
#[utoipa::path(put, path = "/events/invitations/create", tag = "events", request_body = EventInvitation, responses((status = 200, description = "Created event invitation")))]
async fn create_invitation(
    claims: Claims,
    State(pool): State<PgPool>,
    Json(invitation): Json<EventInvitation>,
) -> Result<(), InvitationError> {
    try_create_event_invitation(&pool, invitation).await?;
    debug!(
        "Created event invitation from user: {} to user: {}",
        claims.user_id, invitation.user_id
    );
    Ok(())
}

/// Fetch all invitations
#[debug_handler]
#[utoipa::path(put, path = "/events/invitations/fetch", tag = "events", responses((status = 200, description = "Fetched event invitations")))]
async fn fetch_invitations(
    claims: Claims,
    State(pool): State<PgPool>,
) -> Result<Json<Vec<EventPayload>>, InvitationError> {
    let invitations = fetch_event_invitations(&pool, claims.user_id).await?;
    debug!(
        "Fetched {} event(s) for user: {}",
        invitations.len(),
        claims.user_id
    );
    Ok(Json(invitations))
}

/// Accept invitation
#[debug_handler]
#[utoipa::path(put, path = "/events/invitations/accept", tag = "events", request_body = Uuid, responses((status = 200, description = "Accepted event invitation")))]
async fn accept_invitation(
    claims: Claims,
    State(pool): State<PgPool>,
    Json(event_id): Json<Uuid>, // query?
) -> Result<(), InvitationError> {
    accept_event_invitation(&pool, claims.user_id, event_id).await?;
    debug!(
        "User: {} accepted invitation for event: {}",
        event_id, claims.user_id
    );
    Ok(())
}

/// Reject invitation
#[debug_handler]
#[utoipa::path(put, path = "/events/invitations/reject", tag = "events", request_body = Uuid, responses((status = 200, description = "Rejected event invitation")))]
async fn reject_invitation(
    claims: Claims,
    State(pool): State<PgPool>,
    Json(event_id): Json<Uuid>, // query?
) -> Result<(), InvitationError> {
    reject_event_invitation(&pool, claims.user_id, event_id).await?;
    debug!("User: {} rejected invitation for event: {}", claims.user_id, event_id);
    Ok(())
}
