use crate::client::EmqxClient;
use crate::output::{Column, OutputFormatter};
use anyhow::Result;
use clap::Subcommand;
use reqwest::Method;

#[derive(Subcommand)]
pub enum ActionCommand {
    List,
    Get {
        id: String,
    },
    Create {
        #[arg(short = 'f', long)]
        file: String,
    },
    Update {
        id: String,
        #[arg(short = 'f', long)]
        file: String,
    },
    Delete {
        id: String,
    },
    Metrics {
        id: String,
    },
    Start {
        id: String,
    },
    Stop {
        id: String,
    },
}

const LIST_COLUMNS: &[Column] = &[
    Column {
        header: "ID",
        json_path: "id",
        max_width: Some(30),
    },
    Column {
        header: "TYPE",
        json_path: "type",
        max_width: None,
    },
    Column {
        header: "NAME",
        json_path: "name",
        max_width: Some(20),
    },
    Column {
        header: "STATUS",
        json_path: "status",
        max_width: None,
    },
    Column {
        header: "CONNECTOR",
        json_path: "connector",
        max_width: Some(20),
    },
];

const DETAIL_COLUMNS: &[Column] = &[
    Column {
        header: "ID",
        json_path: "id",
        max_width: None,
    },
    Column {
        header: "TYPE",
        json_path: "type",
        max_width: None,
    },
    Column {
        header: "NAME",
        json_path: "name",
        max_width: None,
    },
    Column {
        header: "STATUS",
        json_path: "status",
        max_width: None,
    },
    Column {
        header: "CONNECTOR",
        json_path: "connector",
        max_width: None,
    },
    Column {
        header: "DESCRIPTION",
        json_path: "description",
        max_width: None,
    },
];

pub async fn execute(
    client: &EmqxClient,
    fmt: &OutputFormatter,
    cmd: &ActionCommand,
) -> Result<()> {
    match cmd {
        ActionCommand::List => {
            let value = client.get("/actions").await?;
            let items = super::extract_items(&value);
            fmt.print_list(&items, LIST_COLUMNS, None, None);
        }
        ActionCommand::Get { id } => {
            super::handle_get(client, fmt, &format!("/actions/{}", id), DETAIL_COLUMNS).await?;
        }
        ActionCommand::Create { file } => {
            super::handle_create_or_update(
                client,
                fmt,
                Method::POST,
                "/actions",
                file,
                "Action created",
            )
            .await?;
        }
        ActionCommand::Update { id, file } => {
            super::handle_create_or_update(
                client,
                fmt,
                Method::PUT,
                &format!("/actions/{}", id),
                file,
                "Action updated",
            )
            .await?;
        }
        ActionCommand::Delete { id } => {
            super::handle_delete(
                client,
                fmt,
                &format!("/actions/{}", id),
                &format!("Action '{}' deleted", id),
            )
            .await?;
        }
        ActionCommand::Metrics { id } => {
            let value = client.get(&format!("/actions/{}/metrics", id)).await?;
            fmt.print_value(&value);
        }
        ActionCommand::Start { id } => {
            client
                .post(&format!("/actions/{}/start", id), &serde_json::json!({}))
                .await?;
            fmt.print_success(&format!("Action '{}' started", id));
        }
        ActionCommand::Stop { id } => {
            client
                .post(&format!("/actions/{}/stop", id), &serde_json::json!({}))
                .await?;
            fmt.print_success(&format!("Action '{}' stopped", id));
        }
    }
    Ok(())
}
