use crate::client::EmqxClient;
use crate::output::{Column, OutputFormatter};
use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum CertCommand {
    List,
    Get { id: String },
}

const LIST_COLUMNS: &[Column] = &[
    Column {
        header: "ID",
        json_path: "id",
        max_width: None,
    },
    Column {
        header: "COMMON NAME",
        json_path: "common_name",
        max_width: None,
    },
    Column {
        header: "NOT BEFORE",
        json_path: "not_before",
        max_width: None,
    },
    Column {
        header: "NOT AFTER",
        json_path: "not_after",
        max_width: None,
    },
];

pub async fn execute(client: &EmqxClient, fmt: &OutputFormatter, cmd: &CertCommand) -> Result<()> {
    match cmd {
        CertCommand::List => {
            let value = client.get("/ssl_certs").await?;
            let items = super::extract_items(&value);
            fmt.print_list(&items, LIST_COLUMNS, None, None);
        }
        CertCommand::Get { id } => {
            let value = client.get(&format!("/ssl_certs/{}", id)).await?;
            fmt.print_value(&value);
        }
    }
    Ok(())
}
