use anyhow::Result;
use clap::Subcommand;
use crate::cli::PaginationArgs;
use crate::client::EmqxClient;
use crate::output::{Column, OutputFormatter};

#[derive(Subcommand)]
pub enum SubscriptionCommand {
    /// List subscriptions
    List {
        #[command(flatten)]
        pagination: PaginationArgs,
        #[arg(long)]
        clientid: Option<String>,
        #[arg(short = 't', long)]
        topic: Option<String>,
    },
}

const LIST_COLUMNS: &[Column] = &[
    Column { header: "CLIENTID", json_path: "clientid", max_width: Some(30) },
    Column { header: "TOPIC", json_path: "topic", max_width: Some(50) },
    Column { header: "QOS", json_path: "qos", max_width: None },
    Column { header: "NL", json_path: "nl", max_width: None },
    Column { header: "RAP", json_path: "rap", max_width: None },
    Column { header: "RH", json_path: "rh", max_width: None },
];

pub async fn execute(client: &EmqxClient, fmt: &OutputFormatter, cmd: &SubscriptionCommand) -> Result<()> {
    match cmd {
        SubscriptionCommand::List { pagination, clientid, topic } => {
            let mut query: Vec<(&str, String)> = Vec::new();
            if let Some(ref v) = clientid { query.push(("clientid", v.clone())); }
            if let Some(ref v) = topic { query.push(("topic", v.clone())); }
            super::handle_paginated_list(client, fmt, "/subscriptions", &query, pagination, LIST_COLUMNS, None).await?;
        }
    }
    Ok(())
}
