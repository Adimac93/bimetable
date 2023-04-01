use crate::routes::{
    auth::models::*, auth::*, events::models::*, events::*, invitations::models::*, invitations::*,
    search::models::*, search::*,
};
use crate::utils::events::models::*;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
info(title = "Bimetable", description = "Bimetable calendar", ),
paths(
post_register_user,
post_login_user,
post_logout_user,
post_refresh_user_token,
protected_zone,
create_event,
get_events,
get_event,
delete_event_permanently,
update_event,
create_event_override,
update_edit_privileges,
update_event_owner,
disconnect_user_from_event,
disconnect_owner_from_event,
create_direct,
fetch_direct,
respond_direct,
search_users,
search_events,
),
components(schemas(
CreateEvent,
EventData,
EventPayload,
RecurrenceRule,
RecurrenceEndsAt,
RecurrenceEndsAt,
TimeRules,
EventFilter,
Event,
Events,
Entry,
Override,
OptionalEventData,
OverrideEvent,
UpdateEvent,
LoginCredentials,
RegisterCredentials,
CreateEventResult,
UpdateEditPrivilege,
UpdateEventOwner,
NewEventOwner,
SearchUsers,
SearchUsersResult,
SearchEvents,
CreateDirectInvitation,
RespondDirectInvitation
)),
tags((name = "auth"),(name = "events"),(name = "event-ownership"),(name = "invitations"),(name = "search"))
)]
pub struct ApiDoc;
