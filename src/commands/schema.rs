use crate::client::EmqxClient;
use crate::output::{Column, OutputFormatter};
use anyhow::Result;
use clap::Subcommand;
use reqwest::Method;

#[derive(Subcommand)]
pub enum SchemaCommand {
    List,
    Get {
        name: String,
    },
    Create {
        #[arg(short = 'f', long)]
        file: String,
    },
    Update {
        name: String,
        #[arg(short = 'f', long)]
        file: String,
    },
    Delete {
        name: String,
    },
}

const LIST_COLUMNS: &[Column] = &[
    Column {
        header: "NAME",
        json_path: "name",
        max_width: None,
    },
    Column {
        header: "TYPE",
        json_path: "type",
        max_width: None,
    },
    Column {
        header: "DESCRIPTION",
        json_path: "description",
        max_width: Some(40),
    },
];

pub async fn execute(
    client: &EmqxClient,
    fmt: &OutputFormatter,
    cmd: &SchemaCommand,
) -> Result<()> {
    match cmd {
        SchemaCommand::List => {
            let value = client.get("/schemas").await?;
            let items = super::extract_items(&value);
            fmt.print_list(&items, LIST_COLUMNS, None, None);
        }
        SchemaCommand::Get { name } => {
            let value = client.get(&format!("/schemas/{}", name)).await?;
            fmt.print_value(&value);
        }
        SchemaCommand::Create { file } => {
            super::handle_create_or_update(
                client,
                fmt,
                Method::POST,
                "/schemas",
                file,
                "Schema created",
            )
            .await?;
        }
        SchemaCommand::Update { name, file } => {
            super::handle_create_or_update(
                client,
                fmt,
                Method::PUT,
                &format!("/schemas/{}", name),
                file,
                "Schema updated",
            )
            .await?;
        }
        SchemaCommand::Delete { name } => {
            super::handle_delete(
                client,
                fmt,
                &format!("/schemas/{}", name),
                &format!("Schema '{}' deleted", name),
            )
            .await?;
        }
    }
    Ok(())
}
