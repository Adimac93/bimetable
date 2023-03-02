use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct EventInvitation {
    pub user_id: Uuid,
    pub event_id: Uuid,
}
