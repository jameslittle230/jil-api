use anyhow::{Error, Result};
use aws_sdk_dynamodb::model::AttributeValue;
use chrono::{DateTime, NaiveDateTime};
use serde::Serialize;
use std::{collections::HashMap, str::FromStr};
use uuid::Uuid;

use crate::{
    guestbook::GuestbookPostData,
    slack::{SlackApiRequest, SlackChannel},
};

#[derive(Debug, Clone, Serialize)]
pub struct GuestbookEntry {
    pub id: Uuid,
    pub created_at: DateTime<chrono::Utc>,
    pub deleted_at: Option<DateTime<chrono::Utc>>,

    url: Option<String>,
    email: Option<String>,
    message: String,
    name: String,
}

impl GuestbookEntry {
    pub fn items(&self) -> HashMap<String, AttributeValue> {
        let mut h = HashMap::with_capacity(6);

        h.insert("message".into(), AttributeValue::S(self.message.clone()));
        h.insert("name".into(), AttributeValue::S(self.name.clone()));

        h.insert(
            "id".into(),
            AttributeValue::S(self.id.to_hyphenated().to_string()),
        );

        h.insert(
            "created_at".into(),
            AttributeValue::S(self.created_at.timestamp_millis().to_string()),
        );

        if let Some(deleted_at) = self.deleted_at {
            h.insert(
                "deleted_at".into(),
                AttributeValue::S(deleted_at.timestamp_millis().to_string()),
            );
        }

        if let Some(email) = self.email.clone() {
            h.insert("email".into(), AttributeValue::S(email));
        }

        if let Some(url) = self.url.clone() {
            h.insert("url".into(), AttributeValue::S(url));
        }

        log::debug!("{:?}", h);

        h
    }

    pub fn slack_api_request(&self) -> SlackApiRequest {
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
				"text": "*{}:*"
			}}
		}},
		{{
			"type": "section",
			"text": {{
				"type": "plain_text",
				"text": "{}",
				"emoji": true
			}}
		}},
		{{
			"type": "context",
			"elements": [
				{{
					"type": "mrkdwn",
					"text": "From *TBD*"
				}},
				{{
					"type": "mrkdwn",
					"text": "URL: *TBD*"
				}},
				{{
					"type": "mrkdwn",
					"text": "Email *TBD*"
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
					"url": "https://api.jameslittle.me/{}/delete"
				}}
			]
		}}
	]
            "#,
                self.name,
                self.message,
                self.id.to_hyphenated().to_string()
            ))
            .unwrap(),
        }
    }
}

impl TryFrom<GuestbookPostData> for GuestbookEntry {
    type Error = anyhow::Error;

    fn try_from(value: GuestbookPostData) -> Result<Self, Self::Error> {
        if value.message.len() > 1200 {
            return Err(Error::msg("Message must be <= 1200 letters."));
        }

        if value.name.len() > 600 {
            return Err(Error::msg("Name must be <= 600 characters"));
        }

        Ok(GuestbookEntry {
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

impl TryFrom<HashMap<String, AttributeValue>> for GuestbookEntry {
    type Error = anyhow::Error;
    fn try_from(value: HashMap<String, AttributeValue>) -> Result<Self> {
        fn parse_datetime(string_ms: &str) -> Result<DateTime<chrono::Utc>> {
            let ms: i64 = string_ms.parse()?;

            Ok(DateTime::from_utc(
                NaiveDateTime::from_timestamp(ms / 1000, 0),
                chrono::Utc,
            ))
        }

        let id = Uuid::from_str(
            value
                .get("id")
                .ok_or(Error::msg("No id attribute found"))?
                .as_s()
                .map_err(|_| Error::msg("id attribute is not a string"))?,
        )?;

        let created_at = value
            .get("created_at")
            .ok_or(Error::msg("No created_at attribute found"))?
            .as_s()
            .map(|str| parse_datetime(str))
            .map_err(|_| Error::msg("id attribute is not a string"))??;

        let deleted_at: Option<DateTime<chrono::Utc>> = {
            match value.get("deleted_at") {
                Some(AttributeValue::S(attribute)) => {
                    let deleted_at_ms: i64 = attribute.parse().unwrap();
                    Some(DateTime::from_utc(
                        NaiveDateTime::from_timestamp(
                            deleted_at_ms / 1000,
                            (deleted_at_ms % 1000) as u32,
                        ),
                        chrono::Utc,
                    ))
                }
                _ => None,
            }
        };

        let message = value
            .get("message")
            .ok_or(Error::msg("No message found"))?
            .as_s()
            .map_err(|_| Error::msg("Message is not a string"))?
            .to_string();

        let name = value
            .get("name")
            .ok_or(Error::msg("No name found"))?
            .as_s()
            .map_err(|_| Error::msg("Name is not a string"))?
            .to_string();

        let url = value
            .get("url")
            .map(|v| v.as_s().ok().map(|s| s.to_string()))
            .flatten();

        let email = value
            .get("email")
            .map(|v| v.as_s().ok().map(|s| s.to_string()))
            .flatten();

        Ok(GuestbookEntry {
            url,
            id,
            email,
            created_at,
            deleted_at,
            message,
            name,
        })
    }
}
