use anyhow::Result;
use crate::client::EmqxClient;
use crate::output::OutputFormatter;

pub async fn execute(client: &EmqxClient, fmt: &OutputFormatter) -> Result<()> {
    let value = client.get("/status").await?;
    fmt.print_value(&value);
    Ok(())
}
