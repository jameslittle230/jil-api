mod methods;
mod migration;
mod models;
mod queries;

use actix_cors::Cors;
use actix_web::web;
use actix_web_httpauth::middleware::HttpAuthentication;

use crate::admin_validator;

pub(crate) fn cfg(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("")
            .route(web::get().to(methods::get_entries_route))
            .route(web::post().to(methods::post_entry_route))
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allowed_methods(["GET", "POST", "OPTIONS"]),
            ),
    )
    .service(
        web::scope("/{id}")
            .service(web::resource("").route(web::get().to(methods::get_entry_route)))
            .service(
                web::resource("/delete")
                    .wrap(HttpAuthentication::bearer(admin_validator))
                    .route(web::post().to(methods::delete_entry_route)),
            ),
    );
}
