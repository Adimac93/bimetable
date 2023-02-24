pub mod models;

use crate::utils::auth::models::Claims;
use crate::utils::events::errors::EventError;
use crate::{modules::AppState, utils::events::models::Event};
use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use http::StatusCode;
use sqlx::{types::Uuid, PgPool};
use time::{Duration, OffsetDateTime};

use crate::modules::database::PgQuery;
use crate::utils::events::{EventPayload, EventQuery, UserEvent};

use self::models::{CreateEvent, GetEventsQuery};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_events).put(put_new_event))
        .route("/:id", get(get_event).put(put_event).delete(delete_event))
}

async fn get_events(
    claims: Claims,
    State(pool): State<PgPool>,
    Query(query): Query<GetEventsQuery>,
) -> Result<Json<EventPayload>, EventError> {
    let mut conn = pool.begin().await?;
    let mut q = PgQuery::new(EventQuery {}, &mut *conn);
    // for dev purposes
    let starts_at = query.starts_at.unwrap_or(OffsetDateTime::UNIX_EPOCH);
    let ends_at = query
        .ends_at
        .unwrap_or(OffsetDateTime::now_utc().saturating_add(Duration::days(365)));
    let events = q.get_many(claims.user_id, starts_at, ends_at).await?;
    Ok(Json(events))
}

async fn put_new_event(
    claims: Claims,
    State(pool): State<PgPool>,
    Json(body): Json<CreateEvent>,
) -> Result<(StatusCode, Json<Uuid>), EventError> {
    let mut conn = pool.acquire().await?;
    let mut q = PgQuery::new(EventQuery {}, &mut *conn);
    let event_id = q
        .create(
            claims.user_id,
            body
        )
        .await?;

    Ok((StatusCode::CREATED, Json(event_id)))
}

async fn get_event(
    claims: Claims,
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<UserEvent>, EventError> {
    let mut conn = pool.acquire().await?;
    let mut q = PgQuery::new(EventQuery {}, &mut *conn);
    let event = q
        .get(claims.user_id, id)
        .await?
        .ok_or(EventError::NotFound)?;

    Ok(Json(event))
}

async fn put_event(
    claims: Claims,
    State(pool): State<PgPool>,
    Json(body): Json<Event>,
) -> Result<StatusCode, EventError> {
    let mut conn = pool.acquire().await?;
    let mut q = PgQuery::new(EventQuery {}, &mut *conn);
    q.update_event(claims.user_id, body).await?;

    Ok(StatusCode::OK)
}

async fn delete_event(
    claims: Claims,
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, EventError> {
    let mut conn = pool.acquire().await?;
    let mut q = PgQuery::new(EventQuery {}, &mut *conn);
    q.perm_delete(claims.user_id, id).await?;

    Ok(StatusCode::NO_CONTENT)
}
