use actix_web::{web, HttpRequest, HttpResponse};

use crate::{
    error::ApiError,
    guestbook::{models::entry::Entry, queries::get_single_entry::get_single_entry},
    AppState,
};

pub(crate) async fn exec(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, ApiError> {
    let entry_id: String = req.match_info().query("id").parse().unwrap();
    let entry: Entry = get_single_entry(&state, entry_id)
        .await
        .map_err(|err| ApiError::bad_request(err.to_string().as_str()))?;

    Ok(HttpResponse::Ok().body(serde_json::to_string(&entry).unwrap()))
}
