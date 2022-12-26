use actix_web::{web, HttpRequest, HttpResponse};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    guestbook::{models::entry::Entry, queries::get_undeleted_entries::get_undeleted_entries},
    AppState,
};

pub async fn exec(req: HttpRequest, state: web::Data<AppState>) -> HttpResponse {
    let after = web::Query::<GetGuestbookQueryParameters>::from_query(req.query_string())
        .ok()
        .map(|params| params.after);

    let guestbook_entries = get_undeleted_entries(state, after).await;

    match guestbook_entries {
        Ok((total_count, items)) => {
            let count = items.len();
            HttpResponse::Ok().json(GuestbookListResponse {
                items,
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
    items: Vec<Entry>,
    count: usize,
    total_count: usize,
}
