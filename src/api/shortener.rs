use actix_web::{get, post, web, Either, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::{
    error::ApiError,
    shortener::{
        entry::Entry,
        queries::{
            create_shortlink_entry, get_shortlink_entry, list_shortlink_entries,
            put_shortlink_entry,
        },
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
    create_shortlink_entry(&state.dynamodb, &shortener_entry).await?;
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

#[post("/shortener/entries/{id}/delete")]
pub(crate) async fn delete_entry(
    state: web::Data<crate::AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let entry_id = path.into_inner();
    let mut entry = get_shortlink_entry(&state.dynamodb, &entry_id).await?;
    entry.deleted_at = Some(chrono::Utc::now());
    put_shortlink_entry(&state.dynamodb, &entry).await?;
    Ok(HttpResponse::Ok().json(&entry))
}

// Update bulk stats
#[post("/shortener/stats")]
pub(crate) async fn update_stats(
    payload: web::Json<serde_json::Value>,
    state: web::Data<crate::AppState>,
) -> Result<HttpResponse, ApiError> {
    if payload.is_array() {
        for entry in payload.as_array().unwrap() {
            let entry = serde_json::from_value(entry.clone()).map_err(|_| {
                return ApiError::bad_request(
                    format!("Could not parse entry: {:?}", entry).as_str(),
                );
            })?;
            println!("Updating entry: {:?}", entry);
            put_shortlink_entry(&state.dynamodb, &entry).await?;
        }
        Ok(HttpResponse::Ok().body("OK"))
    } else {
        Ok(HttpResponse::BadRequest().body("Payload must be an array"))
    }
}
