use actix_web::{
    error::{Error as AWError, ErrorBadRequest, ErrorInternalServerError},
    web, HttpRequest, HttpResponse,
};

use anyhow::Result;

use crate::{
    guestbook::{
        models::{Entry, GuestbookPostData},
        queries::put_entry::put_guestbook_entry,
    },
    slack::send_slack_message,
    AppState,
};

pub async fn exec(
    state: web::Data<AppState>,
    payload: web::Json<GuestbookPostData>,
    req: HttpRequest,
) -> Result<HttpResponse, AWError> {
    let guestbook_entry = Entry::try_from(payload.into_inner()).map_err(ErrorBadRequest)?;

    put_guestbook_entry(state, &guestbook_entry)
        .await
        .map_err(ErrorInternalServerError)?;

    let _ = send_slack_message(&guestbook_entry.slack_api_request(req.peer_addr())).await;

    // let _ = trigger_netlify_rebuild().await;

    Ok(HttpResponse::Ok().body(serde_json::to_string(&guestbook_entry).unwrap()))
}
