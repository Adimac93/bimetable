use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Deserialize, Debug, ToSchema, Clone, Copy)]
pub struct EventInvitation {
    pub user_id: Uuid,
    pub event_id: Uuid,
}
