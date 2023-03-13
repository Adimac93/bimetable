pub mod errors;

use crate::modules::database::PgQuery;
use sqlx::{query, query_as, Acquire, PgPool};
use tracing::trace;
use uuid::Uuid;

use crate::routes::events::models::EventPayload;
use crate::routes::invitations::models::{
    CreateDirectInvitation, DirectInvitation, RespondDirectInvitation,
};

use self::errors::InvitationError;

struct Invitation;

impl<'c> PgQuery<'c, Invitation> {
    async fn get_all_direct(
        &mut self,
        receiver_id: &Uuid,
    ) -> Result<Vec<DirectInvitation>, InvitationError> {
        let res = query_as!(
            DirectInvitation,
            r#"
            SELECT * FROM user_event_invitations
            WHERE receiver_id = $1
        "#,
            receiver_id
        )
        .fetch_all(&mut *self.conn)
        .await?;

        trace!("Got {} direct invitations", res.len());

        Ok(res)
    }
    async fn get_one_direct(
        &mut self,
        event_id: &Uuid,
        sender_id: &Uuid,
        receiver_id: &Uuid,
    ) -> Result<Option<DirectInvitation>, InvitationError> {
        let res = query_as!(
            DirectInvitation,
            r#"
            SELECT * FROM user_event_invitations
            WHERE event_id = $1 AND sender_id = $2 AND receiver_id = $3
        "#,
            event_id,
            sender_id,
            receiver_id
        )
        .fetch_optional(&mut *self.conn)
        .await?;

        Ok(res)
    }

    async fn delete_remaining_direct_for_event(
        &mut self,
        event_id: &Uuid,
        receiver_id: &Uuid,
    ) -> Result<(), InvitationError> {
        let affected = query!(
            r#"
            DELETE FROM user_event_invitations
            WHERE event_id = $1 AND receiver_id = $2
        "#,
            event_id,
            receiver_id
        )
        .execute(&mut *self.conn)
        .await?
        .rows_affected();

        trace!(
            "Deleted {affected} remaining direct invitations for event {:?}",
            event_id
        );

        Ok(())
    }

    async fn was_sent_direct(
        &mut self,
        event_id: &Uuid,
        sender_id: &Uuid,
        receiver_id: &Uuid,
    ) -> Result<bool, InvitationError> {
        let was_sent = query!(
            r#"
            SELECT * FROM user_event_invitations
            WHERE event_id = $1 AND sender_id = $2 AND receiver_id = $3
        "#,
            event_id,
            sender_id,
            receiver_id
        )
        .fetch_optional(&mut *self.conn)
        .await?
        .is_some();

        if was_sent {
            trace!("Direct invitation was previously sent");
        }

        Ok(was_sent)
    }

    async fn can_edit_direct(
        &mut self,
        event_id: &Uuid,
        sender_id: &Uuid,
        receiver_id: &Uuid,
    ) -> Result<bool, InvitationError> {
        let can_edit = query!(
            r#"
            SELECT can_edit FROM user_event_invitations
            WHERE event_id = $1 AND sender_id = $2 AND receiver_id = $3
        "#,
            event_id,
            sender_id,
            receiver_id
        )
        .fetch_one(&mut *self.conn)
        .await?
        .can_edit;

        Ok(can_edit)
    }

    async fn create_direct(
        &mut self,
        event_id: &Uuid,
        sender_id: &Uuid,
        receiver_id: &Uuid,
        can_edit: bool,
    ) -> Result<(), InvitationError> {
        let res = query!(
            r#"
                INSERT INTO user_event_invitations (event_id, sender_id, receiver_id, can_edit)
                VALUES ($1, $2, $3, $4)
            "#,
            event_id,
            sender_id,
            receiver_id,
            can_edit
        )
        .execute(&mut *self.conn)
        .await?;

        trace!("Created user event invitation for event: {}", event_id);

        Ok(())
    }

    async fn delete_direct(
        &mut self,
        event_id: &Uuid,
        sender_id: &Uuid,
        receiver_id: &Uuid,
    ) -> Result<(), InvitationError> {
        query!(
            r#"
            DELETE FROM user_event_invitations
            WHERE event_id = $1 AND sender_id = $2 AND receiver_id = $3
        "#,
            event_id,
            sender_id,
            receiver_id
        )
        .execute(&mut *self.conn)
        .await?;

        Ok(())
    }

    async fn create_user_event(
        &mut self,
        event_id: &Uuid,
        receiver_id: &Uuid,
        can_edit: bool,
    ) -> Result<(), InvitationError> {
        query!(
            r#"
            INSERT INTO user_events (user_id, event_id, can_edit)
            VALUES ($1, $2, $3)
        "#,
            event_id,
            receiver_id,
            can_edit
        )
        .execute(&mut *self.conn)
        .await?;

        Ok(())
    }
}

pub async fn get_all_direct_invitations(
    pool: &PgPool,
    user_id: &Uuid,
) -> Result<Vec<DirectInvitation>, InvitationError> {
    let mut conn = pool.acquire().await?;
    let mut q = PgQuery::new(Invitation, &mut conn);
    let invitations = q.get_all_direct(user_id).await?;
    Ok(invitations)
}

pub async fn create_direct_invitation(
    pool: &PgPool,
    inv: DirectInvitation,
) -> Result<(), InvitationError> {
    let mut transaction = pool.begin().await?;
    let mut q = PgQuery::new(Invitation, &mut transaction);
    if !q
        .was_sent_direct(&inv.event_id, &inv.sender_id, &inv.receiver_id)
        .await?
    {
        q.create_direct(
            &inv.event_id,
            &inv.sender_id,
            &inv.receiver_id,
            inv.can_edit,
        )
        .await?;
    } else {
        trace!("Direct invitation already created");
    }

    transaction.commit().await?;
    Ok(())
}

pub async fn respond_to_direct_invitation(
    pool: &PgPool,
    response: RespondDirectInvitation,
) -> Result<(), InvitationError> {
    let mut transaction = pool.begin().await?;
    let mut q = PgQuery::new(Invitation, &mut transaction);

    if let Some(inv) = q
        .get_one_direct(
            &response.event_id,
            &response.sender_id,
            &response.receiver_id,
        )
        .await?
    {
        if response.is_accepted {
            trace!("Invitation was accepted");
            let can_edit = q
                .can_edit_direct(
                    &response.event_id,
                    &response.sender_id,
                    &response.receiver_id,
                )
                .await?;
            q.create_user_event(&response.event_id, &response.receiver_id, can_edit)
                .await?;
            trace!("Created user event");
        }
        q.delete_direct(
            &response.event_id,
            &response.sender_id,
            &response.receiver_id,
        )
        .await?;
        trace!("Deleted direct invitation");
        q.delete_remaining_direct_for_event(&response.event_id, &response.receiver_id)
            .await?;

        transaction.commit().await?;
        return Ok(());
    }

    trace!("Direct invitation missing");
    Err(InvitationError::Missing)
}
