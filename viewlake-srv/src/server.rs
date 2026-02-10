use poem::{
    EndpointExt, Route, Server,
    listener::{Listener, RustlsCertificate, RustlsConfig, TcpListener},
    middleware::Tracing,
    post,
};

mod handlers;

fn setup_rustls_config(cert: &str, key: &str) -> Result<RustlsConfig, std::io::Error> {
    let cert_bytes = std::fs::read(cert)?;
    let key_bytes = std::fs::read(key)?;
    Ok(RustlsConfig::new().fallback(RustlsCertificate::new().cert(cert_bytes).key(key_bytes)))
}

pub async fn run(addr: &str, cert: &str, key: &str) -> Result<(), std::io::Error> {
    let config = setup_rustls_config(cert, key)?;
    let app = Route::new()
        .at("/api/v1/hello", post(handlers::hello))
        .at("/api/v1/sysinfo", post(handlers::sysinfo))
        .at("/api/v1/task", post(handlers::task))
        .with(Tracing);

    Server::new(TcpListener::bind(addr).rustls(config))
        .run(app)
        .await
}
