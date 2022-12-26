use actix_web::{web, HttpResponse};
use serde::Serialize;

use super::super::queries::list_entries::list_shortlink_entries;

use crate::{shortener::models::entry::Entry, AppState};

#[derive(Debug, Serialize)]
struct ListResponse {
    items: Vec<Entry>,
}

pub(crate) async fn exec(state: web::Data<AppState>) -> HttpResponse {
    let entries = list_shortlink_entries(state).await;

    match entries {
        Ok((_, items)) => HttpResponse::Ok().json(ListResponse { items }),
        Err(err) => HttpResponse::InternalServerError().body(format!("{}", err)),
    }
}
