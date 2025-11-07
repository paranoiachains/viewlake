use axum::{Router, body::Bytes, http::StatusCode, routing::post};

pub async fn run() -> Result<(), std::io::Error> {
    let app = Router::new().route("/hi", post(hi));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3333")
        .await
        .expect("Failed to bind");

    println!("Listening on 127.0.0.1:3333");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn hi(body: Bytes) -> StatusCode {
    let body_str = String::from_utf8_lossy(&body);
    println!("Received body: {}", body_str);

    StatusCode::OK
}
