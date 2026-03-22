use crate::client::EmqxClient;
use crate::output::{Column, OutputFormatter};
use anyhow::Result;
use clap::Subcommand;
use reqwest::Method;

#[derive(Subcommand)]
pub enum ConnectorCommand {
    List {
        #[arg(long, name = "type")]
        connector_type: Option<String>,
    },
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
    Test {
        id: String,
    },
    Start {
        id: String,
    },
    Stop {
        id: String,
    },
    Restart {
        id: String,
    },
    Metrics {
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
        header: "DESCRIPTION",
        json_path: "description",
        max_width: None,
    },
];

pub async fn execute(
    client: &EmqxClient,
    fmt: &OutputFormatter,
    cmd: &ConnectorCommand,
) -> Result<()> {
    match cmd {
        ConnectorCommand::List { connector_type } => {
            let value = if let Some(t) = connector_type {
                client
                    .get_with_query("/connectors", &[("type", t.as_str())])
                    .await?
            } else {
                client.get("/connectors").await?
            };
            let items = super::extract_items(&value);
            fmt.print_list(&items, LIST_COLUMNS, None, None);
        }
        ConnectorCommand::Get { id } => {
            super::handle_get(client, fmt, &format!("/connectors/{}", id), DETAIL_COLUMNS).await?;
        }
        ConnectorCommand::Create { file } => {
            super::handle_create_or_update(
                client,
                fmt,
                Method::POST,
                "/connectors",
                file,
                "Connector created",
            )
            .await?;
        }
        ConnectorCommand::Update { id, file } => {
            super::handle_create_or_update(
                client,
                fmt,
                Method::PUT,
                &format!("/connectors/{}", id),
                file,
                "Connector updated",
            )
            .await?;
        }
        ConnectorCommand::Delete { id } => {
            super::handle_delete(
                client,
                fmt,
                &format!("/connectors/{}", id),
                &format!("Connector '{}' deleted", id),
            )
            .await?;
        }
        ConnectorCommand::Test { id } => {
            let result = client
                .post(&format!("/connectors/{}/test", id), &serde_json::json!({}))
                .await?;
            fmt.print_value(&result);
        }
        ConnectorCommand::Start { id } => {
            client
                .post(&format!("/connectors/{}/start", id), &serde_json::json!({}))
                .await?;
            fmt.print_success(&format!("Connector '{}' started", id));
        }
        ConnectorCommand::Stop { id } => {
            client
                .post(&format!("/connectors/{}/stop", id), &serde_json::json!({}))
                .await?;
            fmt.print_success(&format!("Connector '{}' stopped", id));
        }
        ConnectorCommand::Restart { id } => {
            client
                .post(
                    &format!("/connectors/{}/restart", id),
                    &serde_json::json!({}),
                )
                .await?;
            fmt.print_success(&format!("Connector '{}' restarted", id));
        }
        ConnectorCommand::Metrics { id } => {
            let value = client.get(&format!("/connectors/{}/metrics", id)).await?;
            fmt.print_value(&value);
        }
    }
    Ok(())
}
