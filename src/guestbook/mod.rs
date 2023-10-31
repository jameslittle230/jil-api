mod methods;
mod migration;
mod models;
mod queries;

use actix_cors::Cors;
use actix_web::web;
use actix_web_httpauth::middleware::HttpAuthentication;

use crate::validate_admin;

pub(crate) fn cfg(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("")
            .route(web::get().to(methods::list_entries))
            .route(web::post().to(methods::create_entry))
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_header()
                    .allowed_methods(["GET", "POST", "OPTIONS"]),
            ),
    )
    .service(
        web::scope("/{id}")
            .service(web::resource("").route(web::get().to(methods::retrieve_entry)))
            .service(
                web::resource("/delete")
                    .wrap(HttpAuthentication::bearer(validate_admin))
                    .route(web::post().to(methods::delete_entry)),
            ),
    );
}
