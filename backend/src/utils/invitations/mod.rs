pub mod errors;

use sqlx::{query, PgPool};
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

    Ok(())
}

pub async fn fetch_event_invitations(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<EventPayload>, InvitationError> {
    let mut transaction = pool.begin().await?;
    let res = query!(
        r#"
            SELECT name, description FROM user_events JOIN events ON user_events.event_id = events.id
            WHERE user_events.user_id = $1
            AND user_events.is_accepted = false
        "#,
        user_id
    )
    .fetch_all(&mut transaction)
    .await?;

    let res = res
        .into_iter()
        .map(|x| EventPayload {
            name: x.name,
            description: x.description,
        })
        .collect::<Vec<EventPayload>>();

    Ok(res)
}

pub async fn accept_event_invitation(
    pool: &PgPool,
    invitation: EventInvitation,
) -> Result<(), InvitationError> {
    query!(
        r#"
            UPDATE user_events
            SET is_accepted = true
            WHERE user_id = $1
            AND event_id = $2
        "#,
        invitation.user_id,
        invitation.event_id,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn reject_event_invitation(
    pool: &PgPool,
    invitation: EventInvitation,
) -> Result<(), InvitationError> {
    query!(
        r#"
            DELETE FROM user_events
            WHERE user_id = $1
            AND event_id = $2
        "#,
        invitation.user_id,
        invitation.event_id,
    )
    .execute(pool)
    .await?;

    Ok(())
}
