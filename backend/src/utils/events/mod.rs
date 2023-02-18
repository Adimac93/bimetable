use crate::modules::database::PgQuery;
use crate::routes::events::models::Event;
use sqlx::types::time::OffsetDateTime;
use sqlx::{query, query_as};
use uuid::Uuid;

pub mod errors;
pub mod models;
pub mod modification;

pub struct EventQuery {}

impl<'c> PgQuery<'c, EventQuery> {
    pub async fn create(
        &mut self,
        name: String,
        starts_at: OffsetDateTime,
        ends_at: OffsetDateTime,
    ) -> sqlx::Result<Uuid> {
        let id = query!(
            r#"
                INSERT INTO events (starts_at, ends_at, name)
                VALUES
                ($1, $2, $3)
                RETURNING id;
            "#,
            starts_at,
            ends_at,
            name,
        )
        .fetch_one(&mut *self.conn)
        .await?
        .id;

        Ok(id)
    }

    pub async fn get(&mut self, id: Uuid) -> sqlx::Result<Option<Event>> {
        let event = query_as!(
            Event,
            r#"
                SELECT *
                FROM events
                WHERE id = $1;
            "#,
            id,
        )
        .fetch_optional(&mut *self.conn)
        .await?;

        Ok(event)
    }

    pub async fn get_many(
        &mut self,
        starts_at: OffsetDateTime,
        ends_at: OffsetDateTime,
    ) -> sqlx::Result<Vec<Event>> {
        let events = query_as!(
            Event,
            r#"
            SELECT *
            FROM events
            WHERE starts_at >= $1 AND ends_at <= $2;
        "#,
            starts_at,
            ends_at,
        )
        .fetch_all(&mut *self.conn)
        .await?;

        Ok(events)
    }

    pub async fn update(&mut self, event: Event) -> sqlx::Result<()> {
        query!(
            r#"
                UPDATE events SET
                starts_at = $2,
                ends_at = $3,
                name = $4
                WHERE id = $1;
            "#,
            event.id,
            event.starts_at,
            event.ends_at,
            event.name,
        )
        .execute(&mut *self.conn)
        .await?;

        Ok(())
    }

    pub async fn delete(&mut self, id: Uuid) -> sqlx::Result<()> {
        query!(
            r#"
                DELETE FROM events
                WHERE id = $1;
            "#,
            id
        )
        .execute(&mut *self.conn)
        .await?;

        Ok(())
    }
}
