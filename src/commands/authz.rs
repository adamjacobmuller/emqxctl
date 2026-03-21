use anyhow::Result;
use clap::Subcommand;
use reqwest::Method;
use crate::client::EmqxClient;
use crate::output::{Column, OutputFormatter};

#[derive(Subcommand)]
pub enum AuthzCommand {
    List,
    Get {
        #[arg(name = "type")]
        authz_type: String,
    },
    Create {
        #[arg(short = 'f', long)]
        file: String,
    },
    Update {
        #[arg(name = "type")]
        authz_type: String,
        #[arg(short = 'f', long)]
        file: String,
    },
    Delete {
        #[arg(name = "type")]
        authz_type: String,
    },
    Reorder {
        #[arg(short = 'f', long)]
        file: String,
    },
    #[command(name = "cache-clean")]
    CacheClean {
        #[arg(long)]
        clientid: Option<String>,
    },
}

const LIST_COLUMNS: &[Column] = &[
    Column { header: "TYPE", json_path: "type", max_width: None },
    Column { header: "ENABLE", json_path: "enable", max_width: None },
];

pub async fn execute(client: &EmqxClient, fmt: &OutputFormatter, cmd: &AuthzCommand) -> Result<()> {
    match cmd {
        AuthzCommand::List => {
            let value = client.get("/authorization/sources").await?;
            let items = super::extract_items(&value);
            fmt.print_list(&items, LIST_COLUMNS, None, None);
        }
        AuthzCommand::Get { authz_type } => {
            super::handle_get(client, fmt, &format!("/authorization/sources/{}", authz_type), LIST_COLUMNS).await?;
        }
        AuthzCommand::Create { file } => {
            super::handle_create_or_update(client, fmt, Method::POST, "/authorization/sources", file, "Authorization source created").await?;
        }
        AuthzCommand::Update { authz_type, file } => {
            super::handle_create_or_update(client, fmt, Method::PUT, &format!("/authorization/sources/{}", authz_type), file, "Authorization source updated").await?;
        }
        AuthzCommand::Delete { authz_type } => {
            super::handle_delete(client, fmt, &format!("/authorization/sources/{}", authz_type), &format!("Authorization source '{}' deleted", authz_type)).await?;
        }
        AuthzCommand::Reorder { file } => {
            super::handle_create_or_update(client, fmt, Method::POST, "/authorization/sources/order", file, "Authorization order updated").await?;
        }
        AuthzCommand::CacheClean { clientid } => {
            if let Some(cid) = clientid {
                client.delete(&format!("/authorization/cache/{}", cid)).await?;
                fmt.print_success(&format!("Authorization cache cleaned for client '{}'", cid));
            } else {
                client.delete("/authorization/cache").await?;
                fmt.print_success("Authorization cache cleaned");
            }
        }
    }
    Ok(())
}
