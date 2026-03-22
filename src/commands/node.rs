use crate::client::EmqxClient;
use crate::output::{Column, OutputFormatter};
use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum NodeCommand {
    /// List all nodes
    List,
    /// Get node details
    Get { node: String },
    /// Get node metrics
    Metrics { node: String },
    /// Get node stats
    Stats { node: String },
}

const LIST_COLUMNS: &[Column] = &[
    Column {
        header: "NODE",
        json_path: "node",
        max_width: None,
    },
    Column {
        header: "STATUS",
        json_path: "node_status",
        max_width: None,
    },
    Column {
        header: "VERSION",
        json_path: "version",
        max_width: None,
    },
    Column {
        header: "UPTIME",
        json_path: "uptime",
        max_width: None,
    },
    Column {
        header: "CONNECTIONS",
        json_path: "connections",
        max_width: None,
    },
    Column {
        header: "LOAD1",
        json_path: "load1",
        max_width: None,
    },
];

const DETAIL_COLUMNS: &[Column] = &[
    Column {
        header: "NODE",
        json_path: "node",
        max_width: None,
    },
    Column {
        header: "STATUS",
        json_path: "node_status",
        max_width: None,
    },
    Column {
        header: "VERSION",
        json_path: "version",
        max_width: None,
    },
    Column {
        header: "UPTIME",
        json_path: "uptime",
        max_width: None,
    },
    Column {
        header: "CONNECTIONS",
        json_path: "connections",
        max_width: None,
    },
    Column {
        header: "LOAD1",
        json_path: "load1",
        max_width: None,
    },
    Column {
        header: "LOAD5",
        json_path: "load5",
        max_width: None,
    },
    Column {
        header: "LOAD15",
        json_path: "load15",
        max_width: None,
    },
    Column {
        header: "MAX FDS",
        json_path: "max_fds",
        max_width: None,
    },
    Column {
        header: "MEMORY TOTAL",
        json_path: "memory_total",
        max_width: None,
    },
    Column {
        header: "MEMORY USED",
        json_path: "memory_used",
        max_width: None,
    },
    Column {
        header: "OTP RELEASE",
        json_path: "otp_release",
        max_width: None,
    },
    Column {
        header: "EDITION",
        json_path: "edition",
        max_width: None,
    },
];

pub async fn execute(client: &EmqxClient, fmt: &OutputFormatter, cmd: &NodeCommand) -> Result<()> {
    match cmd {
        NodeCommand::List => {
            let value = client.get("/nodes").await?;
            let items = super::extract_items(&value);
            fmt.print_list(&items, LIST_COLUMNS, None, None);
        }
        NodeCommand::Get { node } => {
            let value = client.get(&format!("/nodes/{}", node)).await?;
            fmt.print_item(&value, DETAIL_COLUMNS);
        }
        NodeCommand::Metrics { node } => {
            let value = client.get(&format!("/nodes/{}/metrics", node)).await?;
            fmt.print_value(&value);
        }
        NodeCommand::Stats { node } => {
            let value = client.get(&format!("/nodes/{}/stats", node)).await?;
            fmt.print_value(&value);
        }
    }
    Ok(())
}
