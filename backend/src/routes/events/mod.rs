pub mod models;

use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use http::StatusCode;
use sqlx::{query, query_as, types::Uuid, PgPool};

use crate::modules::database::PgQuery;
use crate::modules::AppState;
use crate::utils::auth::models::Claims;
use crate::utils::events::errors::EventError;
use crate::utils::events::EventQuery;

use self::models::{CreateEvent, Event, GetEventsQuery};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_events).put(put_new_event))
        .route("/:id", get(get_event).put(put_event).delete(delete_event))
}

async fn get_events(
    Query(query): Query<GetEventsQuery>,
    State(pool): State<PgPool>,
) -> Result<Json<Vec<Event>>, EventError> {
    let mut conn = pool.acquire().await?;
    let mut q = PgQuery::new(EventQuery {}, &mut *conn);
    let events = q.get_many(query.starts_at, query.ends_at).await?;
    Ok(Json(events))
}

async fn put_new_event(
    State(pool): State<PgPool>,
    Json(body): Json<CreateEvent>,
) -> Result<(StatusCode, Json<Uuid>), EventError> {
    let mut conn = pool.acquire().await?;
    let mut q = PgQuery::new(EventQuery {}, &mut *conn);
    let event_id = q.create(body.name, body.starts_at, body.ends_at).await?;

    Ok((StatusCode::CREATED, Json(event_id)))
}

async fn get_event(
    Path(id): Path<Uuid>,
    State(pool): State<PgPool>,
) -> Result<Json<Event>, EventError> {
    let mut conn = pool.acquire().await?;
    let mut q = PgQuery::new(EventQuery {}, &mut *conn);
    let event = q.get(id).await?.ok_or(EventError::NotFound)?;

    Ok(Json(event))
}

async fn put_event(
    State(pool): State<PgPool>,
    Json(body): Json<Event>,
) -> Result<StatusCode, EventError> {
    let mut conn = pool.acquire().await?;
    let mut q = PgQuery::new(EventQuery {}, &mut *conn);
    q.update(body).await?;

    Ok(StatusCode::OK)
}

async fn delete_event(
    Path(id): Path<Uuid>,
    State(pool): State<PgPool>,
) -> Result<StatusCode, EventError> {
    let mut conn = pool.acquire().await?;
    let mut q = PgQuery::new(EventQuery {}, &mut *conn);
    q.delete(id).await?;

    Ok(StatusCode::NO_CONTENT)
}
