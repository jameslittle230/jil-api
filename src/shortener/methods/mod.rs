use actix_web::{error::Error as AWError, web, HttpRequest, HttpResponse};

use crate::AppState;

mod create_entry;
mod list_entries;

pub(super) use create_entry::exec as create_entry;
pub(super) use list_entries::exec as list_entries;

#[allow(unused)]
pub(super) async fn update_entry(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse, AWError> {
    Ok(HttpResponse::NotImplemented().body("not implemented"))
}

#[allow(unused)]
pub(super) async fn delete_entry(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse, AWError> {
    Ok(HttpResponse::NotImplemented().body("not implemented"))
}

#[allow(unused)]
pub(super) async fn get_stats(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse, AWError> {
    Ok(HttpResponse::NotImplemented().body("not implemented"))
}

#[allow(unused)]
pub(super) async fn update_stats(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse, AWError> {
    Ok(HttpResponse::NotImplemented().body("not implemented"))
}
