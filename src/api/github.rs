use actix_web::{get, HttpResponse};

#[get("/github/stork-stars")]
pub async fn get_github_stork_stars() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!(
        {
            "stargazers": 0,
            "has_notifs": false,
            "message": "This API endpoint is deprecated."
        }
    ))
}
