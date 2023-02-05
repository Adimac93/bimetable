pub mod models;

use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use http::StatusCode;
use sqlx::{query, query_as, types::Uuid, PgPool};
use crate::{modules::AppState, utils::events::models::Event};
use crate::utils::auth::models::Claims;
use crate::utils::events::errors::EventError;

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
) -> Result<Json<Vec<Event>>, EventError> {
    let events = query_as!(
        Event,
        r#"
            SELECT id, owner_id, name, starts_at, ends_at, recurrence_rule as "recurrence_rule: _"
            FROM events
            WHERE owner_id = $1 AND starts_at >= $2 AND ends_at <= $3;
        "#,
        claims.user_id,
        query.starts_at,
        query.ends_at,
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(events))
}

async fn put_new_event(
    claims: Claims,
    State(pool): State<PgPool>,
    Json(body): Json<CreateEvent>,
) -> Result<(StatusCode, Json<Uuid>), EventError> {
    let id = query!(
        r#"
            INSERT INTO events (name, owner_id, starts_at, ends_at, recurrence_rule)
            VALUES
            ($1, $2, $3, $4, $5)
            RETURNING id;
        "#,
        body.name,
        claims.user_id,
        body.starts_at,
        body.ends_at,
        sqlx::types::Json(body.recurrence_rule) as _
    )
    .fetch_one(&pool)
    .await?
    .id;

    Ok((StatusCode::CREATED, Json(id)))
}



async fn get_event(
    claims: Claims,
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    
) -> Result<Json<Event>, EventError> {
    let event = query_as!(
        Event,
        r#"
            SELECT id, owner_id, name, starts_at, ends_at, recurrence_rule as "recurrence_rule: _"
            FROM events
            WHERE owner_id = $1 AND id = $2;
        "#,
        claims.user_id,
        id,
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(EventError::NotFound)?;

    Ok(Json(event))
}

async fn put_event(
    claims: Claims,
    State(pool): State<PgPool>,
    Json(body): Json<Event>,
) -> Result<StatusCode, EventError> {
    query!(
        r#"
            UPDATE events SET
            name = $1,
            owner_id = $2,
            starts_at = $3,
            ends_at = $4,
            recurrence_rule = $5
            WHERE owner_id = $6 AND id = $7
        "#,
        body.name,
        body.owner_id,
        body.starts_at,
        body.ends_at,
        sqlx::types::Json(body.recurrence_rule) as _,
        claims.user_id,
        body.id,
    )
    .execute(&pool)
    .await?;

    Ok(StatusCode::OK)
}

async fn delete_event(
    claims: Claims,
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, EventError> {
    query!(
        r#"
            DELETE FROM events
            WHERE owner_id = $1 AND id = $2;
        "#,
        claims.user_id,
        id
    )
    .execute(&pool)
    .await?;

    Ok(StatusCode::NO_CONTENT)
}
