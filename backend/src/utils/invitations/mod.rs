pub mod errors;

use sqlx::{query, query_as, PgPool};
use tracing::trace;
use uuid::Uuid;

use crate::routes::{events::models::EventPayload, invitations::models::EventInvitation};

use self::errors::InvitationError;

pub async fn try_create_event_invitation(
    pool: &PgPool,
    invitation: EventInvitation,
) -> Result<(), InvitationError> {
    query!(
        r#"
            INSERT INTO user_events (user_id, event_id, can_edit)
            VALUES ($1, $2, false)
        "#,
        invitation.user_id,
        invitation.event_id,
    )
    .execute(pool)
    .await?;

    trace!(
        "Created user event invitation - event_id: {}",
        invitation.event_id
    );

    Ok(())
}

pub async fn fetch_event_invitations(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<EventPayload>, InvitationError> {
    let res = query_as!(
        EventPayload,
        r#"
            SELECT name, description FROM user_events JOIN events ON user_events.event_id = events.id
            WHERE user_events.user_id = $1
            AND user_events.is_accepted = false
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    trace!("Got {} event invitation(s)", res.len());

    Ok(res)
}

pub async fn accept_event_invitation(
    pool: &PgPool,
    user_id: Uuid,
    event_id: Uuid,
) -> Result<(), InvitationError> {
    query!(
        r#"
            UPDATE user_events
            SET is_accepted = true
            WHERE user_id = $1
            AND event_id = $2
        "#,
        user_id,
        event_id,
    )
    .execute(pool)
    .await?;

    trace!("Accepted event invitation - event_id: {event_id}");

    Ok(())
}

pub async fn reject_event_invitation(
    pool: &PgPool,
    user_id: Uuid,
    event_id: Uuid,
) -> Result<(), InvitationError> {
    query!(
        r#"
            DELETE FROM user_events
            WHERE user_id = $1
            AND event_id = $2
        "#,
        user_id,
        event_id,
    )
    .execute(pool)
    .await?;

    trace!("Rejected event invitation - event_id: {event_id}");

    Ok(())
}
