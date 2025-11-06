use axum::routing::get;

pub async fn run() -> Result<(), std::io::Error> {
    let app = axum::Router::new().route("/", get(|| async {}));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3333")
        .await
        .expect("Error while creating listener");

    axum::serve(listener, app).await
}
