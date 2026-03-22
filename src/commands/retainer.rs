use crate::cli::PaginationArgs;
use crate::client::EmqxClient;
use crate::output::{Column, OutputFormatter};
use anyhow::Result;
use clap::Subcommand;
use reqwest::Method;

#[derive(Subcommand)]
pub enum RetainerCommand {
    List {
        #[command(flatten)]
        pagination: PaginationArgs,
    },
    Get {
        topic: String,
    },
    Delete {
        topic: String,
    },
    Config,
    #[command(name = "config-update")]
    ConfigUpdate {
        #[arg(short = 'f', long)]
        file: String,
    },
}

const LIST_COLUMNS: &[Column] = &[
    Column {
        header: "TOPIC",
        json_path: "topic",
        max_width: Some(50),
    },
    Column {
        header: "QOS",
        json_path: "qos",
        max_width: None,
    },
    Column {
        header: "PUBLISH AT",
        json_path: "publish_at",
        max_width: None,
    },
];

pub async fn execute(
    client: &EmqxClient,
    fmt: &OutputFormatter,
    cmd: &RetainerCommand,
) -> Result<()> {
    match cmd {
        RetainerCommand::List { pagination } => {
            super::handle_paginated_list(
                client,
                fmt,
                "/retainer/messages",
                &[],
                pagination,
                LIST_COLUMNS,
                None,
            )
            .await?;
        }
        RetainerCommand::Get { topic } => {
            let value = client.get(&format!("/retainer/message/{}", topic)).await?;
            fmt.print_value(&value);
        }
        RetainerCommand::Delete { topic } => {
            super::handle_delete(
                client,
                fmt,
                &format!("/retainer/message/{}", topic),
                &format!("Retained message for '{}' deleted", topic),
            )
            .await?;
        }
        RetainerCommand::Config => {
            let value = client.get("/retainer").await?;
            fmt.print_value(&value);
        }
        RetainerCommand::ConfigUpdate { file } => {
            super::handle_create_or_update(
                client,
                fmt,
                Method::PUT,
                "/retainer",
                file,
                "Retainer config updated",
            )
            .await?;
        }
    }
    Ok(())
}
