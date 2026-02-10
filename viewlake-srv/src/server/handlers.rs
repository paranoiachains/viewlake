use poem::{handler, http};

use tracing::{debug, error, info};
use viewlake_agent::collector;

#[handler]
pub async fn hello(body: poem::Body) -> poem::Result<http::StatusCode> {
    let bytes = body.into_bytes().await?;

    info!("got initial message, agent id: {:?}", bytes);

    Ok(http::StatusCode::OK)
}

#[handler]
pub async fn sysinfo(body: poem::Body) -> poem::Result<http::StatusCode> {
    let bytes = body.into_bytes().await?;

    let fingerprint: collector::SystemFingerprint = match postcard::from_bytes(&bytes) {
        Ok(fprint) => fprint,
        Err(e) => {
            error!("got deserialization error: {e}");
            return Err(poem::Error::new(e, http::StatusCode::INTERNAL_SERVER_ERROR));
        }
    };

    debug!(
        "fingerprint deserialized: {:?}",
        fingerprint.network.hostname
    );

    Ok(http::StatusCode::OK)
}

#[handler]
pub async fn task() -> &'static str {
    "sleep:5"
}
