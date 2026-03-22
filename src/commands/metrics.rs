use crate::client::EmqxClient;
use crate::output::OutputFormatter;
use anyhow::Result;

pub async fn execute_metrics(
    client: &EmqxClient,
    fmt: &OutputFormatter,
    node: Option<&str>,
) -> Result<()> {
    let value = if let Some(node) = node {
        client.get(&format!("/nodes/{}/metrics", node)).await?
    } else {
        client.get("/metrics").await?
    };
    fmt.print_value(&value);
    Ok(())
}

pub async fn execute_stats(
    client: &EmqxClient,
    fmt: &OutputFormatter,
    node: Option<&str>,
) -> Result<()> {
    let value = if let Some(node) = node {
        client.get(&format!("/nodes/{}/stats", node)).await?
    } else {
        client.get("/stats").await?
    };
    fmt.print_value(&value);
    Ok(())
}
