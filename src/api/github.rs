use actix_web::{get, HttpResponse};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct StargazerResponse {
    stargazers: u32,
    has_notifs: bool,
    message: String,
}

impl Default for StargazerResponse {
    fn default() -> Self {
        Self {
            stargazers: 0,
            has_notifs: false,
            message: "This endpoint is deprecated.".to_string(),
        }
    }
}

/// Get Stork Github Star count
///
/// _DEPRECATED_: This API endpoint used to return the number of Github stars
/// that <https://github.com/jameslittle230/stork> had.
#[utoipa::path(
    responses(
        (status=200, description = "Success response", body = inline(StargazerResponse))
    ),
    tag="Generic",
)]
#[get("/github/stork-stars")]
pub async fn get_github_stork_stars() -> HttpResponse {
    HttpResponse::Ok().json(StargazerResponse::default())
}
