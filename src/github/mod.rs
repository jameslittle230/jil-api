use actix_web::{web, HttpResponse};
use serde::Serialize;

#[derive(Serialize)]
struct StorkStarsResponse {
    stargazers: usize,
    has_notifs: bool,
    message: String,
}

pub(crate) fn cfg(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/stork-stars").route(web::get().to(stork_stars)));
}

pub async fn stork_stars() -> HttpResponse {
    HttpResponse::Ok().json(StorkStarsResponse {
        stargazers: 0,
        has_notifs: false,
        message: "This API endpoint is deprecated.".to_string(),
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_notifs_github_api_response_empty() {
        let json = r#"[]"#;
        let response: Vec<String> = serde_json::from_str(json).unwrap();
        assert_eq!(response.len(), 0);
    }

    #[test]
    fn test_notifs_github_api_response_full() {
        let json = r#"[{"a": 12, "c": "d"}, {"e": "f", "g": {"h": "i"}}]"#;
        let response: Vec<serde_json::Value> = serde_json::from_str(json).unwrap();
        assert_eq!(response.len(), 2);
    }
}
