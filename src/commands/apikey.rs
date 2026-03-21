use anyhow::Result;
use clap::Subcommand;
use reqwest::Method;
use crate::client::EmqxClient;
use crate::output::{Column, OutputFormatter};

#[derive(Subcommand)]
pub enum ApikeyCommand {
    List,
    Get { name: String },
    Create {
        #[arg(long)]
        name: String,
        #[arg(long)]
        expired_at: Option<String>,
        #[arg(long)]
        role: Option<String>,
    },
    Update {
        name: String,
        #[arg(short = 'f', long)]
        file: String,
    },
    Delete { name: String },
}

const LIST_COLUMNS: &[Column] = &[
    Column { header: "NAME", json_path: "name", max_width: None },
    Column { header: "API KEY", json_path: "api_key", max_width: Some(20) },
    Column { header: "ROLE", json_path: "role", max_width: None },
    Column { header: "EXPIRED AT", json_path: "expired_at", max_width: None },
    Column { header: "CREATED AT", json_path: "created_at", max_width: None },
];

pub async fn execute(client: &EmqxClient, fmt: &OutputFormatter, cmd: &ApikeyCommand) -> Result<()> {
    match cmd {
        ApikeyCommand::List => {
            let value = client.get("/api_key").await?;
            let items = super::extract_items(&value);
            fmt.print_list(&items, LIST_COLUMNS, None, None);
        }
        ApikeyCommand::Get { name } => {
            super::handle_get(client, fmt, &format!("/api_key/{}", name), LIST_COLUMNS).await?;
        }
        ApikeyCommand::Create { name, expired_at, role } => {
            let mut body = serde_json::json!({ "name": name });
            if let Some(e) = expired_at {
                body["expired_at"] = serde_json::Value::String(e.clone());
            }
            if let Some(r) = role {
                body["role"] = serde_json::Value::String(r.clone());
            }
            let result = client.post("/api_key", &body).await?;
            fmt.print_value(&result);
        }
        ApikeyCommand::Update { name, file } => {
            super::handle_create_or_update(client, fmt, Method::PUT, &format!("/api_key/{}", name), file, &format!("API key '{}' updated", name)).await?;
        }
        ApikeyCommand::Delete { name } => {
            super::handle_delete(client, fmt, &format!("/api_key/{}", name), &format!("API key '{}' deleted", name)).await?;
        }
    }
    Ok(())
}
