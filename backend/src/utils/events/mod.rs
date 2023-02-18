use crate::modules::database::PgQuery;
use sqlx::types::time::OffsetDateTime;
use sqlx::{query, query_as};
use uuid::Uuid;
use crate::utils::events::models::{Event, EventRules};

pub mod errors;
pub mod models;
pub mod modification;

pub struct EventQuery {}

impl<'c> PgQuery<'c, EventQuery> {
    pub async fn create(
        &mut self,
        user_id: Uuid,
        name: String,
        description: String,
        starts_at: Option<OffsetDateTime>,
        ends_at: Option<OffsetDateTime>,
    ) -> sqlx::Result<Uuid> {
        let id = query!(
            r#"
                INSERT INTO events (owner_id, name, description, starts_at, ends_at)
                VALUES
                ($1, $2, $3, $4, $5)
                RETURNING id
            "#,
            user_id,
            name,
            description,
            starts_at,
            ends_at,
        )
        .fetch_one(&mut *self.conn)
        .await?
        .id;

        Ok(id)
    }

    pub async fn get(&mut self, user_id: Uuid, id: Uuid) -> sqlx::Result<Option<Event>> {
        let event = query_as!(
            Event,
            r#"
                SELECT id, owner_id, name, description, starts_at, ends_at, recurrence_rule as "recurrence_rule: sqlx::types::Json<EventRules>"
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

    pub async fn get_many(
        &mut self,
        user_id: Uuid,
        starts_at: Option<OffsetDateTime>,
        ends_at: Option<OffsetDateTime>,
    ) -> sqlx::Result<Vec<Event>> {
        let events = query_as!(
            Event,
            r#"
            SELECT id, owner_id, name, description, starts_at, ends_at, recurrence_rule as "recurrence_rule: sqlx::types::Json<EventRules>"
            FROM events
            WHERE owner_id = $1 AND starts_at >= $2 AND ends_at <= $3;
        "#,
            user_id,
            starts_at,
            ends_at,
        )
        .fetch_all(&mut *self.conn)
        .await?;

        Ok(events)
    }

    pub async fn update(&mut self, user_id: Uuid, event: Event) -> sqlx::Result<()> {
        query!(
            r#"
                UPDATE events SET
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

    pub async fn delete(&mut self, user_id: Uuid, id: Uuid) -> sqlx::Result<()> {
        query!(
            r#"
                DELETE FROM events
                WHERE owner_id = $1 AND id = $2
            "#,
            user_id,
            id
        )
        .execute(&mut *self.conn)
        .await?;

        Ok(())
    }
}
