use std::{ops::Not, str::FromStr};

use actix_web::{post, web, HttpResponse};
use aws_sdk_dynamodb::model::{
    AttributeDefinition, KeySchemaElement, KeyType, ProvisionedThroughput, ScalarAttributeType,
};
use chrono::{DateTime, NaiveDateTime};
use dynomite::{AttributeError, Item};
use serde::Serialize;
use uuid::Uuid;

use crate::{guestbook_entry::GuestbookEntry, AppState};

#[derive(Debug, Clone, Serialize, Item)]
pub struct StringTypedGuestbookEntry {
    #[dynomite(partition_key)]
    pub id: String,

    pub created_at: String,

    #[dynomite(default)]
    pub deleted_at: Option<String>,

    #[dynomite(default)]
    url: Option<String>,

    #[dynomite(default)]
    email: Option<String>,

    message: String,

    name: String,
}

impl From<StringTypedGuestbookEntry> for GuestbookEntry {
    fn from(value: StringTypedGuestbookEntry) -> Self {
        return Self {
            id: Uuid::from_str(value.id.as_str()).unwrap(),
            created_at: DateTime::from_utc(
                NaiveDateTime::from_timestamp(
                    value.created_at.parse::<i64>().unwrap() / 1000,
                    (value.created_at.parse::<i64>().unwrap() % 1000) as u32,
                ),
                chrono::Utc,
            ),
            deleted_at: match value.deleted_at {
                Some(str) => Some(DateTime::from_utc(
                    NaiveDateTime::from_timestamp(
                        str.parse::<i64>().unwrap() / 1000,
                        (str.parse::<i64>().unwrap() % 1000) as u32,
                    ),
                    chrono::Utc,
                )),
                None => None,
            },
            url: value
                .url
                .and_then(|string| string.is_empty().not().then(|| string)),
            email: value
                .email
                .and_then(|string| string.is_empty().not().then(|| string)),
            message: value.message,
            name: value.name,
        };
    }
}

#[post("/migrations/2022-03-20-01-new-guestbook-table/perform")]
pub async fn migration_2022_03_20_01(state: web::Data<AppState>) -> HttpResponse {
    // Step 1: Create a new table
    println!(
        "create table result {:#?}",
        state
            .dynamodb
            .create_table()
            .table_name("jil-guestbook")
            .key_schema(
                KeySchemaElement::builder()
                    .attribute_name("id")
                    .key_type(KeyType::Hash)
                    .build(),
            )
            .attribute_definitions(
                AttributeDefinition::builder()
                    .attribute_name("id")
                    .attribute_type(ScalarAttributeType::S)
                    .build(),
            )
            .provisioned_throughput(
                ProvisionedThroughput::builder()
                    .read_capacity_units(1)
                    .write_capacity_units(1)
                    .build(),
            )
            .send()
            .await
    );

    HttpResponse::Ok().body("")
}

#[post("/migrations/2022-03-20-02-migrate-guestbook-entries/perform")]
pub async fn migration_2022_03_20_02(state: web::Data<AppState>) -> HttpResponse {
    // Step 2: Get existing entries and convert to new data type
    let scan_output = state
        .dynamodb
        .scan()
        .table_name("jil-guestbook-9552749")
        .send()
        .await;

    let items = scan_output.unwrap().items.unwrap();

    let guestbook_entries: Vec<Result<StringTypedGuestbookEntry, AttributeError>> = items
        .into_iter()
        .map(StringTypedGuestbookEntry::try_from)
        .collect();

    let entries: Vec<GuestbookEntry> = guestbook_entries
        .into_iter()
        .filter_map(|res| res.ok())
        .map(GuestbookEntry::from)
        .collect();

    // Step 3: Save new data type entries to just-created table

    for entry in entries {
        dbg!(&entry);
        println!(
            "put_item() result {:#?}",
            state
                .dynamodb
                .put_item()
                .table_name("jil-guestbook")
                .set_item(Some(entry.clone().into())) // <= convert book into it's attribute map representation
                .send()
                .await
        );
    }

    HttpResponse::Ok().body("")
}
