pub mod models;
use axum::{
    debug_handler,
    extract::{Path, State},
    routing::{delete, get, patch, post, put},
    Json, Router,
};
use sqlx::PgPool;
use tracing::debug;
use uuid::Uuid;

use crate::routes::invitations::models::{
    CreateDirectInvitation, DirectInvitation, RespondDirectInvitation,
};
use crate::utils::invitations::{
    create_direct_invitation, get_all_direct_invitations, respond_to_direct_invitation,
};
use crate::{
    modules::AppState,
    utils::{auth::models::Claims, invitations::errors::InvitationError},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/create", put(create_direct))
        .route("/fetch", get(fetch_direct))
        .route("/respond/:id", patch(respond_direct))
}

/// Create user event invitation
#[debug_handler]
#[utoipa::path(put, path = "/events/invitations/create", tag = "invitations", request_body = CreateDirectInvitation, responses((status = 200, description = "Created event invitation")))]
async fn create_direct(
    claims: Claims,
    State(pool): State<PgPool>,
    Json(invitation): Json<CreateDirectInvitation>,
) -> Result<(), InvitationError> {
    create_direct_invitation(
        &pool,
        DirectInvitation {
            event_id: invitation.event_id,
            sender_id: claims.user_id,
            receiver_id: invitation.receiver_id,
            can_edit: invitation.can_edit,
        },
    )
    .await?;
    debug!(
        "Created event invitation from user: {} to user: {}",
        claims.user_id, invitation.receiver_id
    );
    Ok(())
}

/// Fetch all invitations
#[debug_handler]
#[utoipa::path(get, path = "/events/invitations/fetch", tag = "invitations", responses((status = 200, body = [DirectInvitation], description = "Fetched event invitations")))]
async fn fetch_direct(
    claims: Claims,
    State(pool): State<PgPool>,
) -> Result<Json<Vec<DirectInvitation>>, InvitationError> {
    let invitations = get_all_direct_invitations(&pool, &claims.user_id).await?;
    debug!(
        "Fetched {} event(s) for user: {}",
        invitations.len(),
        claims.user_id
    );
    Ok(Json(invitations))
}

/// Respond to direct invitation
#[debug_handler]
#[utoipa::path(patch, path = "/events/invitations/respond/{id}", tag = "invitations", request_body = RespondDirectInvitation, responses((status = 200, description = "Responded to direct event invitation")))]
async fn respond_direct(
    claims: Claims,
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(response): Json<RespondDirectInvitation>,
) -> Result<(), InvitationError> {
    respond_to_direct_invitation(&pool, response).await?;
    debug!(
        "User: {} responded ({}) invitation for event: {}",
        claims.user_id, response.is_accepted, id
    );
    Ok(())
}
