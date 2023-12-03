use actix_web::{post, web, HttpRequest, HttpResponse};
use serde_json::json;

use crate::{
    error::ApiError,
    slack::{send_slack_message, SlackApiRequest},
};

#[post("/slack")]
pub(crate) async fn post_slack(
    req: HttpRequest,
    payload: web::Json<SlackApiRequest>,
) -> Result<HttpResponse, ApiError> {
    let mut payload = payload.into_inner();

    // If there are no blocks, add a default block with the given text.
    if payload.blocks.is_empty() {
        payload.blocks.push(
            json!({
                "type": "section",
                "text": {
                    "type": "plain_text",
                    "text": payload.text,
                    "emoji": true
                }
            })
            .as_object()
            .unwrap()
            .clone(),
        );
    }

    // Add a context block with the IP address of the request.
    payload.blocks.push(
        json!({
            "type": "context",
            "elements": [
                {
                    "type": "plain_text",
                    "text": format!("From {}", req.connection_info().realip_remote_addr().unwrap_or("unknown")),
                    "emoji": true
                }
            ]
        })
        .as_object()
        .unwrap()
        .clone()
    );

    match send_slack_message(&payload).await {
        Ok(response) => match response.error_for_status() {
            Ok(response) => {
                let body = response.text().await.unwrap_or_default();
                Ok(HttpResponse::Ok().body(body))
            }
            Err(err) => Err(ApiError::bad_request(err.to_string().as_str())),
        },
        Err(err) => Err(ApiError::internal_server_error(err.to_string().as_str())),
    }
}
