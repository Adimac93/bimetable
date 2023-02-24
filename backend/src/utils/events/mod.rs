use crate::modules::database::PgQuery;
use crate::routes::events::models::CreateEvent;
use crate::utils::events::models::{Event, EventRules};
use serde::Serialize;
use sqlx::types::time::OffsetDateTime;
use sqlx::{query, query_as, Acquire, Connection};
use uuid::Uuid;

pub mod additions;
pub mod calculations;
pub mod count_to_until;
pub mod errors;
pub mod event_range;
pub mod models;

#[derive(Serialize)]
pub struct EventPayload {
    owned: Vec<UserEvent>,
    shared: Vec<SharedEvent>,
    overrides: Vec<Override>,
}

#[derive(Serialize)]
pub struct UserEvent {
    id: Uuid,
    name: String,
    description: Option<String>,
    starts_at: OffsetDateTime,
    ends_at: OffsetDateTime,
    deleted_at: Option<OffsetDateTime>,
    recurrence_rule: Option<sqlx::types::Json<EventRules>>,
}

#[derive(Serialize)]
pub struct SharedEvent {
    id: Uuid,
    name: String,
    description: Option<String>,
    starts_at: OffsetDateTime,
    ends_at: OffsetDateTime,
    deleted_at: Option<OffsetDateTime>,
    recurrence_rule: Option<sqlx::types::Json<EventRules>>,
    can_edit: bool,
}

#[derive(Serialize)]
pub struct Override {
    event_id: Uuid,
    override_starts_at: OffsetDateTime,
    override_ends_at: OffsetDateTime,
    created_at: OffsetDateTime,
    name: Option<String>,
    description: Option<String>,
    starts_at: Option<OffsetDateTime>,
    ends_at: Option<OffsetDateTime>,
    deleted_at: Option<OffsetDateTime>,
}

pub struct EventQuery {}

impl<'c> PgQuery<'c, EventQuery> {
    pub async fn get_many(
        &mut self,
        user_id: Uuid,
        starts_at: OffsetDateTime,
        ends_at: OffsetDateTime,
    ) -> sqlx::Result<EventPayload> {
        let events = query_as!(
            UserEvent,
            r#"
                SELECT id, name, description, starts_at, ends_at, deleted_at, recurrence_rule as "recurrence_rule: _" 
                FROM events
                WHERE owner_id = $1 AND starts_at >= $2 AND ends_at <= $3
            "#,
            user_id,
            starts_at,
            ends_at
        )
            .fetch_all(&mut *self.conn)
            .await?;

        let shared_events = query_as!(
            SharedEvent,
            r#"
                SELECT id, name, description, starts_at, ends_at, deleted_at, recurrence_rule as "recurrence_rule: _", can_edit
                FROM user_events
                JOIN events ON user_events.event_id = events.id
                WHERE owner_id = $1 AND starts_at >= $2 AND ends_at <= $3
            "#,
            user_id,
            starts_at,
            ends_at
        )
            .fetch_all(&mut *self.conn)
            .await?;

        let ids: Vec<Uuid> = events
            .iter()
            .map(|event| event.id)
            .chain(shared_events.iter().map(|event| event.id))
            .collect();

        let overrides = query_as!(
            Override,
            r#"
                SELECT event_id, override_starts_at, override_ends_at, created_at, name, description, starts_at, ends_at, deleted_at
                FROM event_overrides
                WHERE event_id in ($1)
            "#,
            ids as _
        )
            .fetch_all(&mut *self.conn)
            .await?;

        Ok(EventPayload {
            owned: events,
            shared: shared_events,
            overrides,
        })
    }

    pub async fn create(
        &mut self,
        user_id: Uuid,
        event: CreateEvent
    ) -> sqlx::Result<Uuid> {
        let id = query!(
            r#"
                INSERT INTO events (owner_id, name, description, starts_at, ends_at, recurrence_rule)
                VALUES
                ($1, $2, $3, $4, $5, $6)
                RETURNING id
            "#,
            user_id,
            event.data.name,
            event.data.description,
            event.data.starts_at,
            event.data.ends_at,
            event.recurrence_rule as _
        )
            .fetch_one(&mut *self.conn)
            .await?
            .id;

        Ok(id)
    }

    pub async fn get(&mut self, user_id: Uuid, id: Uuid) -> sqlx::Result<Option<UserEvent>> {
        let event = query_as!(
            UserEvent,
            r#"
                SELECT id, name, description, starts_at, ends_at, recurrence_rule as "recurrence_rule: sqlx::types::Json<EventRules>", deleted_at
                FROM events
                WHERE owner_id = $1 AND id = $2;
            "#,
            user_id,
            id,
        )
            .fetch_optional(&mut *self.conn)
            .await?;

        Ok(event)
    }
    pub async fn create_override(&mut self, user_id: Uuid, ovr: Override) -> sqlx::Result<()> {
        query!(
            r#"
                INSERT INTO event_overrides (event_id, override_starts_at, override_ends_at, name, description, starts_at, ends_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            ovr.event_id,
            ovr.override_starts_at,
            ovr.override_ends_at,
            ovr.name,
            ovr.description,
            ovr.starts_at,
            ovr.ends_at
        ).execute(&mut *self.conn).await?;

        Ok(())
    }
    pub async fn update_event(&mut self, user_id: Uuid, event: Event) -> sqlx::Result<()> {
        query!(
            r#"
                UPDATE events
                SET
                name = $1,
                description = $2,
                starts_at = $3,
                ends_at = $4
                WHERE owner_id = $5 AND id = $6
            "#,
            event.name,
            event.description,
            event.starts_at,
            event.ends_at,
            user_id,
            event.id,
        )
        .execute(&mut *self.conn)
        .await?;

        Ok(())
    }

    pub async fn temp_delete(&mut self, user_id: Uuid, event_id: Uuid) -> sqlx::Result<()> {
        let now = OffsetDateTime::now_utc();
        query!(
            r#"
                UPDATE events
                SET
                deleted_at = $1
                WHERE owner_id = $2 AND id = $3
            "#,
            now,
            user_id,
            event_id
        )
        .execute(&mut *self.conn)
        .await?;

        Ok(())
    }
    pub async fn perm_delete(&mut self, user_id: Uuid, event_id: Uuid) -> sqlx::Result<()> {
        query!(
            r#"
                DELETE FROM events
                WHERE owner_id = $1 AND id = $2
            "#,
            user_id,
            event_id
        )
        .execute(&mut *self.conn)
        .await?;

        Ok(())
    }
}
