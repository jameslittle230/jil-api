use actix_web::web;
use aws_sdk_dynamodb::{error::PutItemError, output::PutItemOutput, types::SdkError};

use crate::{shortener::models::entry::Entry, AppState};

pub async fn put_shortlink_entry(
    state: web::Data<AppState>,
    entry: &Entry,
) -> Result<PutItemOutput, SdkError<PutItemError>> {
    state
        .dynamodb
        .put_item()
        .table_name("jil-link-shortener")
        .set_item(Some(entry.clone().into()))
        .set_condition_expression(Some("attribute_not_exists(shortname)".to_string()))
        .send()
        .await
}
