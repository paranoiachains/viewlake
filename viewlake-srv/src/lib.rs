use poem::{
    Body, Route, Server, handler,
    listener::{Listener, RustlsCertificate, RustlsConfig, TcpListener},
    post,
};

fn setup_rustls_config(cert: &str, key: &str) -> Result<RustlsConfig, std::io::Error> {
    let cert_bytes = std::fs::read(cert)?;
    let key_bytes = std::fs::read(key)?;

    Ok(RustlsConfig::new().fallback(RustlsCertificate::new().cert(cert_bytes).key(key_bytes)))
}

pub async fn run(addr: &str, cert: &str, key: &str) -> Result<(), std::io::Error> {
    let config = setup_rustls_config(cert, key)?;
    let app = Route::new().at("/hello", post(hello));
    Server::new(TcpListener::bind(addr).rustls(config))
        .run(app)
        .await
}

#[handler]
async fn hello(data: Body) -> String {
    format!(
        "hello! body: {:?}",
        data.into_string()
            .await
            .unwrap_or("Couldn't read body".to_string())
    )
}
