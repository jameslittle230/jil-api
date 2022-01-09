use actix_web::{get, HttpResponse};
use anyhow::Result;
use futures::join;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct StorkStarsResponse {
    stargazers: usize,
    has_notifs: bool,
}

#[derive(Deserialize)]
struct StorkStarsGithubApiResponse {
    stargazers_count: usize,
}

async fn get_stork_stargazers_count() -> Result<usize> {
    let client = reqwest::Client::new();

    let resp = client
        .get("https://api.github.com/repos/jameslittle230/stork")
        .header("User-Agent", "actix-web-jil-api")
        .send()
        .await?;

    let body = resp.text().await?;
    let api_response: StorkStarsGithubApiResponse = serde_json::from_str(&body)?;
    Ok(api_response.stargazers_count)
}

async fn get_stork_has_notifs() -> Result<bool> {
    let client = reqwest::Client::new();

    let token = std::env::var("GITHUB_TOKEN")?;

    let resp = client
        .get("https://api.github.com/notifications")
        .header("User-Agent", "actix-web-jil-api")
        .header("Authorization", format!("Basic {}", token))
        .send()
        .await?;

    let body = resp.text().await?;
    let api_response: Vec<serde_json::Value> = serde_json::from_str(&body)?;
    Ok(api_response.is_empty())
}

#[get("/github/stork-stars")]
pub async fn stork_stars() -> HttpResponse {
    let joined_results = join!(get_stork_stargazers_count(), get_stork_has_notifs());

    match joined_results {
        (Ok(stargazers), Ok(has_notifs)) => HttpResponse::Ok().json(StorkStarsResponse {
            stargazers,
            has_notifs,
        }),

        (Err(err), _) | (_, Err(err)) => HttpResponse::BadRequest().body(format!("{}", err)),
    }
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
