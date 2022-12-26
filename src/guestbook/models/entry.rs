use std::net;

use anyhow::{Error, Result};
use chrono::DateTime;
use dynomite::Item;
use serde::Serialize;
use uuid::Uuid;

use crate::slack::{SlackApiRequest, SlackChannel};

use super::super::methods::create_entry::PostData;

#[derive(Debug, Clone, Serialize, Item)]
pub struct Entry {
    #[dynomite(partition_key)]
    pub id: Uuid,

    pub created_at: DateTime<chrono::Utc>,

    #[dynomite(default)]
    #[serde(skip_serializing)]
    pub deleted_at: Option<DateTime<chrono::Utc>>,

    #[dynomite(default)]
    pub url: Option<String>,

    #[dynomite(default)]
    #[serde(skip_serializing)]
    pub email: Option<String>,

    pub message: String,

    pub name: String,
}

impl Entry {
    pub fn slack_api_request(&self, peer: Option<net::SocketAddr>) -> SlackApiRequest {
        let name = self.name.clone();
        let message = self.message.clone();
        let id = self.id.to_hyphenated().to_string();
        let url = self.url.clone().unwrap_or_else(|| "N/A".to_string());
        let email = self.email.clone().unwrap_or_else(|| "N/A".to_string());
        let peer = peer
            .map(|peer| peer.to_string())
            .unwrap_or_else(|| "N/A".to_string());

        SlackApiRequest {
            text: format!("Guestbook entry from {}: {}", self.name, self.message),
            channel: SlackChannel::JilGuestbook,
            blocks: serde_json::from_str(&format!(
                r#"
    [
		{{
			"type": "section",
			"text": {{
				"type": "mrkdwn",
				"text": "*{name}:*"
			}}
		}},
		{{
			"type": "section",
			"text": {{
				"type": "plain_text",
				"text": "{message}",
				"emoji": true
			}}
		}},
		{{
			"type": "context",
			"elements": [
				{{
					"type": "mrkdwn",
					"text": "From *{peer}*"
				}},
				{{
					"type": "mrkdwn",
					"text": "URL: *{url}*"
				}},
				{{
					"type": "mrkdwn",
					"text": "Email: *{email}*"
				}}
			]
		}},
		{{
			"type": "divider"
		}},
		{{
			"type": "actions",
			"elements": [
				{{
					"type": "button",
					"text": {{
						"type": "plain_text",
						"text": "View Online",
						"emoji": true
					}},
					"value": "click_me_123",
                    "url": "https://jameslittle.me/guestbook"
				}},
				{{
					"type": "button",
					"text": {{
						"type": "plain_text",
						"text": "Delete",
						"emoji": true
					}},
					"value": "click_me_123",
					"url": "https://api.jameslittle.me/{id}/delete"
				}}
			]
		}}
	]
            "#,
            ))
            .unwrap(),
        }
    }
}

impl TryFrom<PostData> for Entry {
    type Error = anyhow::Error;

    fn try_from(value: PostData) -> Result<Self, Self::Error> {
        if value.message.len() > 1200 {
            return Err(Error::msg("Message must be <= 1200 letters."));
        }

        if value.name.len() > 600 {
            return Err(Error::msg("Name must be <= 600 characters"));
        }

        Ok(Entry {
            id: Uuid::new_v4(),
            created_at: chrono::Utc::now(),
            deleted_at: None,
            url: value.url,
            email: value.email,
            message: value.message,
            name: value.name,
        })
    }
}
