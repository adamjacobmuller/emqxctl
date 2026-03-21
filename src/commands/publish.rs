use anyhow::Result;
use clap::Args;
use crate::client::EmqxClient;
use crate::input::read_input_file;
use crate::output::OutputFormatter;

#[derive(Args)]
pub struct PublishArgs {
    /// Topic to publish to
    #[arg(short = 't', long)]
    pub topic: String,
    /// Message payload
    #[arg(short = 'p', long)]
    pub payload: String,
    /// QoS level (0, 1, 2)
    #[arg(short = 'q', long, default_value = "0")]
    pub qos: u8,
    /// Retain flag
    #[arg(long)]
    pub retain: bool,
}

pub async fn execute(client: &EmqxClient, fmt: &OutputFormatter, args: &PublishArgs) -> Result<()> {
    let body = serde_json::json!({
        "topic": args.topic,
        "payload": args.payload,
        "qos": args.qos,
        "retain": args.retain,
    });
    client.post("/publish", &body).await?;
    fmt.print_success(&format!("Published to '{}'", args.topic));
    Ok(())
}

pub async fn execute_batch(client: &EmqxClient, fmt: &OutputFormatter, file: &str) -> Result<()> {
    let body = read_input_file(file)?;
    client.post("/publish/bulk", &body).await?;
    fmt.print_success("Batch publish completed");
    Ok(())
}
