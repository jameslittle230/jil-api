use std::net::TcpListener;

use jil_api::run;

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    let port = std::env::var("PORT").unwrap_or_else(|_| "0".to_string());

    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
        .unwrap_or_else(|_| panic!("Failed to bind to port {}", port));

    println!(
        "Server starting on http://{}",
        listener.local_addr().unwrap()
    );

    let _ = run(listener).await?.await;
    Ok(())
}
