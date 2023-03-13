use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Deserialize, Debug, ToSchema, Clone, Copy)]
pub struct CreateDirectInvitation {
    pub event_id: Uuid,
    pub receiver_id: Uuid,
    pub can_edit: bool,
}

#[derive(Deserialize, Serialize, Debug, ToSchema, Clone, Copy)]
pub struct DirectInvitation {
    pub event_id: Uuid,
    pub sender_id: Uuid,
    pub receiver_id: Uuid,
    pub can_edit: bool,
}

#[derive(Deserialize, Debug, ToSchema, Clone, Copy)]
pub struct RespondDirectInvitation {
    pub event_id: Uuid,
    pub sender_id: Uuid,
    pub receiver_id: Uuid,
    pub is_accepted: bool,
}
