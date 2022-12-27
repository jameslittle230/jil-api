use actix_web::{web, HttpRequest, HttpResponse};

use serde::{Deserialize, Serialize};

use crate::{
    error::ApiError,
    guestbook::{models::entry::Entry, queries::put_entry::put_guestbook_entry},
    slack::send_slack_message,
    AppState,
};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct PostData {
    pub name: String,
    pub message: String,
    pub email: Option<String>,
    pub url: Option<String>,
}

pub(crate) async fn exec(
    state: web::Data<AppState>,
    payload: web::Json<PostData>,
    req: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    let guestbook_entry = Entry::try_from(payload.into_inner())
        .map_err(|err| ApiError::bad_request(err.to_string().as_str()))?;

    put_guestbook_entry(state, &guestbook_entry)
        .await
        .map_err(|err| ApiError::internal_server_error(err.to_string().as_str()))?;

    let _ = send_slack_message(&guestbook_entry.slack_api_request(req.peer_addr())).await;

    Ok(HttpResponse::Ok().body(serde_json::to_string(&guestbook_entry).unwrap()))
}
