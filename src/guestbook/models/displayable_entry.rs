use chrono::DateTime;
use serde::Serialize;
use uuid::Uuid;

use super::Entry;

#[derive(Debug, Serialize)]
pub struct DisplayableEntry {
    pub id: Uuid,
    pub created_at: DateTime<chrono::Utc>,
    pub url: Option<String>,
    pub message: String,
    pub name: String,
}

impl From<Entry> for DisplayableEntry {
    fn from(guestbook_entry: Entry) -> Self {
        Self {
            id: guestbook_entry.id,
            created_at: guestbook_entry.created_at,
            url: guestbook_entry.url.clone(),
            message: guestbook_entry.message.clone(),
            name: guestbook_entry.name,
        }
    }
}
