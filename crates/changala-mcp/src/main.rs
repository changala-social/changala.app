//! Standalone MCP server binary (for independent deployment).
//! In most cases, mount MCP directly on the Ring via CHANGALA_MCP_ENABLED=true instead.

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("changala_mcp=info".parse()?),
        )
        .init();

    // Default to loopback — never bind 0.0.0.0 unless explicitly requested.
    let host = std::env::var("MCP_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("MCP_PORT").unwrap_or_else(|_| "3001".to_string());
    let bind = format!("{}:{}", host, port);

    tracing::info!(bind = %bind, "starting changala-mcp server");

    let service = changala_mcp::mcp_service();
    let router = axum::Router::new()
        .nest_service("/mcp", service)
        .layer(axum::middleware::from_fn(changala_mcp::mcp_auth_middleware));

    let listener = tokio::net::TcpListener::bind(&bind).await?;
    tracing::info!("MCP server listening on http://{}/mcp", bind);
    axum::serve(listener, router).await?;

    Ok(())
}
