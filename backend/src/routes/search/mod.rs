pub mod models;

use crate::modules::AppState;
use crate::routes::search::models::{SearchUsers, SearchUsersResult};
use crate::utils::auth::models::Claims;
use crate::utils::search::errors::SearchError;
use crate::utils::search::get_users;
use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use sqlx::PgPool;
use tracing::debug;

pub fn router() -> Router<AppState> {
    Router::new().route("/users", get(search_users))
}

/// Search users
#[utoipa::path(get, path = "/search/users", tag = "search", params(SearchUsers), responses((status = 200, description = "Received users", body = SearchUsersResult)))]
pub async fn search_users(
    _claims: Claims,
    State(pool): State<PgPool>,
    Query(q): Query<SearchUsers>,
) -> Result<Json<Vec<SearchUsersResult>>, SearchError> {
    let search_res: Vec<SearchUsersResult> = get_users(&pool, q.text)
        .await?
        .into_iter()
        .map(|x| SearchUsersResult::from(x))
        .collect();

    if search_res.is_empty() {
        debug!("Found no users with user search");
    } else {
        debug!("Found {} user(s) with user search", search_res.len());
    }

    Ok(Json(search_res))
}
