use crate::modules::database::PgQuery;
use crate::routes::events::models::{
    CreateEvent, Event, EventFilter, Events, OverrideEvent, UpdateEvent,
};
use crate::utils::events::errors::EventError;
use crate::utils::events::models::TimeRange;
use crate::utils::events::{get_owned, get_shared, EventQuery};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn get_many_events(
    user_id: Uuid,
    search_range: TimeRange,
    filter: EventFilter,
    pool: &PgPool,
) -> Result<Events, EventError> {
    let mut conn = pool.begin().await?;
    let mut q = PgQuery::new(EventQuery { user_id }, &mut conn);
    return match filter {
        EventFilter::All => {
            let owned_events = get_owned(search_range, &mut q).await?;
            let shared_events = get_shared(search_range, &mut q).await?;

            Ok(owned_events.merge(shared_events))
        }
        EventFilter::Owned => Ok(get_owned(search_range, &mut q).await?),
        EventFilter::Shared => Ok(get_shared(search_range, &mut q).await?),
    };
}

pub async fn create_new_event(
    pool: &PgPool,
    user_id: Uuid,
    body: CreateEvent,
) -> Result<Uuid, EventError> {
    let mut conn = pool.acquire().await?;
    let mut q = PgQuery::new(EventQuery::new(user_id), &mut conn);
    let event_id = q.create_event(body).await?;

    Ok(event_id)
}

pub async fn get_one_event(
    pool: &PgPool,
    user_id: Uuid,
    event_id: Uuid,
) -> Result<Event, EventError> {
    let mut conn = pool.acquire().await?;
    let mut q = PgQuery::new(EventQuery::new(user_id), &mut conn);
    let event = q.get_event(event_id).await?.ok_or(EventError::NotFound)?;

    Ok(event)
}

pub async fn update_one_event(
    pool: &PgPool,
    user_id: Uuid,
    body: UpdateEvent,
    event_id: Uuid,
) -> Result<(), EventError> {
    let mut conn = pool.acquire().await?;
    let mut q = PgQuery::new(EventQuery::new(user_id), &mut conn);
    if q.is_owner(event_id).await? || q.can_edit(event_id).await? {
        return q.update_event(event_id, body.data).await;
    }
    Err(EventError::NotFound)
}

pub async fn delete_one_event_temporally(
    pool: &PgPool,
    user_id: Uuid,
    event_id: Uuid,
) -> Result<(), EventError> {
    let mut conn = pool.acquire().await?;
    let mut q = PgQuery::new(EventQuery::new(user_id), &mut conn);
    q.temp_delete(event_id).await?;
    Ok(())
}

pub async fn create_one_event_override(
    pool: &PgPool,
    user_id: Uuid,
    body: OverrideEvent,
    event_id: Uuid,
) -> Result<(), EventError> {
    let mut conn = pool.begin().await?;
    let mut q = PgQuery::new(EventQuery::new(user_id), &mut conn);
    let is_owned = q.is_owned_event(event_id).await?;
    if !is_owned {
        return Err(EventError::NotFound);
    }

    q.create_override(event_id, body).await?;
    Ok(())
}

pub async fn delete_one_event_permanently(
    pool: &PgPool,
    user_id: Uuid,
    event_id: Uuid,
) -> Result<(), EventError> {
    let mut conn = pool.acquire().await?;
    let mut q = PgQuery::new(EventQuery::new(user_id), &mut conn);
    if q.is_owner(event_id).await? {
        return q.perm_delete(event_id).await;
    }
    Err(EventError::NotFound)
}
