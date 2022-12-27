use actix_web::{web, HttpRequest, HttpResponse};

use crate::{
    error::ApiError,
    guestbook::{
        models::entry::Entry,
        queries::{get_single_entry::get_single_entry, put_entry::put_guestbook_entry},
    },
    AppState,
};

pub(crate) async fn exec(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    let entry_id: String = req.match_info().query("id").parse().unwrap();

    let mut entry: Entry = get_single_entry(&state, entry_id)
        .await
        .map_err(|err| ApiError::internal_server_error(err.to_string().as_str()))?;

    entry.deleted_at = Some(chrono::Utc::now());

    put_guestbook_entry(state, &entry)
        .await
        .map_err(|err| ApiError::internal_server_error(err.to_string().as_str()))?;

    Ok(HttpResponse::Ok().body(serde_json::to_string(&entry).unwrap()))
}
