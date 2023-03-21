pub mod errors;

use crate::app_errors::DefaultContext;
use crate::modules::database::PgQuery;
use crate::routes::events::models::{
    Event, EventFilter, EventPayload, EventPrivileges, Events, Override,
};
use crate::routes::search::models::{SearchEvents, SearchUsers};
use crate::utils::events::models::{EntriesSpan, RecurrenceRule, RecurrenceRuleKind, TimeRange};
use crate::utils::events::{map_events, EventQuery, QOverride};
use crate::utils::search::errors::SearchError;
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use sqlx::{query, query_as, PgPool};
use time::OffsetDateTime;
use tracing::trace;
use uuid::Uuid;

pub struct Search {
    pub text: String,
}

impl<'c> PgQuery<'c, Search> {
    pub async fn search_users(&mut self, tag: Option<i32>) -> Result<Vec<QueryUser>, SearchError> {
        let res = query_as!(
            QueryUser,
            r#"
                SELECT id, username, tag FROM users
                WHERE LOWER(username) LIKE CONCAT(LOWER(CAST($1 AS TEXT)), '%')
                AND (CAST($2 AS INT) IS NULL OR tag = $2)
            "#,
            self.payload.text.to_lowercase(),
            tag
        )
        .fetch_all(&mut *self.conn)
        .await
        .dc()?;

        let tag_log = match tag {
            Some(t) => format!(" and with tag {}", t),
            None => "".to_string(),
        };

        if res.is_empty() {
            trace!(
                "Found no users with usernames starting with {}{}",
                self.payload.text,
                tag_log
            );
        } else {
            trace!(
                "Found {} users with usernames starting with {}{}",
                res.len(),
                self.payload.text,
                tag_log
            );
        }

        Ok(res)
    }

    pub async fn get_owned_events(
        &mut self,
        user_id: Uuid,
    ) -> Result<Vec<QueryEvent>, SearchError> {
        let events = query!(
            r#"
                SELECT id, name, description, starts_at, COALESCE(until, ends_at) AS entries_end, recurrence AS "recurrence: Option<sqlx::types::Json<RecurrenceRuleKind>>", until, count, interval AS "interval: Option<i32>"
                FROM events
                LEFT JOIN recurrence_rules ON recurrence_rules.event_id = id
                WHERE owner_id = $1
                AND deleted_at IS NULL
                AND LOWER(events.name) LIKE CONCAT(LOWER(CAST($2 AS TEXT)), '%')
                ORDER BY starts_at ASC
            "#,
            user_id,
            self.payload.text.to_lowercase(),
        ).fetch_all(&mut *self.conn).await.dc()?;

        if !events.is_empty() {
            trace!(
                "Got {} owned events with names starting with {}",
                events.len(),
                self.payload.text
            );
        } else {
            trace!(
                "No owned events with names starting with {}",
                self.payload.text
            );
        }

        let events = events
            .into_iter()
            .map(|event| QueryEvent {
                id: event.id,
                name: event.name,
                description: event.description,
                entries_start: event.starts_at,
                entries_end: event.entries_end,
                recurrence_rule: RecurrenceRule::from_db_data(
                    event.recurrence,
                    event.until,
                    event.count,
                    event.interval,
                ),
                privileges: EventPrivileges::Owned,
            })
            .collect();

        Ok(events)
    }

    pub async fn get_shared_events(
        &mut self,
        user_id: Uuid,
    ) -> Result<Vec<QueryEvent>, SearchError> {
        let events = query!(
            r#"
                SELECT id, name, description, starts_at, COALESCE(until, ends_at) AS entries_end, recurrence AS "recurrence: Option<sqlx::types::Json<RecurrenceRuleKind>>", can_edit, until, count, interval AS "interval: Option<i32>"
                FROM user_events
                JOIN events ON user_events.event_id = events.id
                LEFT JOIN recurrence_rules ON recurrence_rules.event_id = id
                WHERE user_id = $1 AND deleted_at IS NULL AND owner_id <> $1
                AND LOWER(events.name) LIKE CONCAT(LOWER(CAST($2 AS TEXT)), '%')
                ORDER BY events.starts_at ASC
            "#,
            user_id,
            self.payload.text.to_lowercase()
        )
            .fetch_all(&mut *self.conn)
            .await.dc()?;

        dbg!(&events);

        if !events.is_empty() {
            trace!(
                "Got {} shared events with names starting with {}",
                events.len(),
                self.payload.text
            );
        } else {
            trace!(
                "No shared events with names starting with {}",
                self.payload.text
            );
        }

        let events = events
            .into_iter()
            .map(|event| QueryEvent {
                id: event.id,
                name: event.name,
                description: event.description,
                entries_start: event.starts_at,
                entries_end: event.entries_end,
                recurrence_rule: RecurrenceRule::from_db_data(
                    event.recurrence,
                    event.until,
                    event.count,
                    event.interval,
                ),
                privileges: EventPrivileges::Shared {
                    can_edit: event.can_edit,
                },
            })
            .collect();

        Ok(events)
    }
}

impl Search {
    pub fn new(text: String) -> Self {
        Self { text }
    }
}

pub async fn get_users(pool: &PgPool, search: SearchUsers) -> Result<Vec<QueryUser>, SearchError> {
    let mut conn = pool.acquire().await.dc()?;
    let mut q = PgQuery::new(Search::new(search.text), &mut conn);
    Ok(q.search_users(search.tag).await?)
}

pub async fn search_shared(
    q: &mut PgQuery<'_, Search>,
    user_id: Uuid,
) -> Result<Vec<QueryEvent>, SearchError> {
    q.get_shared_events(user_id).await
}

pub async fn search_owned(
    q: &mut PgQuery<'_, Search>,
    user_id: Uuid,
) -> Result<Vec<QueryEvent>, SearchError> {
    q.get_owned_events(user_id).await
}

pub async fn search_many_events(
    pool: &PgPool,
    search: SearchEvents,
) -> Result<Vec<QueryEvent>, SearchError> {
    let mut conn = pool.acquire().await.dc()?;
    let mut q = PgQuery::new(Search::new(search.text), &mut conn);

    match search.filter {
        EventFilter::All => {
            let mut owned = search_owned(&mut q, search.user_id).await?;
            let shared = search_shared(&mut q, search.user_id).await?;

            owned.extend(shared);
            owned.sort_by_key(|x| x.entries_start);

            Ok(owned)
        }
        EventFilter::Owned => search_owned(&mut q, search.user_id).await,
        EventFilter::Shared => search_shared(&mut q, search.user_id).await,
    }
}

#[derive(Debug, PartialEq)]
pub struct QueryUser {
    pub id: Uuid,
    pub username: String,
    pub tag: i32,
}

#[derive(Debug)]
pub struct QueryEvent {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub entries_start: OffsetDateTime,
    pub entries_end: Option<OffsetDateTime>,
    pub recurrence_rule: Option<RecurrenceRule>,
    pub privileges: EventPrivileges,
}
