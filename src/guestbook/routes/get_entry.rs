use actix_web::{
    error::Error as AWError, error::ErrorBadRequest, get, web, HttpRequest, HttpResponse,
};

use anyhow::Result;

use displayable_entry::DisplayableEntry;

use crate::{
    guestbook::{get_single_entry, models::displayable_entry},
    AppState,
};

#[get("/guestbook/{id}")]
pub(crate) async fn exec(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AWError> {
    let entry_id: String = req.match_info().query("id").parse().unwrap();
    let displayable_entry: DisplayableEntry = get_single_entry(&state, entry_id)
        .await
        .map_err(ErrorBadRequest)?
        .into();

    Ok(HttpResponse::Ok().body(serde_json::to_string(&displayable_entry).unwrap()))
}
