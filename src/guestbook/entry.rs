use std::{collections::HashMap, default, net};

use anyhow::{Error, Result};
use chrono::DateTime;
use dynomite::Item;
use serde::{ser::SerializeStruct, Serialize};
use uuid::Uuid;

use crate::{
    api::guestbook::GuestbookForm,
    slack::{channel::SlackChannel, SlackApiRequest},
};

struct DummyStruct;

impl DummyStruct {
    fn always_true<T, U>(_: &HashMap<T, U>) -> bool {
        true
    }
}

#[derive(Debug, Clone, Item)]
pub struct Entry {
    #[dynomite(partition_key)]
    pub id: Uuid,

    pub created_at: DateTime<chrono::Utc>,

    #[dynomite(default)]
    pub deleted_at: Option<DateTime<chrono::Utc>>,

    #[dynomite(default)]
    pub url: Option<String>,

    #[dynomite(default)]
    pub email: Option<String>,

    pub message: String,

    pub name: String,

    #[dynomite(default)]
    pub qa: bool,

    // This should be transparent to the database and to serde
    #[dynomite(default)]
    #[dynomite(skip_serializing_if = "DummyStruct::always_true")]
    pub __ser_options: HashMap<String, bool>,
}

impl Entry {
    pub(crate) fn push_ser_option(&mut self, key: &str) {
        self.__ser_options.insert(key.to_string(), true);
    }
}

impl Serialize for Entry {
    fn serialize<S>(&self, serializer: S) -> std::prelude::v1::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Entry", 7)?;
        state.serialize_field("id", &self.id.to_hyphenated().to_string())?;
        state.serialize_field("created_at", &self.created_at.to_rfc3339())?;
        state.serialize_field("url", &self.url)?;
        state.serialize_field("message", &self.message)?;
        state.serialize_field("name", &self.name)?;

        if self.__ser_options.contains_key("serialize_deleted_at") {
            state.serialize_field("deleted_at", &self.deleted_at.map(|dt| dt.to_rfc3339()))?;
        }

        if self.__ser_options.contains_key("serialize_qa") {
            state.serialize_field("qa", &self.qa)?;
        }

        state.end()
    }
}

impl Entry {
    pub(crate) fn slack_api_request(&self, peer: Option<net::SocketAddr>) -> SlackApiRequest {
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

impl TryFrom<GuestbookForm> for Entry {
    type Error = anyhow::Error;

    fn try_from(value: GuestbookForm) -> Result<Self, Self::Error> {
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
            qa: value.qa,
            ..Default::default()
        })
    }
}

impl Default for Entry {
    fn default() -> Self {
        Entry {
            id: Uuid::new_v4(),
            created_at: chrono::Utc::now(),
            deleted_at: default::Default::default(),
            url: default::Default::default(),
            email: default::Default::default(),
            message: default::Default::default(),
            name: default::Default::default(),
            qa: default::Default::default(),
            __ser_options: default::Default::default(),
        }
    }
}
