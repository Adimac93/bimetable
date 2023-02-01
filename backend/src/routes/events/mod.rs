pub mod models;

use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use http::StatusCode;
use sqlx::{query, query_as, types::Uuid, PgPool};

use crate::modules::AppState;
use crate::utils::auth::models::Claims;
use crate::utils::events::errors::EventError;

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
    let events = query_as!(
        Event,
        r#"
            SELECT *
            FROM events
            WHERE starts_at >= $1 AND ends_at <= $2;
        "#,
        query.starts_at,
        query.ends_at,
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(events))
}

async fn put_new_event(
    State(pool): State<PgPool>,
    Json(body): Json<CreateEvent>,
) -> Result<(StatusCode, Json<Uuid>), EventError> {
    let id = query!(
        r#"
            INSERT INTO events (starts_at, ends_at, name)
            VALUES
            ($1, $2, $3)
            RETURNING id;
        "#,
        body.starts_at,
        body.ends_at,
        body.name,
    )
    .fetch_one(&pool)
    .await?
    .id;

    Ok((StatusCode::CREATED, Json(id)))
}

async fn get_event(
    Path(id): Path<Uuid>,
    State(pool): State<PgPool>,
) -> Result<Json<Event>, EventError> {
    let event = query_as!(
        Event,
        r#"
            SELECT *
            FROM events
            WHERE id = $1;
        "#,
        id,
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(EventError::NotFound)?;

    Ok(Json(event))
}

async fn put_event(
    State(pool): State<PgPool>,
    Json(body): Json<Event>,
) -> Result<StatusCode, EventError> {
    query!(
        r#"
            UPDATE events SET
            starts_at = $2,
            ends_at = $3,
            name = $4
            WHERE id = $1;
        "#,
        body.id,
        body.starts_at,
        body.ends_at,
        body.name,
    )
    .execute(&pool)
    .await?;

    Ok(StatusCode::OK)
}

async fn delete_event(
    Path(id): Path<Uuid>,
    State(pool): State<PgPool>,
) -> Result<StatusCode, EventError> {
    query!(
        r#"
            DELETE FROM events
            WHERE id = $1;
        "#,
        id
    )
    .execute(&pool)
    .await?;

    Ok(StatusCode::NO_CONTENT)
}
