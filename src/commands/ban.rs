use anyhow::Result;
use clap::Subcommand;
use crate::cli::PaginationArgs;
use crate::client::EmqxClient;
use crate::output::{Column, OutputFormatter};

#[derive(Subcommand)]
pub enum BanCommand {
    List {
        #[command(flatten)]
        pagination: PaginationArgs,
    },
    Create {
        #[arg(long)]
        who: String,
        #[arg(long, name = "as")]
        as_type: String,
        #[arg(long)]
        reason: Option<String>,
        #[arg(long)]
        until: Option<String>,
    },
    Delete {
        /// Format: <as>/<who>
        target: String,
    },
    Clear,
}

const LIST_COLUMNS: &[Column] = &[
    Column { header: "WHO", json_path: "who", max_width: None },
    Column { header: "AS", json_path: "as", max_width: None },
    Column { header: "REASON", json_path: "reason", max_width: Some(30) },
    Column { header: "AT", json_path: "at", max_width: None },
    Column { header: "UNTIL", json_path: "until", max_width: None },
];

pub async fn execute(client: &EmqxClient, fmt: &OutputFormatter, cmd: &BanCommand) -> Result<()> {
    match cmd {
        BanCommand::List { pagination } => {
            super::handle_paginated_list(client, fmt, "/banned", &[], pagination, LIST_COLUMNS, None).await?;
        }
        BanCommand::Create { who, as_type, reason, until } => {
            let mut body = serde_json::json!({
                "who": who,
                "as": as_type,
            });
            if let Some(r) = reason {
                body["reason"] = serde_json::Value::String(r.clone());
            }
            if let Some(u) = until {
                body["until"] = serde_json::Value::String(u.clone());
            }
            let result = client.post("/banned", &body).await?;
            if result.is_null() {
                fmt.print_success(&format!("Banned {} '{}'", as_type, who));
            } else {
                fmt.print_value(&result);
            }
        }
        BanCommand::Delete { target } => {
            let parts: Vec<&str> = target.splitn(2, '/').collect();
            if parts.len() != 2 {
                anyhow::bail!("Target must be in format <as>/<who>, e.g. clientid/myclient");
            }
            super::handle_delete(client, fmt, &format!("/banned/{}/{}", parts[0], parts[1]), &format!("Ban on '{}' removed", target)).await?;
        }
        BanCommand::Clear => {
            client.delete("/banned").await?;
            fmt.print_success("All bans cleared");
        }
    }
    Ok(())
}
