use std::fmt::Display;

use actix_web::{
    error,
    http::{header::ContentType, StatusCode},
    HttpResponse,
};

#[derive(Debug)]
pub(crate) struct ApiError {
    message: String,
    status_code: StatusCode,
}

impl ApiError {
    pub(crate) fn bad_request(msg: &str) -> Self {
        Self {
            message: msg.to_string(),
            status_code: StatusCode::BAD_REQUEST,
        }
    }

    pub(crate) fn internal_server_error(msg: &str) -> Self {
        Self {
            message: msg.to_string(),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        Self {
            message: err.to_string(),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            serde_json::json!({
                "error": true,
                "code": self.status_code.to_string(),
                "message": self.message
            })
            .to_string()
            .as_str(),
        )
    }
}

impl error::ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        self.status_code
    }
}
