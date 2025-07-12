#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    rusted_ca::infrastructure::web::run::run().await
}
