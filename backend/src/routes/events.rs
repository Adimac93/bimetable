use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::{
    query,
    query_as,
    // maybe instead import `OffsetDateTime` from `time`?
    types::{time::OffsetDateTime, Uuid},
    PgPool,
};
use time::serde::timestamp;

use crate::modules::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_events).put(put_event))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Event {
    pub id: Uuid,
    #[serde(with = "timestamp")]
    pub starts_at: OffsetDateTime,
    #[serde(with = "timestamp")]
    pub ends_at: OffsetDateTime,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetEventsQuery {
    #[serde(with = "timestamp")]
    starts_at: OffsetDateTime,
    #[serde(with = "timestamp")]
    ends_at: OffsetDateTime,
}

async fn get_events(
    Query(query): Query<GetEventsQuery>,
    State(pool): State<PgPool>,
) -> Result<Json<Vec<Event>>, StatusCode> {
    let events = query_as!(
        Event,
        r#"
            SELECT *
            FROM events
            WHERE starts_at > $1 AND ends_at < $2;
        "#,
        query.starts_at,
        query.ends_at,
    )
    .fetch_all(&pool)
    .await
    // TODO: proper error handling
    .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(Json(events))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateEvent {
    #[serde(with = "timestamp")]
    pub starts_at: OffsetDateTime,
    #[serde(with = "timestamp")]
    pub ends_at: OffsetDateTime,
    pub name: String,
}

async fn put_event(
    State(pool): State<PgPool>,
    Json(body): Json<CreateEvent>,
) -> Result<(StatusCode, Json<Uuid>), StatusCode> {
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
    .await
    // TODO: proper error handling
    .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?
    .id;

    Ok((StatusCode::CREATED, Json(id)))
}
