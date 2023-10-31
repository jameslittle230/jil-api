use std::net::TcpListener;

use jil_api::run;

pub async fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to port");
    let port = listener.local_addr().unwrap().port();
    let server = run(listener).await.expect("Failed to bind address");
    tokio::spawn(server);
    format!("http://127.0.0.1:{}", port)
}
