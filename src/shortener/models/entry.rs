use chrono::DateTime;
use dynomite::Item;
use serde::Serialize;
#[derive(Debug, Clone, Serialize, Item)]
pub struct Entry {
    #[dynomite(partition_key)]
    pub shortname: String,

    pub created_at: DateTime<chrono::Utc>,

    #[dynomite(default)]
    #[serde(skip_serializing)]
    pub deleted_at: Option<DateTime<chrono::Utc>>,

    pub longurl: String,
}
