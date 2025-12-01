use anyhow::Result;
use claude_mcp_rs::server::ClaudeServer;
use rmcp::{transport::stdio, ServiceExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Create an instance of our Claude server
    let service = ClaudeServer::new().serve(stdio()).await.inspect_err(|e| {
        eprintln!("serving error: {:?}", e);
    })?;

    service.waiting().await?;
    Ok(())
}
