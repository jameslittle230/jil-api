use crate::guestbook_entry::GuestbookEntry;
use crate::slack::send_slack_message;
use crate::AppState;
use actix_web::error::{ErrorBadRequest, ErrorInternalServerError};
use actix_web::{error::Error as AWError, get, post, web, HttpRequest, HttpResponse};
use anyhow::{Error as AHError, Result};
use aws_sdk_dynamodb::model::AttributeValue;
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

async fn get_guestbook_entries(state: web::Data<AppState>) -> Result<Vec<GuestbookEntry>> {
    let scan_output = state
        .dynamodb
        .scan()
        .table_name("jil-guestbook")
        .send()
        .await?;

    let items = scan_output
        .items
        .ok_or_else(|| AHError::msg("Could not get items from dynamodb"))?;

    let mut entries: Vec<GuestbookEntry> = items
        .into_iter()
        .map(GuestbookEntry::try_from)
        .filter_map(|res| res.ok())
        .filter(|entry| entry.deleted_at.is_none())
        .collect::<Vec<GuestbookEntry>>();

    entries.sort_by_key(|entry| entry.created_at);

    Ok(entries)
}

#[derive(Debug, Serialize)]
struct GuestbookListEntry {
    pub id: Uuid,
    pub created_at: DateTime<chrono::Utc>,
    pub url: Option<String>,
    pub message: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
struct GuestbookListResponse {
    items: Vec<GuestbookListEntry>,
    count: usize,
}

impl From<&GuestbookEntry> for GuestbookListEntry {
    fn from(guestbook_entry: &GuestbookEntry) -> Self {
        Self {
            id: guestbook_entry.id,
            created_at: guestbook_entry.created_at,
            url: guestbook_entry.url.clone(),
            message: guestbook_entry.message.clone(),
            name: guestbook_entry.name.clone(),
        }
    }
}

#[get("/guestbook")]
pub async fn list_guestbook_entries(state: web::Data<AppState>) -> HttpResponse {
    let guestbook_entries = get_guestbook_entries(state).await;

    match guestbook_entries {
        Ok(items) => {
            let output_items: Vec<GuestbookListEntry> = items
                .iter()
                .filter(|entry| entry.deleted_at.is_none())
                .map(GuestbookListEntry::from)
                .collect();
            let count = (&output_items).len();
            HttpResponse::Ok().json(GuestbookListResponse {
                items: output_items,
                count,
            })
        }
        Err(err) => HttpResponse::InternalServerError().body(format!("{}", err)),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GuestbookPostData {
    pub name: String,
    pub message: String,
    pub email: Option<String>,
    pub url: Option<String>,
}

pub async fn put_guestbook_entry(state: web::Data<AppState>, entry: &GuestbookEntry) -> Result<()> {
    state
        .dynamodb
        .put_item()
        .table_name("jil-guestbook")
        .set_item(Some(entry.clone().into()))
        .send()
        .await?;

    Ok(())
}

#[post("/guestbook")]
pub async fn new_guestbook_entry(
    state: web::Data<AppState>,
    payload: web::Json<GuestbookPostData>,
    req: HttpRequest,
) -> Result<HttpResponse, AWError> {
    let guestbook_entry =
        GuestbookEntry::try_from(payload.into_inner()).map_err(ErrorBadRequest)?;

    put_guestbook_entry(state, &guestbook_entry)
        .await
        .map_err(ErrorInternalServerError)?;

    let _ = send_slack_message(&guestbook_entry.slack_api_request(req.peer_addr())).await;

    // let _ = trigger_netlify_rebuild().await;

    Ok(HttpResponse::Ok().body(serde_json::to_string(&guestbook_entry).unwrap()))
}

#[post("/guestbook/{id}/delete")]
pub async fn delete_guestbook_entry(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse, AWError> {
    let entry_id: String = req.match_info().query("id").parse().unwrap();

    let mut entry: GuestbookEntry = state
        .dynamodb
        .query()
        .table_name("jil-guestbook")
        .key_condition_expression("id = :value".to_string())
        .expression_attribute_values(":value".to_string(), AttributeValue::S(entry_id.clone()))
        .send()
        .await
        .map_err(ErrorBadRequest)?
        .items
        .unwrap()
        .pop()
        .ok_or_else(|| ErrorBadRequest(AHError::msg(format!("No entry found with ID {entry_id}"))))?
        .try_into()
        .map_err(ErrorInternalServerError)?;

    entry.deleted_at = Some(chrono::Utc::now());

    put_guestbook_entry(state, &entry)
        .await
        .map_err(ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().body(serde_json::to_string(&entry).unwrap()))
}
