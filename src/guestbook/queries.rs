use anyhow::{Error, Result};
use dynomite::AttributeValue;
use uuid::Uuid;

use crate::guestbook::entry::Entry;

pub(crate) async fn get_single_entry(
    dynamodb: &aws_sdk_dynamodb::Client,
    id: &Uuid,
) -> Result<Entry> {
    if cfg!(test) {
        return Ok(Entry::default()); // Don't actually read from the database in tests
    }

    let entry: Entry = dynamodb
        .query()
        .table_name("jil-guestbook")
        .key_condition_expression("id = :value".to_string())
        .expression_attribute_values(":value".to_string(), AttributeValue::S(id.to_string()))
        .send()
        .await?
        .items
        .unwrap()
        .pop()
        .ok_or_else(|| Error::msg(format!("No entry found with ID {id}")))?
        .try_into()?;

    Ok(entry)
}

pub(crate) async fn get_undeleted_entries(
    dynamodb: &aws_sdk_dynamodb::Client,
    after: Option<Uuid>,
    include_qa: bool,
) -> Result<(usize, Vec<Entry>)> {
    let entries = if cfg!(test) {
        vec![
            // TODO: Add test data
        ] // Don't actually read from the database in tests
    } else {
        dynamodb
            .scan()
            .table_name("jil-guestbook")
            .send()
            .await?
            .items
            .ok_or_else(|| Error::msg("Could not get items from dynamodb"))?
            .into_iter()
            .map(Entry::try_from)
            .collect::<Result<Vec<Entry>, _>>()?
    };

    let total_size = entries.len();

    let mut filtered_entries: Vec<Entry> = entries
        .iter()
        .filter(|entry| entry.deleted_at.is_none() && (if include_qa { true } else { !entry.qa }))
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

pub async fn put_guestbook_entry(dynamodb: &aws_sdk_dynamodb::Client, entry: &Entry) -> Result<()> {
    if cfg!(test) {
        return Ok(()); // Don't actually write to the database in tests
    }

    dynamodb
        .put_item()
        .table_name("jil-guestbook")
        .set_item(Some(entry.clone().into()))
        .send()
        .await?;

    Ok(())
}
