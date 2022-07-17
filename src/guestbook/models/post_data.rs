use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GuestbookPostData {
    pub name: String,
    pub message: String,
    pub email: Option<String>,
    pub url: Option<String>,
}
