use actix_web::{get, post, web, HttpRequest, HttpResponse};
use anyhow::Context;
use serde_json::json;

use crate::{
    error::ApiError,
    slack::{channel::SlackChannel, send_slack},
};

#[derive(serde::Deserialize)]
pub(crate) struct LightOptions {
    color: Option<String>,
}

async fn get_ip_attribution_string_from_request(
    req: &HttpRequest,
    state: &crate::AppState,
) -> Option<String> {
    let ip = req
        .connection_info()
        .realip_remote_addr()
        .map(|ip| ip.to_string())?;

    let ip_info = state
        .ipinfo_cached_client
        .lock()
        .await
        .get_ip_info(&ip)
        .await
        .ok();

    Some(match ip_info {
        None => format!("{} (No info)", ip),
        Some(ip_info) => format!("{} ({})", ip, ip_info.ip_info.loc_to_string()),
    })
}

fn valid_colors() -> Vec<&'static str> {
    vec!["red", "green", "blue", "yellow", "purple", "white", "off"]
}

#[get("/home/light")]
pub(crate) async fn get_light(
    state: web::Data<crate::AppState>,
) -> actix_web::Result<HttpResponse> {
    let light_state = state.light_state.lock().await.clone();
    Ok(HttpResponse::Ok().json(json!({ "state": light_state, "values": valid_colors() })))
}

#[post("/home/light")]
pub(crate) async fn set_light(
    state: web::Data<crate::AppState>,
    data: web::Either<web::Json<LightOptions>, web::Form<LightOptions>>,
    req: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    let data = data.into_inner();

    let ip_attribution_string = get_ip_attribution_string_from_request(&req, &state).await;

    let _ = send_slack(
        &format!(
            "{}: {}",
            data.color.clone().unwrap_or("no color".to_string()),
            ip_attribution_string.unwrap_or("no ip".to_string())
        ),
        SlackChannel::Lights,
    )
    .await;
    if data.color.is_none() || !(valid_colors().contains(&data.color.as_ref().unwrap().as_str())) {
        return Err(ApiError::bad_request("Invalid color"));
    }

    state
        .rate_limiter
        .check()
        .map_err(|_| ApiError::rate_limit_error())?;

    let color = data.color.clone().unwrap_or("".to_string());
    let shortcut_input = json!({ "color": color });

    let response = reqwest::Client::new()
        .post(format!(
            "https://api.pushcut.io/{}/execute",
            std::env::var("PUSHCUT_KEY").unwrap()
        ))
        .query(&[
            ("shortcut", "Set Light Color"),
            ("input", &shortcut_input.to_string()),
        ])
        .send()
        .await
        .context("Failed to send request to Pushcut")?;

    response.error_for_status_ref().map_err(|err| {
        println!("Error: {:?}", &err);
        ApiError::internal_server_error(&format!(
            "Pushcut responded with an error: {}",
            &response.status()
        ))
    })?;

    let text = response
        .text()
        .await
        .map_err(|err| ApiError::internal_server_error(&err.to_string()))?;

    if serde_json::from_str::<serde_json::Value>(&text).is_ok() {
        let mut light_state = state.light_state.lock().await;
        *light_state = color.clone();
        return Ok(HttpResponse::Ok().json(json!({ "state": color, "values": valid_colors() })));
    }

    Ok(HttpResponse::Ok().body(text))
}
