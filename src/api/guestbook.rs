use actix_web::{get, post, web, Either, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::{
    blog::deploy_blog,
    error::ApiError,
    guestbook::{
        entry::Entry,
        queries::{get_single_entry, get_undeleted_entries, put_guestbook_entry},
    },
    slack::send_slack_message,
};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct GuestbookForm {
    pub name: String,
    pub message: String,
    pub email: Option<String>,
    pub url: Option<String>,

    #[serde(default)]
    pub qa: bool,
}

#[post("/guestbook")]
pub(crate) async fn post_guestbook(
    req: HttpRequest,
    data: Either<web::Form<GuestbookForm>, web::Json<GuestbookForm>>,
    state: web::Data<crate::AppState>,
) -> Result<HttpResponse, ApiError> {
    let guestbook_form = data.into_inner();
    let mut guestbook_entry = Entry::try_from(guestbook_form)?;

    put_guestbook_entry(&state.dynamodb, &guestbook_entry).await?;

    let _ = send_slack_message(&guestbook_entry.slack_api_request(req.peer_addr())).await;
    let _ = deploy_blog().await;

    if guestbook_entry.qa {
        guestbook_entry.push_ser_option("serialize_qa");
    }
    Ok(HttpResponse::Ok().json(&guestbook_entry))
}

#[derive(Debug, Deserialize)]
pub(crate) struct GetGuestbookQueryParameters {
    pub after: Option<uuid::Uuid>,

    #[serde(default)]
    pub qa: bool,
}

#[derive(Debug, Serialize)]
struct GetGuestbookResponse {
    items: Vec<Entry>,
    count: usize,
    total_count: usize,
}

#[get("/guestbook")]
pub(crate) async fn get_guestbook(
    query: web::Query<GetGuestbookQueryParameters>,
    state: web::Data<crate::AppState>,
) -> Result<HttpResponse, ApiError> {
    let (total_count, guestbook_entries) =
        get_undeleted_entries(&state.dynamodb, query.after, query.qa).await?;

    let count = &guestbook_entries.len();
    Ok(HttpResponse::Ok().json(GetGuestbookResponse {
        items: guestbook_entries,
        count: *count,
        total_count,
    }))
}

#[get("/guestbook/{id}")]
pub(crate) async fn get_guestbook_entry(
    path: web::Path<uuid::Uuid>,
    state: web::Data<crate::AppState>,
) -> Result<HttpResponse, ApiError> {
    let entry_id = path.into_inner();
    let mut entry = get_single_entry(&state.dynamodb, &entry_id).await?;
    entry.push_ser_option("serialize_deleted_at");
    entry.push_ser_option("serialize_qa");
    Ok(HttpResponse::Ok().json(&entry))
}

#[post("guestbook/{id}/delete")]
pub(crate) async fn delete_guestbook_entry(
    path: web::Path<uuid::Uuid>,
    state: web::Data<crate::AppState>,
) -> Result<HttpResponse, ApiError> {
    let entry_id = path.into_inner();
    let mut entry = get_single_entry(&state.dynamodb, &entry_id).await?;
    entry.deleted_at = Some(chrono::Utc::now());
    put_guestbook_entry(&state.dynamodb, &entry).await?;
    entry.push_ser_option("serialize_deleted_at");
    entry.push_ser_option("serialize_qa");

    Ok(HttpResponse::Ok().json(&entry))
}
