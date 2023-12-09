use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

pub(crate) mod channel;
use channel::SlackChannel;

#[derive(Serialize, Deserialize, Debug, Default)]
pub(crate) struct SlackApiRequest {
    pub text: String,

    #[serde(default)]
    pub channel: SlackChannel,

    #[serde(default)]
    pub blocks: Vec<Map<String, Value>>,
}

pub(crate) async fn send_slack_message(req: &SlackApiRequest) -> Result<reqwest::Response> {
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

pub(crate) async fn send_slack(text: &str, channel: SlackChannel) -> Result<reqwest::Response> {
    let req = SlackApiRequest {
        text: text.to_string(),
        channel,
        ..Default::default()
    };

    send_slack_message(&req).await
}
