use actix_web::{post, web, HttpResponse};
use anyhow::Context;
use serde_json::json;

use crate::error::ApiError;

#[derive(serde::Deserialize)]
pub(crate) struct LightOptions {
    color: Option<String>,
}

#[post("/home/light")]
pub(crate) async fn set_light(
    state: web::Data<crate::AppState>,
    data: web::Either<web::Json<LightOptions>, web::Form<LightOptions>>,
) -> Result<HttpResponse, ApiError> {
    let data = data.into_inner();
    let valid_colors = vec!["red", "green", "blue", "yellow", "purple"];
    if data.color.is_none() || !valid_colors.contains(&data.color.as_ref().unwrap().as_str()) {
        return Err(ApiError::bad_request("Invalid color"));
    }

    state
        .rate_limiter
        .check()
        .map_err(|_| ApiError::rate_limit_error())?;

    let shortcut_input = json!({ "color": data.color.unwrap_or("".to_string()) });

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

    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
        return Ok(HttpResponse::Ok().json(json));
    }

    Ok(HttpResponse::Ok().body(text))
}
