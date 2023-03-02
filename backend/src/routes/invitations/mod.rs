pub mod models;

use axum::{
    debug_handler,
    extract::{Query, State},
    routing::{delete, get, patch, post, put},
    Json, Router,
};
use sqlx::PgPool;
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
    _claims: Claims,
    State(pool): State<PgPool>,
    Json(invitation): Json<EventInvitation>,
) -> Result<(), InvitationError> {
    Ok(try_create_event_invitation(&pool, invitation).await?)
}

/// Fetch all invitations
#[debug_handler]
#[utoipa::path(put, path = "/events/invitations/fetch", tag = "events", responses((status = 200, description = "Fetched event invitations")))]
async fn fetch_invitations(
    claims: Claims,
    State(pool): State<PgPool>,
) -> Result<Json<Vec<EventPayload>>, InvitationError> {
    Ok(Json(fetch_event_invitations(&pool, claims.user_id).await?))
}

/// Accept invitation
#[debug_handler]
#[utoipa::path(put, path = "/events/invitations/accept", tag = "events", request_body = Uuid, responses((status = 200, description = "Accepted event invitation")))]
async fn accept_invitation(
    claims: Claims,
    State(pool): State<PgPool>,
    Json(event_id): Json<Uuid>, // query?
) -> Result<(), InvitationError> {
    Ok(accept_event_invitation(&pool, claims.user_id, event_id).await?)
}

/// Reject invitation
#[debug_handler]
#[utoipa::path(put, path = "/events/invitations/reject", tag = "events", request_body = Uuid, responses((status = 200, description = "Rejected event invitation")))]
async fn reject_invitation(
    claims: Claims,
    State(pool): State<PgPool>,
    Json(event_id): Json<Uuid>, // query?
) -> Result<(), InvitationError> {
    Ok(reject_event_invitation(&pool, claims.user_id, event_id).await?)
}
