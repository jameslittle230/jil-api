mod test_utils;
use crate::test_utils::spawn_app;

#[tokio::test]
async fn healthcheck_returns_200() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/healthcheck", address))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), reqwest::StatusCode::OK);
    assert_eq!(response.text().await.unwrap(), "up");
}
