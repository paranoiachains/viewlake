use poem::{
    Body, EndpointExt, Route, Server,
    error::ReadBodyError,
    handler,
    http::StatusCode,
    listener::{Listener, RustlsCertificate, RustlsConfig, TcpListener},
    middleware::Tracing,
    post,
};

use viewlake_agent::collector::SystemFingerprint;

use tracing::{debug, error, info};

fn setup_rustls_config(cert: &str, key: &str) -> Result<RustlsConfig, std::io::Error> {
    let cert_bytes = std::fs::read(cert)?;
    let key_bytes = std::fs::read(key)?;

    Ok(RustlsConfig::new().fallback(RustlsCertificate::new().cert(cert_bytes).key(key_bytes)))
}

pub async fn run(addr: &str, cert: &str, key: &str) -> Result<(), std::io::Error> {
    let config = setup_rustls_config(cert, key)?;
    let app = Route::new()
        .at("/api/v1/hello", post(hello))
        .at("/api/v1/sysinfo", post(sysinfo))
        .with(Tracing);

    Server::new(TcpListener::bind(addr).rustls(config))
        .run(app)
        .await
}

#[handler]
async fn hello(body: Body) -> poem::Result<()> {
    let bytes = body.into_bytes().await?;

    let fingerprint: SystemFingerprint = match postcard::from_bytes(&bytes) {
        Ok(fprint) => fprint,
        Err(e) => {
            error!("got deserialization error: {e}");
            return Err(poem::Error::new(e, StatusCode::INTERNAL_SERVER_ERROR));
        }
    };

    debug!(
        "fingerprint deserialized: {:#?}",
        fingerprint.network.hostname
    );

    info!("got initial message, agent id: {:?}", bytes);

    Ok(())
}

#[handler]
async fn sysinfo(body: Body) -> Result<(), ReadBodyError> {
    let bytes = body.into_bytes().await?;

    debug!("got sysinfo: {:?}", bytes);

    Ok(())
}
