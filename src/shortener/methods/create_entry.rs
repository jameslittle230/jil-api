use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::AppState;

use crate::error::ApiError;
use crate::shortener::models::entry::Entry;
use crate::shortener::queries::put_entry::put_shortlink_entry;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct PostData {
    pub shortname: String,
    pub longurl: String,
}

pub(crate) async fn exec(
    state: web::Data<AppState>,
    payload: web::Json<PostData>,
) -> Result<HttpResponse, ApiError> {
    let entry = Entry {
        shortname: payload.shortname.clone(),
        created_at: chrono::Utc::now(),
        deleted_at: None,
        longurl: payload.longurl.clone(),
        clicks: 0,
    };

    if payload.shortname.is_empty() {
        return Err(ApiError::bad_request("Received an empty shortname"));
    }

    if payload.longurl.is_empty() {
        return Err(ApiError::bad_request("Received an empty longurl"));
    }

    let put_result = put_shortlink_entry(state, &entry).await;

    if let Err(err) = put_result {
        match &err {
            aws_sdk_dynamodb::types::SdkError::ServiceError { err, raw: _ } => match err.kind {
                aws_sdk_dynamodb::error::PutItemErrorKind::ConditionalCheckFailedException(_) => {
                    return Err(ApiError::bad_request(
                        format!("Short URL `{}` already exists", payload.shortname.clone())
                            .as_str(),
                    ));
                }
                _ => {}
            },
            _ => {}
        }

        return Err(ApiError::internal_server_error(err.to_string().as_str()));
    }

    // let _ = send_slack_message(&guestbook_entry.slack_api_request(req.peer_addr())).await;

    Ok(HttpResponse::Ok().body(serde_json::to_string(&entry).unwrap()))
}
