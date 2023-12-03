use actix_web::{get, HttpResponse};

#[get("/blog/deploy")]
pub async fn get_blog_deploy() -> HttpResponse {
    let text = crate::blog::deploy_blog()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    HttpResponse::Ok().body(text)
}
