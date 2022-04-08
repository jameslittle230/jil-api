use actix_cors::Cors;
use actix_web::dev::ServiceRequest;
use actix_web::middleware::Logger;
use actix_web::web::Data;
use actix_web::{web, http, App, HttpResponse, HttpServer};

use actix_web_httpauth::extractors::bearer::{BearerAuth, Config};
use actix_web_httpauth::extractors::AuthenticationError;
use actix_web_httpauth::middleware::HttpAuthentication;

use env_logger::Env;

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::Client;

mod github_stork_stars;
mod guestbook;
mod guestbook_entry;
mod slack;

#[derive(Debug, Clone)]
pub struct AppState {
    dynamodb: Client,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = std::env::var("PORT").unwrap_or("8080".to_string());
    let addr = format!("0.0.0.0:{}", port);

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    dotenv::dotenv().ok();

    let region_provider = RegionProviderChain::default_provider().or_else("us-west-1");
    let config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&config);
    let app_state = AppState { dynamodb: client };

    HttpServer::new(move || {

        App::new()
            .app_data(actix_web::web::JsonConfig::default().limit(4096))
            .app_data(Data::new(app_state.clone()))
            .wrap(Logger::new(r#"peer="%a" time="%t" request="%r" response_code=%s response_size_bytes=%b response_time_ms="%D" user_agent="%{User-Agent}i" "#))

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

            .service(web::scope("/admin")
                .wrap(HttpAuthentication::bearer(admin_validator))
                .wrap(Cors::default().allow_any_origin().allowed_methods(["GET", "POST", "OPTIONS"]))
                .service(guestbook::delete_guestbook_entry)
            )

            .service(
                web::scope("")
                .wrap(Cors::default().allow_any_origin().allowed_methods(["GET", "POST", "OPTIONS"]).allowed_header(http::header::CONTENT_TYPE))
                .service(github_stork_stars::stork_stars)
                .service(slack::slack)
                .service(guestbook::new_guestbook_entry)
                .service(guestbook::list_guestbook_entries)
            )

            .default_service(web::route().to(|| HttpResponse::NotFound()))

    })
    .bind(addr)?
    .shutdown_timeout(if cfg!(dbg) { 0 } else { 600 })
    .run()
    .await
}

async fn admin_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, actix_web::Error> {
    let token = std::env::var("ADMIN_BEARER_TOKEN").unwrap();

    if credentials.token() == token {
        Ok(req)
    } else {
        let config = req
            .app_data::<Config>()
            .map(|data| data.clone())
            .unwrap_or_else(Default::default);

        Err(AuthenticationError::from(config).into())
    }
}
