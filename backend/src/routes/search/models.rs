use crate::utils::search::QueryUser;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Serialize, Deserialize, ToSchema, IntoParams)]
pub struct SearchUsers {
    pub text: String,
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
