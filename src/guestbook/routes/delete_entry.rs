use actix_web::{
    error::Error as AWError,
    error::{ErrorBadRequest, ErrorInternalServerError},
    post, web, HttpRequest, HttpResponse,
};

use anyhow::Result;

use crate::{
    guestbook::{get_single_entry, put_guestbook_entry, Entry},
    AppState,
};

#[post("/guestbook/{id}/delete")]
pub async fn exec(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse, AWError> {
    let entry_id: String = req.match_info().query("id").parse().unwrap();

    let mut entry: Entry = get_single_entry(&state, entry_id)
        .await
        .map_err(ErrorBadRequest)?;

    entry.deleted_at = Some(chrono::Utc::now());

    put_guestbook_entry(state, &entry)
        .await
        .map_err(ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().body(serde_json::to_string(&entry).unwrap()))
}
