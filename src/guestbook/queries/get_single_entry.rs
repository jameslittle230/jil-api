use actix_web::web;
use anyhow::{Error as AHError, Result};
use dynomite::AttributeValue;

use crate::{guestbook::models::entry::Entry, AppState};

pub(crate) async fn get_single_entry(state: &web::Data<AppState>, id: String) -> Result<Entry> {
    let entry: Entry = state
        .dynamodb
        .query()
        .table_name("jil-guestbook")
        .key_condition_expression("id = :value".to_string())
        .expression_attribute_values(":value".to_string(), AttributeValue::S(id.clone()))
        .send()
        .await?
        // .map_err(ErrorBadRequest)?
        .items
        .unwrap()
        .pop()
        .ok_or_else(|| AHError::msg(format!("No entry found with ID {id}")))?
        .try_into()?;

    Ok(entry)
}
