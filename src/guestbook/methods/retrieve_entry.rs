use actix_web::{error::Error as AWError, error::ErrorBadRequest, web, HttpRequest, HttpResponse};

use anyhow::Result;

use crate::{
    guestbook::{models::entry::Entry, queries::get_single_entry::get_single_entry},
    AppState,
};

pub(crate) async fn exec(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AWError> {
    let entry_id: String = req.match_info().query("id").parse().unwrap();
    let entry: Entry = get_single_entry(&state, entry_id)
        .await
        .map_err(ErrorBadRequest)?;

    Ok(HttpResponse::Ok().body(serde_json::to_string(&entry).unwrap()))
}
