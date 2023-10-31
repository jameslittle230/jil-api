use std::net::TcpListener;

use actix_web::{
    dev::Server,
    middleware::{Logger, NormalizePath},
    web::{self, Data},
    App, HttpResponse, HttpServer,
};
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::Client;
use env_logger::Env;

mod admin;
use admin::validate_admin;

mod error;
mod github;
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

            .service(web::scope("/guestbook").configure(guestbook::cfg))
            .service(web::scope("/github").configure(github::cfg))
            .service(web::scope("/slack").configure(slack::cfg))
            .service(web::scope("/shortener").configure(shortener::cfg))

            .default_service(web::route().to(HttpResponse::NotFound))

    })
    .listen(listener)?
    .shutdown_timeout(if cfg!(dbg) { 0 } else { 600 })
    .run();

    Ok(server)
}
