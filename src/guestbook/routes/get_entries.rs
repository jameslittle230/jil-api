use actix_web::{get, web, HttpRequest, HttpResponse};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    guestbook::{get_undeleted_entries, DisplayableEntry},
    AppState,
};

#[get("/guestbook")]
pub async fn exec(req: HttpRequest, state: web::Data<AppState>) -> HttpResponse {
    let after = web::Query::<GetGuestbookQueryParameters>::from_query(req.query_string())
        .ok()
        .map(|params| params.after);

    let guestbook_entries = get_undeleted_entries(state, after).await;

    match guestbook_entries {
        Ok((total_count, items)) => {
            let output_items: Vec<DisplayableEntry> = items
                .iter()
                .map(|e| DisplayableEntry::from(e.clone()))
                .collect();

            let count = output_items.len();

            HttpResponse::Ok().json(GuestbookListResponse {
                items: output_items,
                count,
                total_count,
            })
        }
        Err(err) => HttpResponse::InternalServerError().body(format!("{}", err)),
    }
}

#[derive(Debug, Clone, Deserialize)]
struct GetGuestbookQueryParameters {
    after: Uuid,
}

#[derive(Debug, Serialize)]
struct GuestbookListResponse {
    items: Vec<DisplayableEntry>,
    count: usize,
    total_count: usize,
}
