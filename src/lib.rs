use std::net::TcpListener;

use actix_cors::Cors;
use actix_web::{
    dev::Server,
    middleware::{Logger, NormalizePath},
    web::{self, Data},
    App, HttpResponse, HttpServer,
};
use actix_web_httpauth::middleware::HttpAuthentication;
use admin::validate_admin;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::Client;
use env_logger::Env;

mod admin;
mod api;
mod blog;
mod error;
mod guestbook;
mod shortener;
mod slack;

#[derive(Debug, Clone)]
pub struct AppState {
    dynamodb: Client,
}

pub async fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    dotenv::dotenv().ok();

    let region_provider = RegionProviderChain::default_provider().or_else("us-west-1");
    let config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&config);
    let app_state = AppState { dynamodb: client };

    let server = HttpServer::new(move || {

        App::new()
            .app_data(actix_web::web::JsonConfig::default().limit(4096))
            .app_data(Data::new(app_state.clone()))
            .wrap(Logger::new(r#"peer="%a" time="%t" request="%r" response_code=%s response_size_bytes=%b response_time_ms="%D" user_agent="%{User-Agent}i" "#))
            .wrap(NormalizePath::trim())
            .wrap(Cors::permissive())

            .route(
                "/healthcheck",
                web::get().to(|| async { HttpResponse::Ok().body("up") })
            )

            .route(
                "/",
                web::get().to(|| async {
                    HttpResponse::TemporaryRedirect()
                    .append_header(("Location", "https://jameslittle.me"))
                    .body("")
                })
            )

            .service(api::github::get_github_stork_stars)
            .service(api::slack::post_slack)
            .service(api::guestbook::post_guestbook)
            .service(api::guestbook::get_guestbook)
            .service(api::guestbook::get_guestbook_entry)            
            .service(api::shortener::list_entries)
            .service(web::scope("")
                .wrap(HttpAuthentication::bearer(validate_admin))
                .service(api::guestbook::delete_guestbook_entry)
                .service(api::blog::get_blog_deploy)
                .service(api::shortener::create_entry)
            )

            .default_service(web::route().to(HttpResponse::NotFound))

    })
    .listen(listener)?
    .shutdown_timeout(if cfg!(dbg) { 0 } else { 600 })
    .run();

    Ok(server)
}
