use actix_web::web;
use anyhow::Result;

use crate::{guestbook::models::Entry, AppState};

pub async fn put_guestbook_entry(state: web::Data<AppState>, entry: &Entry) -> Result<()> {
    state
        .dynamodb
        .put_item()
        .table_name("jil-guestbook")
        .set_item(Some(entry.clone().into()))
        .send()
        .await?;

    Ok(())
}
