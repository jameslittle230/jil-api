use actix_web::{
    error::{Error as AWError, ErrorBadRequest, ErrorInternalServerError},
    web, HttpRequest, HttpResponse,
};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
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
) -> Result<HttpResponse, AWError> {
    let guestbook_entry = Entry::try_from(payload.into_inner()).map_err(ErrorBadRequest)?;

    put_guestbook_entry(state, &guestbook_entry)
        .await
        .map_err(ErrorInternalServerError)?;

    let _ = send_slack_message(&guestbook_entry.slack_api_request(req.peer_addr())).await;

    Ok(HttpResponse::Ok().body(serde_json::to_string(&guestbook_entry).unwrap()))
}
