pub mod errors;

use crate::app_errors::DefaultContext;
use crate::modules::database::PgQuery;
use crate::utils::search::errors::SearchError;
use serde::{Deserialize, Serialize};
use sqlx::{query_as, PgPool};
use tracing::trace;
use uuid::Uuid;

pub struct Search {
    pub text: String,
}

impl Search {
    pub fn new(text: String) -> Self {
        Self { text }
    }
}

pub async fn get_users(pool: &PgPool, text: String) -> Result<Vec<QueryUser>, SearchError> {
    let mut conn = pool.acquire().await.dc()?;
    let mut q = PgQuery::new(Search::new(text), &mut conn);
    Ok(q.search_users().await?)
}

impl<'c> PgQuery<'c, Search> {
    pub async fn search_users(&mut self) -> Result<Vec<QueryUser>, SearchError> {
        let res = query_as!(
            QueryUser,
            r#"
                SELECT id, username, tag FROM users
                WHERE username LIKE CONCAT(CAST($1 AS TEXT), '%')
            "#,
            self.payload.text
        )
        .fetch_all(&mut *self.conn)
        .await
        .dc()?;

        if res.is_empty() {
            trace!(
                "Found no users with usernames starting with {}",
                self.payload.text
            );
        } else {
            trace!(
                "Found {} users with usernames starting with {}",
                res.len(),
                self.payload.text
            );
        }

        Ok(res)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct QueryUser {
    pub id: Uuid,
    pub username: String,
    pub tag: i32,
}
