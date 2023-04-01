use crate::routes::events::models::{Event, EventFilter, EventPayload, EventPrivileges};
use crate::utils::search::{QueryEvent, QueryUser};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Serialize, Deserialize, ToSchema, IntoParams)]
pub struct SearchUsers {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<i32>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SearchUsersResult {
    pub id: Uuid,
    pub username: String,
    pub tag: i32,
}

impl From<QueryUser> for SearchUsersResult {
    fn from(val: QueryUser) -> Self {
        Self {
            id: val.id,
            username: val.username,
            tag: val.tag,
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema, IntoParams)]
pub struct SearchEvents {
    pub text: String,
    pub user_id: Uuid,
    pub filter: EventFilter,
}

impl From<QueryEvent> for Event {
    fn from(val: QueryEvent) -> Self {
        let (is_owned, can_edit) = match val.privileges {
            EventPrivileges::Owned => (true, true),
            EventPrivileges::Shared { can_edit: x } => (false, x),
        };

        Self {
            payload: EventPayload {
                name: val.name,
                description: val.description,
            },
            recurrence_rule: val.recurrence_rule,
            entries_start: val.entries_start,
            entries_end: val.entries_end,
            is_owned,
            can_edit,
        }
    }
}
