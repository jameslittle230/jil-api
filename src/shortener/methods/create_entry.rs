use actix_web::{error::ErrorInternalServerError, web, Error, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::AppState;

use crate::shortener::models::entry::Entry;
use crate::shortener::queries::put_entry::put_shortlink_entry;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct PostData {
    pub shortname: String,
    pub longurl: String,
}

pub(crate) async fn exec(
    state: web::Data<AppState>,
    payload: web::Json<PostData>,
) -> Result<HttpResponse, Error> {
    let entry = Entry {
        shortname: payload.shortname.clone(),
        created_at: chrono::Utc::now(),
        deleted_at: None,
        longurl: payload.longurl.clone(),
    };

    put_shortlink_entry(state, &entry)
        .await
        .map_err(ErrorInternalServerError)?;

    // let _ = send_slack_message(&guestbook_entry.slack_api_request(req.peer_addr())).await;

    Ok(HttpResponse::Ok().body(serde_json::to_string(&entry).unwrap()))
}
