use actix_web::web;
use anyhow::{Error as AHError, Result};
use uuid::Uuid;

use crate::{guestbook::models::Entry, AppState};

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

    let mut did_see_after_uuid = false;

    let mut entries: Vec<Entry> = items
        .into_iter()
        .map(Entry::try_from)
        .filter_map(|res| res.ok())
        .filter(|entry| entry.deleted_at.is_none())
        .collect();

    let total_size = entries.len();
    entries.sort_by_key(|entry| entry.created_at);

    entries = entries
        .into_iter()
        .skip_while(|entry| match after {
            Some(after) => {
                if entry.id == after {
                    did_see_after_uuid = true;
                    return true; // don't include the one whose ID was specified
                }

                !did_see_after_uuid
            }
            None => false,
        })
        .collect();

    Ok((total_size, entries))
}
