use crate::routes::{auth::models::*, auth::*, events::models::*, events::*};
use crate::utils::{auth::models::*, events::models::*};
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};

#[derive(OpenApi)]
#[openapi(
    info(title = "Bimetable", description = "Bimetable calendar",),
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
        OptionalEventData,
        OverrideEvent,
        UpdateEvent,
    ))
)]
pub struct ApiDoc;