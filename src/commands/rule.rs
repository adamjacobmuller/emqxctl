use crate::client::EmqxClient;
use crate::output::{Column, OutputFormatter};
use anyhow::Result;
use clap::Subcommand;
use reqwest::Method;

#[derive(Subcommand)]
pub enum RuleCommand {
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
    Test {
        id: String,
        #[arg(short = 'f', long)]
        file: String,
    },
    Metrics {
        id: String,
    },
    #[command(name = "reset-metrics")]
    ResetMetrics {
        id: String,
    },
}

const LIST_COLUMNS: &[Column] = &[
    Column {
        header: "ID",
        json_path: "id",
        max_width: Some(20),
    },
    Column {
        header: "NAME",
        json_path: "name",
        max_width: Some(20),
    },
    Column {
        header: "ENABLED",
        json_path: "enable",
        max_width: None,
    },
    Column {
        header: "SQL",
        json_path: "sql",
        max_width: Some(50),
    },
    Column {
        header: "ACTIONS",
        json_path: "actions",
        max_width: Some(30),
    },
];

const DETAIL_COLUMNS: &[Column] = &[
    Column {
        header: "ID",
        json_path: "id",
        max_width: None,
    },
    Column {
        header: "NAME",
        json_path: "name",
        max_width: None,
    },
    Column {
        header: "ENABLED",
        json_path: "enable",
        max_width: None,
    },
    Column {
        header: "SQL",
        json_path: "sql",
        max_width: None,
    },
    Column {
        header: "ACTIONS",
        json_path: "actions",
        max_width: None,
    },
    Column {
        header: "DESCRIPTION",
        json_path: "description",
        max_width: None,
    },
    Column {
        header: "CREATED AT",
        json_path: "created_at",
        max_width: None,
    },
];

pub async fn execute(client: &EmqxClient, fmt: &OutputFormatter, cmd: &RuleCommand) -> Result<()> {
    match cmd {
        RuleCommand::List => {
            let value = client.get("/rules").await?;
            let items = super::extract_items(&value);
            fmt.print_list(&items, LIST_COLUMNS, None, None);
        }
        RuleCommand::Get { id } => {
            super::handle_get(client, fmt, &format!("/rules/{}", id), DETAIL_COLUMNS).await?;
        }
        RuleCommand::Create { file } => {
            super::handle_create_or_update(
                client,
                fmt,
                Method::POST,
                "/rules",
                file,
                "Rule created",
            )
            .await?;
        }
        RuleCommand::Update { id, file } => {
            super::handle_create_or_update(
                client,
                fmt,
                Method::PUT,
                &format!("/rules/{}", id),
                file,
                "Rule updated",
            )
            .await?;
        }
        RuleCommand::Delete { id } => {
            super::handle_delete(
                client,
                fmt,
                &format!("/rules/{}", id),
                &format!("Rule '{}' deleted", id),
            )
            .await?;
        }
        RuleCommand::Test { id, file } => {
            let body = crate::input::read_input_file(file)?;
            let result = client.post(&format!("/rules/{}/test", id), &body).await?;
            fmt.print_value(&result);
        }
        RuleCommand::Metrics { id } => {
            let value = client.get(&format!("/rules/{}/metrics", id)).await?;
            fmt.print_value(&value);
        }
        RuleCommand::ResetMetrics { id } => {
            client
                .put(
                    &format!("/rules/{}/metrics/reset", id),
                    &serde_json::Value::Null,
                )
                .await?;
            fmt.print_success(&format!("Metrics for rule '{}' reset", id));
        }
    }
    Ok(())
}
