use std::{net::TcpListener, sync::Arc, time::Duration};

use actix_cors::Cors;
use actix_web::{
    dev::Server,
    middleware::{Logger, NormalizePath},
    web::{self, Data},
    App, HttpServer,
};
use actix_web_httpauth::middleware::HttpAuthentication;
use admin::validate_admin;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::Client;
use env_logger::Env;
use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use tokio::sync::Mutex;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable as ScalarServable};

mod admin;
mod api;
mod blog;
mod error;
mod guestbook;
mod ipinfo;
mod shortener;
mod slack;

use util::*;
use api::github::*;
use api::slack::*;
use api::guestbook::*;
use api::home::*;
use api::blog::*;
use api::shortener::*;

#[derive(Debug, Clone)]
pub struct AppState {
    dynamodb: Client,

    // a rate limiter with a fixed capacity of 10 requests per second.
    // this is a global rate limiter, so it will apply to all routes that
    // use it.
    rate_limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,

    ipinfo_cached_client: Arc<Mutex<ipinfo::CachedIpInfoClient>>,

    light_state: Arc<Mutex<String>>,

    openapi: String,
}

pub async fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    dotenv::dotenv().ok();

    let region_provider = RegionProviderChain::default_provider().or_else("us-west-1");
    let config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&config);

    

    #[derive(OpenApi)]
    #[openapi(
        info(
            title="api.jameslittle.me",
            description="Endpoint Reference Documentation",
            version="2024-08-13",
        ),
        servers(
            (url = "http://localhost:8125", description = "Development"),
            (url = "https://api.jameslittle.me", description = "Production"),
        ), 
        paths(
            healthcheck,

            post_slack,
            get_blog_deploy,
            get_github_stork_stars,
            
            get_guestbook,
            post_guestbook,
            get_guestbook_entry,
            delete_guestbook_entry,
            
            get_light,
            set_light,

            create_entry,
            list_entries,
            delete_entry,
            update_stats,
        ),
        tags(
            (name = "Guestbook", description = "The backend service storing guestbook entries for the guestbook on my personal website: https://jameslittle.me/guestbook"),
            (name = "Link Shortener", description = "The backend key-value store for the shortlink entries that back https://jil.im."),
            (name = "Home", description = "APIs that change the state of my home. Currently used to power a [light in my office](https://jameslittle.me/blog/2023/lights-api)."),
            (name = "Generic", description = "One-off API endpoints for specific use-cases"),
            (name = "Utility", description = "Utility endpoints for API management"),
        ),
        components(schemas()),
    )]
    struct ApiDoc;

    // Make instance variable of ApiDoc so all worker threads gets the same instance.
    let openapi = ApiDoc::openapi();

    let app_state = AppState {
        dynamodb: client,
        rate_limiter: Arc::new(RateLimiter::direct(
            Quota::with_period(Duration::from_secs(10)).unwrap(),
        )),
        ipinfo_cached_client: Arc::new(Mutex::new(ipinfo::CachedIpInfoClient::new(
            std::env::var("IPINFO_KEY").unwrap(),
        ))),
        light_state: Arc::new(Mutex::new("off".to_string())),
        openapi: openapi.clone().to_json().unwrap()
    };

    let server = HttpServer::new(move || {
        App::new()
            .app_data(actix_web::web::JsonConfig::default().limit(4096))
            .app_data(Data::new(app_state.clone()))
            .wrap(Logger::new(r#"peer="%a" time="%t" request="%r" response_code=%s response_size_bytes=%b response_time_ms="%D" user_agent="%{User-Agent}i" "#))
            .wrap(NormalizePath::trim())
            .wrap(Cors::permissive())
            .service(util::healthcheck)
            .service(util::root_redirect)
            .service(util::openapi_route)
            .service(Scalar::with_url("/docs", openapi.clone()))
            .service(api::github::get_github_stork_stars)
            .service(api::slack::post_slack)
            .service(api::guestbook::post_guestbook)
            .service(api::guestbook::get_guestbook)
            .service(api::guestbook::get_guestbook_entry)
            .service(api::shortener::list_entries)
            .service(api::home::set_light)
            .service(api::home::get_light)
            .service(web::scope("")
                .wrap(HttpAuthentication::bearer(validate_admin))
                .service(api::guestbook::delete_guestbook_entry)
                .service(api::blog::get_blog_deploy)
                .service(api::shortener::create_entry)
                .service(api::shortener::delete_entry)
                .service(api::shortener::update_stats)
            )
    })
    .listen(listener)?
    .shutdown_timeout(600)
    .run();

    Ok(server)
}

mod util {
    use actix_web::{get, web, HttpResponse, Responder};

    /// Healthcheck
    ///
    /// An endpoint that always returns `up`, used by external services to monitor
    /// the health of the service.
    #[utoipa::path(
        tag = "Utility",
        security(("api_key" = [])),
        responses(
            (status=200, description = "Success response", body=String, example=json!("up"))
        )
    )]
    #[get("/healthcheck")]
    async fn healthcheck() -> impl Responder {
        HttpResponse::Ok().content_type("text/plain").body("up")
    }

    /// Root Redirect
    ///
    /// Redirects 
    #[get("/")]
    async fn root_redirect() -> impl Responder {
        HttpResponse::TemporaryRedirect()
            .append_header(("Location", "https://jameslittle.me"))
            .body("")
    }

    #[get("/openapi.json")]
    async fn openapi_route(state: web::Data<crate::AppState>) -> impl Responder {
        HttpResponse::Ok().content_type("application/json").body(state.openapi.clone())
    }
}
