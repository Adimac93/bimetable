pub mod models;
use crate::modules::AppState;
use crate::utils::auth::models::Claims;
use crate::utils::events::errors::EventError;
use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use http::StatusCode;
use serde_json::json;
use sqlx::types::JsonValue;
use sqlx::{types::Uuid, PgPool};
use time::OffsetDateTime;

use crate::modules::database::PgQuery;
use crate::routes::events::models::{
    Event, EventData, EventPayload, Events, OptionalEventData, OverrideEvent, UpdateEvent,
};
use crate::utils::events::{get_many_events, EventQuery};

use self::models::{CreateEvent, GetEventsQuery};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_events).post(create_event))
        .route(
            "/:id",
            get(get_event)
                .put(update_event)
                .delete(delete_event_permanently),
        )
        .route("/override/:id", post(create_event_override))
}

/// Create event
#[utoipa::path(post, path = "/events", tag = "events", request_body = CreateEvent, responses((status = 200, description = "Created event")))]
pub async fn create_event(
    claims: Claims,
    State(pool): State<PgPool>,
    Json(body): Json<CreateEvent>,
) -> Result<Json<JsonValue>, EventError> {
    let mut conn = pool.acquire().await?;
    let mut q = PgQuery::new(EventQuery {}, &mut *conn);
    let event_id = q.create_event(claims.user_id, body).await?;
    Ok(Json(json!({ "event_id": event_id })))
}

/// Get many events
#[utoipa::path(get, path = "/events", tag = "events", params(GetEventsQuery), responses((status = 200, body = Events, description = "Fetched many events")))]
async fn get_events(
    claims: Claims,
    State(pool): State<PgPool>,
    Query(query): Query<GetEventsQuery>,
) -> Result<Json<Events>, EventError> {
    let events = get_many_events(
        claims.user_id,
        query.starts_at,
        query.ends_at,
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
    Path(event_id): Path<Uuid>,
) -> Result<Json<Event>, EventError> {
    let mut conn = pool.acquire().await?;
    let mut q = PgQuery::new(EventQuery {}, &mut *conn);
    let event = q
        .get_event(claims.user_id, event_id)
        .await?
        .ok_or(EventError::NotFound)?;

    Ok(Json(event))
}

/// Update event
#[utoipa::path(put, path = "/events/{id}", tag = "events", request_body = UpdateEvent)]
async fn update_event(
    claims: Claims,
    State(pool): State<PgPool>,
    Path(event_id): Path<Uuid>,
    Json(body): Json<UpdateEvent>,
) -> Result<StatusCode, EventError> {
    let mut conn = pool.acquire().await?;
    let mut q = PgQuery::new(EventQuery {}, &mut *conn);
    q.update_event(claims.user_id, event_id, body.data).await?;

    Ok(StatusCode::OK)
}

/// Delete event temporarily
#[utoipa::path(put, path = "/events/{id}", tag = "events")]
async fn delete_event_temporarily(
    claims: Claims,
    State(pool): State<PgPool>,
    Path(event_id): Path<Uuid>,
) -> Result<StatusCode, EventError> {
    let mut conn = pool.acquire().await?;
    let mut q = PgQuery::new(EventQuery {}, &mut *conn);
    q.temp_delete(claims.user_id, event_id).await?;
    Ok(StatusCode::OK)
}

/// Delete event permanently
#[utoipa::path(delete, path = "/events/{id}", tag = "events")]
async fn delete_event_permanently(
    claims: Claims,
    State(pool): State<PgPool>,
    Path(event_id): Path<Uuid>,
) -> Result<StatusCode, EventError> {
    let mut conn = pool.acquire().await?;
    let mut q = PgQuery::new(EventQuery {}, &mut *conn);
    q.perm_delete(claims.user_id, event_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Create event override
#[utoipa::path(post, path = "/events/{id}", tag = "events", request_body = OverrideEvent)]
async fn create_event_override(
    claims: Claims,
    State(pool): State<PgPool>,
    Path(event_id): Path<Uuid>,
    Json(body): Json<OverrideEvent>,
) -> Result<StatusCode, EventError> {
    let mut conn = pool.begin().await?;
    let mut q = PgQuery::new(EventQuery {}, &mut *conn);
    let is_owned = q.is_owned_event(claims.user_id, event_id).await?;
    if !is_owned {
        return Err(EventError::NotFound);
    }

    q.create_override(claims.user_id, event_id, body).await?;
    Ok(StatusCode::OK)
}
