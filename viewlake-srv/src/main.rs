use tracing_subscriber::{EnvFilter, fmt};

use viewlake_srv::server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    init_tracing();

    let addr = "127.0.0.1:3000";
    let cert_path = "ssl/cert.pem";
    let key_path = "ssl/key.pem";

    server::run(addr, cert_path, key_path).await?;

    Ok(())
}

fn init_tracing() {
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
}
