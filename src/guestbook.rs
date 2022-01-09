use crate::guestbook_entry::GuestbookEntry;
use crate::slack::send_slack_message;
use crate::AppState;
use actix_web::error::{ErrorBadRequest, ErrorInternalServerError};
use actix_web::{error::Error as AWError, get, post, web, HttpRequest, HttpResponse};
use anyhow::{Error as AHError, Result};
use aws_sdk_dynamodb::model::{AttributeValue, Select};
use serde::{Deserialize, Serialize};
use serde_json::json;

async fn get_guestbook_entries(state: web::Data<AppState>) -> Result<Vec<GuestbookEntry>> {
    let scan_output = state
        .dynamodb
        .scan()
        .table_name("jil-guestbook-9552749")
        .send()
        .await?;

    let items = scan_output
        .items
        .ok_or(AHError::msg("Could not get items from dynamodb"))?;

    let guestbook_entries: Vec<Result<GuestbookEntry>> =
        items.into_iter().map(GuestbookEntry::try_from).collect();

    let entries: Vec<GuestbookEntry> = guestbook_entries
        .into_iter()
        .filter_map(|res| res.ok())
        .filter(|entry| entry.deleted_at.is_none())
        .collect();

    // let errs: Vec<String> = guestbook_entries
    //     .into_iter()
    //     .filter_map(|res| res.err())
    //     .collect();

    Ok(entries)
}

#[derive(Debug, Serialize)]
struct GuestbookListResponse {
    items: Vec<GuestbookEntry>,
}

#[get("/guestbook")]
pub async fn list_guestbook_entries(state: web::Data<AppState>) -> HttpResponse {
    let guestbook_entries = get_guestbook_entries(state).await;

    match guestbook_entries {
        Ok(entries) => {
            let sorted_entries = {
                let mut e = entries;
                e.sort_by_key(|entry| entry.created_at);
                e
            };

            HttpResponse::Ok().json(GuestbookListResponse {
                items: sorted_entries,
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
    let mut request = state
        .dynamodb
        .put_item()
        .table_name("jil-guestbook-9552749");

    request = request.clone().set_item(Some(entry.items()));

    request.send().await?;

    Ok(())
}

#[post("/guestbook")]
pub async fn new_guestbook_entry(
    state: web::Data<AppState>,
    payload: web::Json<GuestbookPostData>,
) -> Result<HttpResponse, AWError> {
    let guestbook_entry =
        GuestbookEntry::try_from(payload.into_inner()).map_err(|err| ErrorBadRequest(err))?;

    put_guestbook_entry(state, &guestbook_entry)
        .await
        .map_err(|err| ErrorInternalServerError(err))?;

    let _ = send_slack_message(&guestbook_entry.slack_api_request()).await;

    // let _ = trigger_netlify_rebuild().await;

    Ok(HttpResponse::Ok().body(
        serde_json::to_string(&json!({ "id": &guestbook_entry.id.to_hyphenated().to_string() }))
            .unwrap(),
    ))
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
        .table_name("jil-guestbook-9552749")
        .key_condition_expression("#key = :value".to_string())
        .expression_attribute_names("#key".to_string(), "id".to_string())
        .expression_attribute_values(":value".to_string(), AttributeValue::S(entry_id))
        .select(Select::AllAttributes)
        .send()
        .await
        .map_err(|err| ErrorBadRequest(err))?
        .items
        .unwrap()
        .pop()
        .ok_or(ErrorBadRequest(AHError::msg(
            "No entry found with given ID",
        )))?
        .try_into()
        .map_err(|err| ErrorInternalServerError(err))?;

    entry.deleted_at = Some(chrono::Utc::now());

    put_guestbook_entry(state, &entry)
        .await
        .map_err(|err| ErrorInternalServerError(err))?;

    Ok(HttpResponse::Ok().body("Updated"))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GuestbookReactionPostData {
    emoji: String,
}
