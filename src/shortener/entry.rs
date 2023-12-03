use chrono::DateTime;
use dynomite::Item;
use serde::{Deserialize, Serialize};

use crate::api::shortener::CreateEntryForm;
#[derive(Debug, Clone, Serialize, Deserialize, Item)]
pub struct Entry {
    #[dynomite(partition_key)]
    pub shortname: String,

    pub created_at: DateTime<chrono::Utc>,

    #[dynomite(default)]
    #[serde(skip_serializing)]
    pub deleted_at: Option<DateTime<chrono::Utc>>,

    pub longurl: String,

    #[dynomite(default)]
    pub clicks: u32,
}

impl TryFrom<CreateEntryForm> for Entry {
    type Error = anyhow::Error;

    fn try_from(form: CreateEntryForm) -> Result<Self, Self::Error> {
        if form.shortname.is_empty() {
            return Err(anyhow::anyhow!("Received an empty shortname"));
        }

        if form.longurl.is_empty() {
            return Err(anyhow::anyhow!("Received an empty longurl"));
        }

        Ok(Entry {
            shortname: form.shortname,
            created_at: chrono::Utc::now(),
            deleted_at: None,
            longurl: form.longurl,
            clicks: 0,
        })
    }
}
