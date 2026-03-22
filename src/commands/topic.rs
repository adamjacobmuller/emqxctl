use crate::cli::PaginationArgs;
use crate::client::EmqxClient;
use crate::output::{Column, OutputFormatter};
use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum TopicCommand {
    /// List topics
    List {
        #[command(flatten)]
        pagination: PaginationArgs,
        #[arg(short = 't', long)]
        topic: Option<String>,
    },
    /// Get topic details
    Get { topic: String },
}

const LIST_COLUMNS: &[Column] = &[
    Column {
        header: "TOPIC",
        json_path: "topic",
        max_width: Some(50),
    },
    Column {
        header: "NODE",
        json_path: "node",
        max_width: None,
    },
];

pub async fn execute(client: &EmqxClient, fmt: &OutputFormatter, cmd: &TopicCommand) -> Result<()> {
    match cmd {
        TopicCommand::List { pagination, topic } => {
            let mut query: Vec<(&str, String)> = Vec::new();
            if let Some(ref t) = topic {
                query.push(("topic", t.clone()));
            }
            super::handle_paginated_list(
                client,
                fmt,
                "/topics",
                &query,
                pagination,
                LIST_COLUMNS,
                None,
            )
            .await?;
        }
        TopicCommand::Get { topic } => {
            let value = client
                .get(&format!("/topics/{}", urlencoding_simple(topic)))
                .await?;
            fmt.print_value(&value);
        }
    }
    Ok(())
}

fn urlencoding_simple(s: &str) -> String {
    s.replace('/', "%2F")
        .replace('#', "%23")
        .replace('+', "%2B")
}
