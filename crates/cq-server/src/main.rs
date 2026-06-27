#[tokio::main]
async fn main() -> anyhow::Result<()> {
    cq_server::bootstrap::run().await
}
