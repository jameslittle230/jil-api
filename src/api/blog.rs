use actix_web::{get, HttpResponse};

/// Deploy my Blog
///
/// Forwards a request to Render, the PaaS platform that hosts my blog, to rebuild
/// and redeploy the static site service.
///
/// This endpoint must be called with a bearer token header:
///
/// ```
/// Authorization: Bearer admin
/// ```
#[utoipa::path(
    responses(
        (status=200, description = "Success response")
    ),
    tag = "Generic",
    security(
        ("api_key" = []),
    )
)]
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
