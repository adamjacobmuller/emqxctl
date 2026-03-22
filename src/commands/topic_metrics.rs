use crate::client::EmqxClient;
use crate::output::{Column, OutputFormatter};
use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum TopicMetricsCommand {
    List,
    Get { topic: String },
    Register { topic: String },
    Deregister { topic: String },
}

const LIST_COLUMNS: &[Column] = &[
    Column {
        header: "TOPIC",
        json_path: "topic",
        max_width: Some(50),
    },
    Column {
        header: "CREATE TIME",
        json_path: "create_time",
        max_width: None,
    },
    Column {
        header: "RESET TIME",
        json_path: "reset_time",
        max_width: None,
    },
];

pub async fn execute(
    client: &EmqxClient,
    fmt: &OutputFormatter,
    cmd: &TopicMetricsCommand,
) -> Result<()> {
    match cmd {
        TopicMetricsCommand::List => {
            let value = client.get("/topic-metrics").await?;
            let items = super::extract_items(&value);
            fmt.print_list(&items, LIST_COLUMNS, None, None);
        }
        TopicMetricsCommand::Get { topic } => {
            let value = client.get(&format!("/topic-metrics/{}", topic)).await?;
            fmt.print_value(&value);
        }
        TopicMetricsCommand::Register { topic } => {
            let body = serde_json::json!({ "topic": topic });
            client.post("/topic-metrics", &body).await?;
            fmt.print_success(&format!("Topic metrics registered for '{}'", topic));
        }
        TopicMetricsCommand::Deregister { topic } => {
            client.delete(&format!("/topic-metrics/{}", topic)).await?;
            fmt.print_success(&format!("Topic metrics deregistered for '{}'", topic));
        }
    }
    Ok(())
}
