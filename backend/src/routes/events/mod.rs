pub mod models;
use crate::utils::auth::models::Claims;
use crate::utils::events::errors::EventError;
use crate::{modules::AppState, validation::ValidateContent};
use axum::{
    extract::{Path, Query, State},
    routing::{get, patch, post},
    Json, Router,
};
use http::StatusCode;
use serde_json::json;
use sqlx::types::JsonValue;
use sqlx::{types::Uuid, PgPool};
use tracing::debug;

use crate::modules::database::PgQuery;
use crate::routes::events::models::{
    Event, EventData, EventPayload, Events, OptionalEventData, OverrideEvent, UpdateEvent,
};
use crate::utils::events::models::TimeRange;
use crate::utils::events::{get_many_events, EventQuery};

use self::models::{CreateEvent, GetEventsQuery};

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
}

/// Create event
#[utoipa::path(put, path = "/events", tag = "events", request_body = CreateEvent, responses((status = 200, description = "Created event")))]
pub async fn create_event(
    claims: Claims,
    State(pool): State<PgPool>,
    Json(body): Json<CreateEvent>,
) -> Result<Json<JsonValue>, EventError> {
    body.validate_content()?;
    let mut conn = pool.acquire().await?;
    let mut q = PgQuery::new(EventQuery::new(claims.user_id), &mut *conn);
    let event_id = q.create_event(body).await?;
    debug!("Created event: {}", event_id);

    Ok(Json(json!({ "event_id": event_id })))
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
        pool,
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
    let mut conn = pool.acquire().await?;
    let mut q = PgQuery::new(EventQuery::new(claims.user_id), &mut *conn);
    let event = q.get_event(id).await?.ok_or(EventError::NotFound)?;

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
    let mut conn = pool.acquire().await?;
    let mut q = PgQuery::new(EventQuery::new(claims.user_id), &mut *conn);
    q.update_event(id, body.data).await?;
    debug!("Updated event: {}", id);

    Ok(StatusCode::OK)
}

/// Delete event temporarily
#[utoipa::path(patch, path = "/events/{id}", tag = "events")]
async fn delete_event_temporarily(
    claims: Claims,
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, EventError> {
    let mut conn = pool.acquire().await?;
    let mut q = PgQuery::new(EventQuery::new(claims.user_id), &mut *conn);
    q.temp_delete(id).await?;
    debug!("Deleted event temporalily: {}", id);

    Ok(StatusCode::OK)
}

/// Delete event permanently
#[utoipa::path(delete, path = "/events/{id}", tag = "events")]
async fn delete_event_permanently(
    claims: Claims,
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, EventError> {
    let mut conn = pool.acquire().await?;
    let mut q = PgQuery::new(EventQuery::new(claims.user_id), &mut *conn);
    q.perm_delete(id).await?;
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
    let mut conn = pool.begin().await?;
    let mut q = PgQuery::new(EventQuery::new(claims.user_id), &mut *conn);
    let is_owned = q.is_owned_event(id).await?;
    if !is_owned {
        return Err(EventError::NotFound);
    }

    q.create_override(id, body).await?;
    debug!("Created override on event: {}", id);

    Ok(StatusCode::OK)
}
