use actix_web::web;
use anyhow::{Error as AHError, Result};

use crate::{shortener::models::entry::Entry, AppState};

pub async fn list_shortlink_entries(state: web::Data<AppState>) -> Result<(usize, Vec<Entry>)> {
    let scan_output = state
        .dynamodb
        .scan()
        .table_name("jil-link-shortener")
        .send()
        .await?;

    let items = scan_output
        .items
        .ok_or_else(|| AHError::msg("Could not get items from dynamodb"))?;

    let entries = items
        .into_iter()
        .map(Entry::try_from)
        .collect::<Result<Vec<Entry>, _>>()?;

    let total_size = entries.len();

    let mut filtered_entries: Vec<Entry> = entries
        .iter()
        .filter(|entry| entry.deleted_at.is_none())
        .cloned()
        .collect();

    filtered_entries.sort_by_key(|entry| entry.created_at);

    Ok((total_size, filtered_entries))
}
