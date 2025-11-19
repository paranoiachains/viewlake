#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = "127.0.0.1:3000";
    let cert_path = "ssl/cert.pem";
    let key_path = "ssl/key.pem";

    viewlake_srv::run(addr, cert_path, key_path).await?;

    Ok(())
}
