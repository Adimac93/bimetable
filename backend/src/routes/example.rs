use crate::modules::database::PgPool;
use crate::modules::AppState;
use axum::extract::State;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::Router;
use sqlx::query;

/// [Stateful routers](https://docs.rs/axum/latest/axum/extract/struct.State.html#combining-stateful-routers)
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(handler))
        .route("/uuid", get(db_handler))
}

async fn handler() -> impl IntoResponse {
    Html("Hello")
}

async fn db_handler(State(pool): State<PgPool>) -> impl IntoResponse {
    let res = query!(
        r#"
            select * from gen_random_uuid()
        "#
    )
    .fetch_one(&pool)
    .await
    .unwrap()
    .gen_random_uuid
    .unwrap();

    Html(format!("Random uuid: {res}"))
}
