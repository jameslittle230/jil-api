mod test_utils;
use serde_json::json;

use crate::test_utils::spawn_app;

#[tokio::test]
async fn guestbook_post_takes_form_data() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .post(&format!("{}/guestbook", address))
        .form(&[("name", "lemon"), ("message", "hello")])
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), reqwest::StatusCode::OK);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&response.text().await.unwrap()).unwrap(),
        json!({
            "name": "lemon",
            "message": "hello",
            "email": null,
            "url": null,
            "qa": false,
        })
    );
}

#[tokio::test]
async fn guestbook_post_takes_json_data() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .post(&format!("{}/guestbook", address))
        .header("Content-Type", "application/json")
        .body(
            serde_json::to_string(&json!({
                "name": "lemon",
                "message": "hello",
            }))
            .unwrap(),
        )
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let response_value =
        serde_json::from_str::<serde_json::Value>(&response.text().await.unwrap()).unwrap();
    println!("{:#?}", response_value);
    assert_eq!(response_value.get("name").unwrap(), "lemon");
    assert_eq!(response_value.get("message").unwrap(), "hello");
    assert!(response_value.get("url").unwrap().is_null());
    assert!(response_value.get("email").is_none()); // don't ever reveal the email while serializing
    assert!(response_value.get("created_at").is_some());
    assert!(response_value.get("id").is_some());
}
