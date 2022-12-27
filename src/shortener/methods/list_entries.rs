use actix_web::{web, HttpResponse};
use serde::Serialize;

use super::super::queries::list_entries::list_shortlink_entries;

use crate::{error::ApiError, shortener::models::entry::Entry, AppState};

#[derive(Debug, Serialize)]
struct ListResponse {
    items: Vec<Entry>,
}

pub(crate) async fn exec(state: web::Data<AppState>) -> Result<HttpResponse, ApiError> {
    let entries = list_shortlink_entries(state).await;

    match entries {
        Ok((_, items)) => Ok(HttpResponse::Ok().json(ListResponse { items })),
        Err(err) => Err(ApiError::internal_server_error(err.to_string().as_str())),
    }
}
