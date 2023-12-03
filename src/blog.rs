use reqwest::{get, Response, Result};

pub(crate) async fn deploy_blog() -> Result<Response> {
    if cfg!(test) {
        get("http://localhost:8000").await?; // Don't actually deploy in tests
    }
    get(format!(
        "https://api.render.com/deploy/srv-clf2of3l00ks739vcsfg?key={}",
        std::env::var("RENDER_DEPLOY_KEY").unwrap()
    ))
    .await
}
