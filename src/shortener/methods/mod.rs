use actix_web::{error::Error as AWError, web, HttpRequest, HttpResponse};

use crate::AppState;

pub(super) async fn list_entries(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse, AWError> {
    Ok(HttpResponse::Ok().body("yeet"))
}

pub(super) async fn create_entry(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse, AWError> {
    Ok(HttpResponse::NotImplemented().body("not implemented"))
}

pub(super) async fn update_entry(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse, AWError> {
    Ok(HttpResponse::NotImplemented().body("not implemented"))
}

pub(super) async fn delete_entry(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse, AWError> {
    Ok(HttpResponse::NotImplemented().body("not implemented"))
}

pub(super) async fn get_stats(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse, AWError> {
    Ok(HttpResponse::NotImplemented().body("not implemented"))
}

pub(super) async fn update_stats(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse, AWError> {
    Ok(HttpResponse::NotImplemented().body("not implemented"))
}
