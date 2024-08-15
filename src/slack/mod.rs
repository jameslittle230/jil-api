use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

pub(crate) mod channel;
use channel::SlackChannel;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, Default, ToSchema)]
pub(crate) struct SlackApiRequest {
    /// The fallback text displayed in the OS notification sent by the Slack application.
    #[schema(inline, example = "A regular bit of plaintext")]
    pub text: String,

    /// The name of the Slack channel to send the message to.
    #[schema(inline, additional_properties)]
    pub channel: SlackChannel,

    /// The [slack blocks](https://app.slack.com/block-kit-builder/T74S9SE9F)
    /// that represent a rich message to be sent to the specified channel.
    #[schema(example = json!([{
        "type": "section",
        "text": {
          "text": "A message *with some bold text* and _some italicized text_.",
          "type": "mrkdwn"
        },
        "fields": [
          {
            "type": "mrkdwn",
            "text": "High"
          },
          {
            "type": "plain_text",
            "emoji": true,
            "text": "Silly"
          }
        ]
      }]))]
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
