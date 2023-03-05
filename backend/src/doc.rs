use crate::routes::{
    auth::models::*, auth::*, events::models::*, events::*, invitations::models::*, invitations::*,
};
use crate::utils::{auth::models::*, events::models::*};
use utoipa::openapi::security::{
    AuthorizationCode, Flow, HttpAuthScheme, HttpBuilder, OAuth2, Password, Scopes,
};
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};

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
create_invitation,
fetch_invitations,
accept_invitation,
reject_invitation,
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
EventInvitation,
CreateEventResult,
)),
tags((name = "auth"),(name = "events"), (name = "invitations"))
)]
pub struct ApiDoc;
