#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    viewlake_srv::run().await
}
