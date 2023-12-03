// use actix_web::web;
// use actix_web_httpauth::middleware::HttpAuthentication;

pub(crate) mod entry;
pub(crate) mod queries;
// pub(self) mod methods;

// use crate::validate_admin;

// pub(crate) fn cfg(cfg: &mut web::ServiceConfig) {
//     cfg.service(
//         web::scope("/entries")
//             .service(
//                 web::resource("")
//                     .route(web::get().to(methods::list_entries))
//                     .route(
//                         web::post()
//                             .to(methods::create_entry)
//                             .wrap(HttpAuthentication::bearer(validate_admin)),
//                     ),
//             )
//             .service(
//                 web::resource("/{id}")
//                     .wrap(HttpAuthentication::bearer(validate_admin))
//                     .route(web::post().to(methods::update_entry)),
//             )
//             .service(
//                 web::resource("/{id}/delete")
//                     .wrap(HttpAuthentication::bearer(validate_admin))
//                     .route(web::post().to(methods::delete_entry)),
//             ),
//     )
//     .service(
//         web::scope("/stats").service(
//             web::resource("")
//                 .route(web::get().to(methods::get_stats))
//                 .route(
//                     web::post()
//                         .to(methods::update_stats)
//                         .wrap(HttpAuthentication::bearer(validate_admin)),
//                 ),
//         ),
//     );
// }
