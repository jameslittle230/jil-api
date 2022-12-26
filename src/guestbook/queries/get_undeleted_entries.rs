use actix_web::web;
use anyhow::{Error as AHError, Result};
use uuid::Uuid;

use crate::{guestbook::models::entry::Entry, AppState};

pub(crate) async fn get_undeleted_entries(
    state: web::Data<AppState>,
    after: Option<Uuid>,
) -> Result<(usize, Vec<Entry>)> {
    let scan_output = state
        .dynamodb
        .scan()
        .table_name("jil-guestbook")
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

    filtered_entries = filtered_entries
        .into_iter()
        .skip_while(|entry| {
            if let Some(after) = after {
                entry.id != after
            } else {
                false
            }
        })
        .skip(1) // Don't return the element that matches the after thing
        .collect();

    Ok((total_size, filtered_entries))
}
