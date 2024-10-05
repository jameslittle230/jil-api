use actix_web::{get, http::header::ContentType, post, web, Either, HttpRequest, HttpResponse};
use minijinja::render;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{
    blog::deploy_blog,
    error::ApiError,
    guestbook::{
        entry::Entry,
        queries::{get_single_entry, get_undeleted_entries, put_guestbook_entry},
    },
    slack::send_slack_message,
};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub(crate) struct GuestbookForm {
    #[schema(example = "John Appleseed", max_length = 600)]
    pub name: String,

    #[schema(example = "Just stopping by to say hello!", max_length = 1200)]
    pub message: String,

    #[schema(example = "john@example.com")]
    pub email: Option<String>,

    #[schema(example = "https://example.com")]
    pub url: Option<String>,

    #[serde(default)]
    #[schema(example = true)]
    pub qa: bool,
}

/// Create a Guestbook Entry
///
/// Adds an entry to the guestbook.
///
/// Successfully submitting this API will also redeploy my blog, which displays
/// all the guestbook entries.
///
/// Passing the `qa: true` option will still create an entry in the backing database,
/// and all side effects will still take place, but the new entry will not be displayed
/// on my blog.
#[utoipa::path(
    request_body(content = inline(GuestbookForm)),
    responses(
        (status=200, description = "Success response", body = inline(Entry))
    ),
    tag = "Guestbook"
)]
#[post("/guestbook")]
pub(crate) async fn post_guestbook(
    req: HttpRequest,
    data: Either<web::Form<GuestbookForm>, web::Json<GuestbookForm>>,
    state: web::Data<crate::AppState>,
) -> Result<HttpResponse, ApiError> {
    let guestbook_form = data.into_inner();
    let mut guestbook_entry = Entry::try_from(guestbook_form)?;

    put_guestbook_entry(&state.dynamodb, &guestbook_entry).await?;

    let _ = send_slack_message(&guestbook_entry.slack_api_request(req.peer_addr())).await;

    if !guestbook_entry.qa {
        let _ = deploy_blog().await;
    }

    if guestbook_entry.qa {
        guestbook_entry.push_ser_option("serialize_qa");
    }
    Ok(HttpResponse::Ok().json(&guestbook_entry))
}

#[derive(Debug, Deserialize, IntoParams)]
pub(crate) struct GetGuestbookQueryParameters {
    pub after: Option<uuid::Uuid>,

    #[serde(default)]
    pub qa: bool,

    #[serde(default)]
    pub htmx: bool,
}

#[derive(Debug, Serialize, ToSchema)]
struct GetGuestbookResponse {
    #[schema(example = json!([{
        "id": "fefedb65-0d84-4d96-8b52-162799098cc6",
        "created_at": "2023-07-13T13:04:48.264445125+00:00",
        "url": "example.com",
        "message": "Excited to visit your website!",
        "name": "Paulo"
      },
      {
        "id": "d203030a-8edc-4ada-9d77-6417fb110f33",
        "created_at": "2024-04-05T16:11:03.657861836+00:00",
        "url": null,
        "message": "Woooooo",
        "name": "Mat"
      }]))]
    items: Vec<Entry>,

    #[schema(example = "10")]
    count: usize,

    #[schema(example = "15")]
    total_count: usize,
}

/// List Guestbook Entries
///
/// List all guestbook entries in order of creation date.
///
/// Passing the `after` query parameter lets you only display entries after the given ID,
/// not including the entry whose ID was passed in.
///
/// Passing the `qa` query parameter will include all entries, including the ones
/// submitted with the `QA` field, in the response.
///
/// Passing with the `htmx` query parameter will render the listed entries as a set of
/// HTML `<li>` elements.
///
/// There is no way to display deleted guestbook entries; however, the `total_count`
/// field in the response will include deleted entries. This was probably a mistake.
#[utoipa::path(
    params(GetGuestbookQueryParameters),
    responses(
        (status=200, description = "Success response", body = inline(GetGuestbookResponse))
    ),
    tag = "Guestbook"
)]
#[get("/guestbook")]
pub(crate) async fn get_guestbook(
    query: web::Query<GetGuestbookQueryParameters>,
    state: web::Data<crate::AppState>,
) -> Result<HttpResponse, ApiError> {
    let (total_count, mut guestbook_entries) =
        get_undeleted_entries(&state.dynamodb, query.after, query.qa).await?;

    let count = &guestbook_entries.len();

    if query.htmx {
        guestbook_entries.reverse();
        let r = render!(HTMX_GUESTBOOK_LIST_TEMPLATE, entries => guestbook_entries );
        Ok(HttpResponse::Ok().content_type(ContentType::html()).body(r))
    } else {
        Ok(HttpResponse::Ok().json(GetGuestbookResponse {
            items: guestbook_entries,
            count: *count,
            total_count,
        }))
    }
}

/// Get a Single Guestbook Entry
///
/// Returns a single guestbook entry based on the ID.
#[utoipa::path(
    responses(
        (status=200, description = "Success response", body = inline(Entry))
    ),
    tag = "Guestbook"
)]
#[get("/guestbook/{id}")]
pub(crate) async fn get_guestbook_entry(
    path: web::Path<uuid::Uuid>,
    state: web::Data<crate::AppState>,
) -> Result<HttpResponse, ApiError> {
    let entry_id = path.into_inner();
    let mut entry = get_single_entry(&state.dynamodb, &entry_id).await?;
    entry.push_ser_option("serialize_deleted_at");
    entry.push_ser_option("serialize_qa");
    Ok(HttpResponse::Ok().json(&entry))
}

/// Delete a Guestbook Entry
///
/// Deletes the guestbook entry with the given ID, then returns the newly-deleted object.
///
/// This endpoint must be called with a bearer token header:
///
/// ```
/// Authorization: Bearer admin
/// ```
#[utoipa::path(
    responses(
        (status=200, description = "Success response", body = inline(Entry))
    ),
    tag = "Guestbook"
)]
#[post("/guestbook/{id}/delete")]
pub(crate) async fn delete_guestbook_entry(
    path: web::Path<uuid::Uuid>,
    state: web::Data<crate::AppState>,
) -> Result<HttpResponse, ApiError> {
    let entry_id = path.into_inner();
    let mut entry = get_single_entry(&state.dynamodb, &entry_id).await?;
    entry.deleted_at = Some(chrono::Utc::now());
    put_guestbook_entry(&state.dynamodb, &entry).await?;
    entry.push_ser_option("serialize_deleted_at");
    entry.push_ser_option("serialize_qa");

    Ok(HttpResponse::Ok().json(&entry))
}

const HTMX_GUESTBOOK_LIST_TEMPLATE: &'static str = r#"
{% for entry in entries %}
<li class="guestbook-entry" data-entry="{{entry.id}}">
    <div class="guestbook-metadata" hx-disable>
        <span class="name">{{entry.name}}</span>
        <div>
        <span class="timestamp metadata">
            <span>Just now</span>
        </span>
        {% if entry.url %}
        <span class="url">
            <a href="{{entry.url}}">{{entry.url}}</a>
        </span>
        {% endif %}
        </div>
    </div>
    <div class="message" hx-disable>
        <p>{{entry.message}}</p>
    </div>
</li>
{% endfor %}
"#;
