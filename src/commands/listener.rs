use anyhow::Result;
use clap::Subcommand;
use reqwest::Method;
use crate::client::EmqxClient;
use crate::output::{Column, OutputFormatter};

#[derive(Subcommand)]
pub enum ListenerCommand {
    List,
    Get { id: String },
    Create {
        #[arg(short = 'f', long)]
        file: String,
    },
    Update {
        id: String,
        #[arg(short = 'f', long)]
        file: String,
    },
    Delete { id: String },
    Start { id: String },
    Stop { id: String },
    Restart { id: String },
}

const LIST_COLUMNS: &[Column] = &[
    Column { header: "ID", json_path: "id", max_width: None },
    Column { header: "TYPE", json_path: "type", max_width: None },
    Column { header: "NAME", json_path: "name", max_width: None },
    Column { header: "BIND", json_path: "bind", max_width: None },
    Column { header: "RUNNING", json_path: "running", max_width: None },
    Column { header: "ACCEPTORS", json_path: "acceptors", max_width: None },
];

const DETAIL_COLUMNS: &[Column] = &[
    Column { header: "ID", json_path: "id", max_width: None },
    Column { header: "TYPE", json_path: "type", max_width: None },
    Column { header: "NAME", json_path: "name", max_width: None },
    Column { header: "BIND", json_path: "bind", max_width: None },
    Column { header: "RUNNING", json_path: "running", max_width: None },
    Column { header: "ACCEPTORS", json_path: "acceptors", max_width: None },
    Column { header: "MAX CONNECTIONS", json_path: "max_connections", max_width: None },
    Column { header: "CURRENT CONNECTIONS", json_path: "current_connections", max_width: None },
];

pub async fn execute(client: &EmqxClient, fmt: &OutputFormatter, cmd: &ListenerCommand) -> Result<()> {
    match cmd {
        ListenerCommand::List => {
            let value = client.get("/listeners").await?;
            let items = super::extract_items(&value);
            fmt.print_list(&items, LIST_COLUMNS, None, None);
        }
        ListenerCommand::Get { id } => {
            super::handle_get(client, fmt, &format!("/listeners/{}", id), DETAIL_COLUMNS).await?;
        }
        ListenerCommand::Create { file } => {
            super::handle_create_or_update(client, fmt, Method::POST, "/listeners", file, "Listener created").await?;
        }
        ListenerCommand::Update { id, file } => {
            super::handle_create_or_update(client, fmt, Method::PUT, &format!("/listeners/{}", id), file, "Listener updated").await?;
        }
        ListenerCommand::Delete { id } => {
            super::handle_delete(client, fmt, &format!("/listeners/{}", id), &format!("Listener '{}' deleted", id)).await?;
        }
        ListenerCommand::Start { id } => {
            client.post(&format!("/listeners/{}/start", id), &serde_json::json!({})).await?;
            fmt.print_success(&format!("Listener '{}' started", id));
        }
        ListenerCommand::Stop { id } => {
            client.post(&format!("/listeners/{}/stop", id), &serde_json::json!({})).await?;
            fmt.print_success(&format!("Listener '{}' stopped", id));
        }
        ListenerCommand::Restart { id } => {
            client.post(&format!("/listeners/{}/restart", id), &serde_json::json!({})).await?;
            fmt.print_success(&format!("Listener '{}' restarted", id));
        }
    }
    Ok(())
}
