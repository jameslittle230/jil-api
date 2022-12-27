use actix_web::{web, HttpRequest, HttpResponse};
use anyhow::Context;
use anyhow::Result;
use serde::Serializer;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::str::FromStr;
use strum_macros::{Display, EnumString};

use crate::error::ApiError;

#[derive(EnumString, Display, PartialEq, Eq, Deserialize, Clone, Debug)]
#[serde(try_from = "String")]
pub enum SlackChannel {
    #[strum(to_string = "C75C3AW66", serialize = "general")]
    General,

    #[strum(to_string = "CLVH6SLAZ", serialize = "jil-dot-im")]
    JilDotIm,

    #[strum(to_string = "CVBH1GHSM", serialize = "jil-guestbook")]
    JilGuestbook,

    #[strum(to_string = "C01HFPUJGHZ", serialize = "rrl-feedback")]
    RRLFeedback,

    #[strum(to_string = "C04GDTZ408Y", serialize = "stork-analytics")]
    StorkAnalytics,

    #[strum(to_string = "C04FM91GHA9", serialize = "stork-feedback")]
    StorkFeedback,
}

impl Serialize for SlackChannel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl TryFrom<String> for SlackChannel {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value).context("Couldn't parse the given Slack channel")
    }
}

impl Default for SlackChannel {
    fn default() -> Self {
        SlackChannel::General
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SlackApiRequest {
    pub text: String,

    #[serde(default)]
    pub channel: SlackChannel,

    #[serde(default)]
    pub blocks: Vec<Map<String, Value>>,
}

pub(crate) fn cfg(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("").route(web::post().to(slack)));
}

pub async fn send_slack_message(req: &SlackApiRequest) -> Result<reqwest::Response> {
    let client = reqwest::Client::new();

    let body = serde_json::to_string(&req)?;

    let slack_webhook_url = std::env::var("SLACK_WEBHOOK_URL")?;

    client
        .post(format!(
            "https://hooks.slack.com/services/{slack_webhook_url}",
        ))
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await
        .map_err(|err| err.into())
}

pub(crate) async fn slack(
    req: HttpRequest,
    payload: web::Json<SlackApiRequest>,
) -> Result<HttpResponse, ApiError> {
    let mut payload = payload.into_inner();

    if payload.blocks.is_empty() {
        payload.blocks.push(
            serde_json::from_str(&format!(r#"{{"type": "section", "text": {{ "type": "plain_text", "text": "{}", "emoji": true }} }}"#,
                payload.text
            ))
            .unwrap(),
        );
    }

    payload.blocks.push(
        serde_json::from_str(&format!(
            r#"{{
			"type": "context",
			"elements": [
                {{
                    "type": "plain_text",
                    "text": "From {}",
                    "emoji": true
                }}
            ]
		}}"#,
            req.peer_addr().unwrap()
        ))
        .unwrap(),
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

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_channel_tostring() {
        assert_eq!(SlackChannel::General.to_string(), "C75C3AW66");
    }

    #[test]
    fn test_channel_fromstring() {
        assert_eq!(
            SlackChannel::from_str("general").unwrap(),
            SlackChannel::General
        );
    }
}
