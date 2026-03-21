use anyhow::Result;
use clap::Subcommand;
use reqwest::Method;
use crate::cli::PaginationArgs;
use crate::client::EmqxClient;
use crate::output::{Column, OutputFormatter};

#[derive(Subcommand)]
pub enum AuthnCommand {
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
    Reorder {
        #[arg(short = 'f', long)]
        file: String,
    },
    Users {
        id: String,
        #[command(subcommand)]
        command: AuthnUsersCommand,
    },
    Import {
        id: String,
        #[arg(short = 'f', long)]
        file: String,
    },
}

#[derive(Subcommand)]
pub enum AuthnUsersCommand {
    List {
        #[command(flatten)]
        pagination: PaginationArgs,
    },
    Get { userid: String },
    Create {
        #[arg(short = 'f', long)]
        file: String,
    },
    Update {
        userid: String,
        #[arg(short = 'f', long)]
        file: String,
    },
    Delete { userid: String },
}

const LIST_COLUMNS: &[Column] = &[
    Column { header: "ID", json_path: "id", max_width: None },
    Column { header: "MECHANISM", json_path: "mechanism", max_width: None },
    Column { header: "BACKEND", json_path: "backend", max_width: None },
    Column { header: "ENABLE", json_path: "enable", max_width: None },
];

const USER_COLUMNS: &[Column] = &[
    Column { header: "USER ID", json_path: "user_id", max_width: None },
    Column { header: "IS SUPERUSER", json_path: "is_superuser", max_width: None },
];

pub async fn execute(client: &EmqxClient, fmt: &OutputFormatter, cmd: &AuthnCommand) -> Result<()> {
    match cmd {
        AuthnCommand::List => {
            let value = client.get("/authentication").await?;
            let items = super::extract_items(&value);
            fmt.print_list(&items, LIST_COLUMNS, None, None);
        }
        AuthnCommand::Get { id } => {
            super::handle_get(client, fmt, &format!("/authentication/{}", id), LIST_COLUMNS).await?;
        }
        AuthnCommand::Create { file } => {
            super::handle_create_or_update(client, fmt, Method::POST, "/authentication", file, "Authenticator created").await?;
        }
        AuthnCommand::Update { id, file } => {
            super::handle_create_or_update(client, fmt, Method::PUT, &format!("/authentication/{}", id), file, "Authenticator updated").await?;
        }
        AuthnCommand::Delete { id } => {
            super::handle_delete(client, fmt, &format!("/authentication/{}", id), &format!("Authenticator '{}' deleted", id)).await?;
        }
        AuthnCommand::Reorder { file } => {
            super::handle_create_or_update(client, fmt, Method::POST, "/authentication/order", file, "Authentication order updated").await?;
        }
        AuthnCommand::Users { id, command } => {
            let base = format!("/authentication/{}/users", id);
            match command {
                AuthnUsersCommand::List { pagination } => {
                    super::handle_paginated_list(client, fmt, &base, &[], pagination, USER_COLUMNS, None).await?;
                }
                AuthnUsersCommand::Get { userid } => {
                    super::handle_get(client, fmt, &format!("{}/{}", base, userid), USER_COLUMNS).await?;
                }
                AuthnUsersCommand::Create { file } => {
                    super::handle_create_or_update(client, fmt, Method::POST, &base, file, "User created").await?;
                }
                AuthnUsersCommand::Update { userid, file } => {
                    super::handle_create_or_update(client, fmt, Method::PUT, &format!("{}/{}", base, userid), file, "User updated").await?;
                }
                AuthnUsersCommand::Delete { userid } => {
                    super::handle_delete(client, fmt, &format!("{}/{}", base, userid), &format!("User '{}' deleted", userid)).await?;
                }
            }
        }
        AuthnCommand::Import { id, file } => {
            client.upload(&format!("/authentication/{}/import_users", id), file).await?;
            fmt.print_success("Users imported");
        }
    }
    Ok(())
}
