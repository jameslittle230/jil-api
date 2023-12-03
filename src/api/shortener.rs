use actix_web::{get, post, web, Either, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::{
    error::ApiError,
    shortener::{
        entry::Entry,
        queries::{list_shortlink_entries, put_shortlink_entry},
    },
};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct CreateEntryForm {
    pub shortname: String,
    pub longurl: String,
}

#[post("/shortener/entries")]
pub(crate) async fn create_entry(
    state: web::Data<crate::AppState>,
    payload: Either<web::Json<CreateEntryForm>, web::Form<CreateEntryForm>>,
) -> Result<HttpResponse, ApiError> {
    let shortener_entry = Entry::try_from(payload.into_inner())?;
    put_shortlink_entry(&state.dynamodb, &shortener_entry).await?;
    Ok(HttpResponse::Ok().json(&shortener_entry))
}

#[get("/shortener/entries")]
pub(crate) async fn list_entries(
    state: web::Data<crate::AppState>,
) -> Result<HttpResponse, ApiError> {
    let (_, entries) = list_shortlink_entries(&state.dynamodb).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({ "items": entries })))
}

// Get single entry
// #[get("/shortener/entries/{id}")]

// Update single entry
// #[post("/shortener/entries/{id}")]

// Delete single entry
// #[post("/shortener/entries/{id}/delete")]

// Update bulk stats
// #[get("/shortener/stats")]
