use actix_web::{get, post, web, Either, HttpResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    error::ApiError,
    shortener::{
        entry::Entry,
        queries::{get_shortlink_entry, list_shortlink_entries, put_shortlink_entry},
    },
    slack::{channel::SlackChannel, send_slack_message, SlackApiRequest},
};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub(crate) struct CreateEntryForm {
    pub shortname: String,
    pub longurl: String,
}

/// Create a Shortener Entry
///
/// Creates an entry to be added to my personal link shortener.
///
/// This API does not resolve the shortlinks to long URLs; it only holds a
/// key-value store for the mapping between the two.
///
/// This endpoint must be called with a bearer token header:
///
/// ```
/// Authorization: Bearer admin
/// ```
#[utoipa::path(
    request_body = inline(CreateEntryForm),
    responses(
        (status=200, description = "Success response", body=inline(Entry))
    ),
    tag = "Link Shortener"
)]
#[post("/shortener/entries")]
pub(crate) async fn create_entry(
    state: web::Data<crate::AppState>,
    payload: Either<web::Json<CreateEntryForm>, web::Form<CreateEntryForm>>,
) -> Result<HttpResponse, ApiError> {
    let payload = payload.into_inner();
    let get_result = get_shortlink_entry(&state.dynamodb, &payload.shortname).await;
    if let Ok(entry) = get_result {
        if entry.deleted_at.is_none() {
            return Err(ApiError::bad_request("Shortname already exists"));
        }
    }
    let shortener_entry = Entry::try_from(payload)?;
    put_shortlink_entry(&state.dynamodb, &shortener_entry).await?;
    Ok(HttpResponse::Ok().json(&shortener_entry))
}

/// List Shortener Entries
///
/// Lists all key/value pairs that are saved as link shortener entries.
#[utoipa::path(
    responses(
        (status=200, description = "Success response")
    ),
    tag = "Link Shortener"
)]
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

/// Delete a Shortener Entry
///
/// This endpoint must be called with a bearer token header:
///
/// ```
/// Authorization: Bearer admin
/// ```
#[utoipa::path(
    responses(
        (status=200, description = "Success response", body=inline(Entry))
    ),
    tag = "Link Shortener"
)]
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

/// Update Statistics
///
/// This endpoint must be called with a bearer token header:
///
/// ```
/// Authorization: Bearer admin
/// ```
#[utoipa::path(
    responses(
        (status=200, description = "Success response")
    ),
    tag = "Link Shortener"
)]
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
        let _ = send_slack_message(&SlackApiRequest {
            channel: SlackChannel::General,
            blocks: vec![],
            text: format!(
                "Updated {} shortlink entries",
                payload.as_array().unwrap().len()
            ),
        })
        .await;
        Ok(HttpResponse::Ok().body("OK"))
    } else {
        Ok(HttpResponse::BadRequest().body("Payload must be an array"))
    }
}
