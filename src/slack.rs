use actix_web::{post, web, HttpRequest, HttpResponse};
use anyhow::Context;
use anyhow::Result;
use reqwest::StatusCode;
use serde::Serializer;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::str::FromStr;
use strum_macros::{Display, EnumString};

#[derive(EnumString, Display, PartialEq, Deserialize, Clone, Debug)]
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

    #[strum(to_string = "C01075PV3QU", serialize = "stork-ci")]
    StorkCI,

    #[strum(to_string = "C02HKGWCYL9", serialize = "api-requests-without-channel")]
    ApiRequestsWithoutChannel,
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

// impl Into<String> for SlackChannel {
//     fn into(self) -> String {
//         self.to_string()
//     }
// }

impl Default for SlackChannel {
    fn default() -> Self {
        SlackChannel::ApiRequestsWithoutChannel
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

pub async fn send_slack_message(req: &SlackApiRequest) -> Result<(StatusCode, String)> {
    let client = reqwest::Client::new();

    let body = serde_json::to_string(&req)?;

    let slack_webhook_url = std::env::var("SLACK_WEBHOOK_URL")?;

    let req = client
        .post(format!(
            "https://hooks.slack.com/services/{}",
            slack_webhook_url
        ))
        .header("Content-Type", "application/json")
        .body(body);

    let result = req.send().await?;
    let status = result.status();
    let body = &result.text().await?;

    Ok((status, body.to_owned()))
}

#[post("/slack")]
pub async fn slack(req: HttpRequest, payload: web::Json<SlackApiRequest>) -> HttpResponse {
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
                    "text": "From: {}",
                    "emoji": true
                }}
            ]
		}}"#,
            req.peer_addr().unwrap()
        ))
        .unwrap(),
    );

    match send_slack_message(&payload).await {
        Ok((status, body)) => match status.as_u16() {
            200 => HttpResponse::Ok().body(body),
            _ => {
                println!("{}", serde_json::to_string_pretty(&payload).unwrap());
                HttpResponse::BadRequest().body(format!("{} - {}", status, body))
            }
        },
        Err(err) => HttpResponse::InternalServerError().body(format!("{}", err)),
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
