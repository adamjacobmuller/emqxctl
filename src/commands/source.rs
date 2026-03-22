use crate::client::EmqxClient;
use crate::output::{Column, OutputFormatter};
use anyhow::Result;
use clap::Subcommand;
use reqwest::Method;

#[derive(Subcommand)]
pub enum SourceCommand {
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
    cmd: &SourceCommand,
) -> Result<()> {
    match cmd {
        SourceCommand::List => {
            let value = client.get("/sources").await?;
            let items = super::extract_items(&value);
            fmt.print_list(&items, LIST_COLUMNS, None, None);
        }
        SourceCommand::Get { id } => {
            super::handle_get(client, fmt, &format!("/sources/{}", id), DETAIL_COLUMNS).await?;
        }
        SourceCommand::Create { file } => {
            super::handle_create_or_update(
                client,
                fmt,
                Method::POST,
                "/sources",
                file,
                "Source created",
            )
            .await?;
        }
        SourceCommand::Update { id, file } => {
            super::handle_create_or_update(
                client,
                fmt,
                Method::PUT,
                &format!("/sources/{}", id),
                file,
                "Source updated",
            )
            .await?;
        }
        SourceCommand::Delete { id } => {
            super::handle_delete(
                client,
                fmt,
                &format!("/sources/{}", id),
                &format!("Source '{}' deleted", id),
            )
            .await?;
        }
        SourceCommand::Metrics { id } => {
            let value = client.get(&format!("/sources/{}/metrics", id)).await?;
            fmt.print_value(&value);
        }
    }
    Ok(())
}
