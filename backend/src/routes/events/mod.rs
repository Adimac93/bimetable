pub mod models;
use crate::utils::auth::models::Claims;
use crate::utils::events::errors::EventError;
use crate::{modules::AppState, validation::ValidateContent};
use axum::debug_handler;
use axum::{
    extract::{Path, Query, State},
    routing::{get, patch, post},
    Json, Router,
};
use http::StatusCode;
use sqlx::{types::Uuid, PgPool};
use tracing::debug;

use crate::routes::events::models::{CreateEventResult, Event, Events, OverrideEvent, UpdateEvent};
use crate::utils::events::exe::{
    create_new_event, create_one_event_override, delete_one_event_permanently,
    delete_one_event_temporally, get_many_events, get_one_event, set_event_ownership,
    update_one_event, update_user_editing_privileges,
};
use crate::utils::events::models::TimeRange;

use self::models::{CreateEvent, GetEventsQuery, UpdateEditPrivilege, UpdateEventOwner};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_events).put(create_event))
        .route(
            "/:id",
            get(get_event)
                .patch(update_event)
                .delete(delete_event_permanently),
        )
        .route("/override/:id", patch(create_event_override))
        .route("/set-edit/:id", patch(update_edit_privileges))
        .route("/set-owner/:id", patch(update_event_owner))
}

/// Create event
#[utoipa::path(put, path = "/events", tag = "events", request_body = CreateEvent, responses((status = 200, description = "Created event", body = CreateEventResult)))]
pub async fn create_event(
    claims: Claims,
    State(pool): State<PgPool>,
    Json(body): Json<CreateEvent>,
) -> Result<(StatusCode, Json<CreateEventResult>), EventError> {
    body.validate_content()?;
    let event_id = create_new_event(&pool, claims.user_id, body).await?;
    debug!("Created event: {}", event_id);

    Ok((StatusCode::CREATED, Json(CreateEventResult { event_id })))
}

/// Get many events
#[utoipa::path(get, path = "/events", tag = "events", params(GetEventsQuery), responses((status = 200, body = Events, description = "Fetched many events")))]
async fn get_events(
    claims: Claims,
    State(pool): State<PgPool>,
    Query(query): Query<GetEventsQuery>,
) -> Result<Json<Events>, EventError> {
    query.validate_content()?;
    let events = get_many_events(
        claims.user_id,
        TimeRange::new(query.starts_at, query.ends_at),
        query.filter,
        &pool,
    )
    .await?;
    Ok(Json(events))
}

/// Get event
#[utoipa::path(get, path = "/events/{id}", tag = "events", responses((status = 200, body = Event)))]
async fn get_event(
    claims: Claims,
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<Event>, EventError> {
    let event = get_one_event(&pool, claims.user_id, id).await?;

    Ok(Json(event))
}

/// Update event
#[utoipa::path(patch, path = "/events/{id}", tag = "events", request_body = UpdateEvent)]
async fn update_event(
    claims: Claims,
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateEvent>,
) -> Result<StatusCode, EventError> {
    body.validate_content()?;
    update_one_event(&pool, claims.user_id, body, id).await?;
    debug!("Updated event: {}", id);

    Ok(StatusCode::NO_CONTENT)
}

/// Delete event temporarily
#[utoipa::path(patch, path = "/events/{id}", tag = "events")]
async fn delete_event_temporarily(
    claims: Claims,
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, EventError> {
    delete_one_event_temporally(&pool, claims.user_id, id).await?;
    debug!("Deleted event temporally: {}", id);

    Ok(StatusCode::NO_CONTENT)
}

/// Delete event permanently
#[utoipa::path(delete, path = "/events/{id}", tag = "events")]
async fn delete_event_permanently(
    claims: Claims,
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, EventError> {
    delete_one_event_permanently(&pool, claims.user_id, id).await?;
    debug!("Deleted event permanently: {}", id);

    Ok(StatusCode::NO_CONTENT)
}

/// Create event override
#[utoipa::path(put, path = "/events/override/{id}", tag = "events", request_body = OverrideEvent)]
async fn create_event_override(
    claims: Claims,
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(body): Json<OverrideEvent>,
) -> Result<StatusCode, EventError> {
    body.validate_content()?;
    create_one_event_override(&pool, claims.user_id, body, id).await?;
    debug!("Created override on event: {}", id);

    Ok(StatusCode::CREATED)
}

/// Update editing privileges
#[utoipa::path(patch, path = "/events/set-edit/{id}", tag = "events", request_body = UpdateEditPrivilege)]
async fn update_edit_privileges(
    claims: Claims,
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateEditPrivilege>,
) -> Result<(), EventError> {
    update_user_editing_privileges(&pool, claims.user_id, body.user_id, body.can_edit, id).await?;
    debug!(
        "Updated editing privileges for user {} and event {id} to {}",
        body.user_id, body.can_edit
    );

    Ok(())
}

/// Update event owner
#[utoipa::path(patch, path = "/events/set-owner/{id}", tag = "events", request_body = UpdateEventOwner)]
async fn update_event_owner(
    claims: Claims,
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateEventOwner>,
) -> Result<(), EventError> {
    set_event_ownership(&pool, claims.user_id, body.user_id, id).await?;
    debug!("Updated owner of event {id} to {}", body.user_id);

    Ok(())
}
