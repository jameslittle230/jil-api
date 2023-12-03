use crate::shortener::entry::Entry;
use anyhow::{Error, Result};

pub async fn put_shortlink_entry(dynamodb: &aws_sdk_dynamodb::Client, entry: &Entry) -> Result<()> {
    dynamodb
        .put_item()
        .table_name("jil-link-shortener")
        .set_item(Some(entry.clone().into()))
        .set_condition_expression(Some("attribute_not_exists(shortname)".to_string()))
        .send()
        .await?;

    Ok(())
}

pub async fn list_shortlink_entries(
    dynamodb: &aws_sdk_dynamodb::Client,
) -> Result<(usize, Vec<Entry>)> {
    let scan_output = dynamodb
        .scan()
        .table_name("jil-link-shortener")
        .send()
        .await?;

    let items = scan_output
        .items
        .ok_or_else(|| Error::msg("Could not get items from dynamodb"))?;

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
