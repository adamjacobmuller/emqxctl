use crate::cli::PaginationArgs;
use crate::client::EmqxClient;
use crate::output::{Column, OutputFormatter};
use anyhow::Result;
use clap::Subcommand;
use reqwest::Method;

#[derive(Subcommand)]
pub enum GatewayCommand {
    /// List gateways
    List,
    /// Get gateway details
    Get { name: String },
    /// Enable a gateway
    Enable { name: String },
    /// Disable a gateway
    Disable { name: String },
    /// Update gateway configuration from file
    Update {
        name: String,
        #[arg(short = 'f', long)]
        file: String,
    },
    /// Manage gateway clients
    Clients {
        name: String,
        #[command(subcommand)]
        command: GatewayClientCommand,
    },
    /// Manage gateway authentication
    Authn {
        name: String,
        #[command(subcommand)]
        command: GatewayAuthnCommand,
    },
    /// Manage gateway listeners
    Listeners {
        name: String,
        #[command(subcommand)]
        command: GatewayListenerCommand,
    },
}

#[derive(Subcommand)]
pub enum GatewayClientCommand {
    List {
        #[command(flatten)]
        pagination: PaginationArgs,
    },
    Get {
        clientid: String,
    },
    Kick {
        clientid: String,
    },
}

#[derive(Subcommand)]
pub enum GatewayAuthnCommand {
    List,
    Create {
        #[arg(short = 'f', long)]
        file: String,
    },
    Update {
        #[arg(short = 'f', long)]
        file: String,
    },
    Delete,
    Users {
        #[command(subcommand)]
        command: GatewayAuthnUsersCommand,
    },
}

#[derive(Subcommand)]
pub enum GatewayAuthnUsersCommand {
    List {
        #[command(flatten)]
        pagination: PaginationArgs,
    },
    Create {
        #[arg(short = 'f', long)]
        file: String,
    },
}

#[derive(Subcommand)]
pub enum GatewayListenerCommand {
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
}

const GW_LIST_COLUMNS: &[Column] = &[
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
        header: "ENABLE",
        json_path: "enable",
        max_width: None,
    },
    Column {
        header: "CURRENT CONNECTIONS",
        json_path: "current_connections",
        max_width: None,
    },
];

const CLIENT_COLUMNS: &[Column] = &[
    Column {
        header: "CLIENTID",
        json_path: "clientid",
        max_width: Some(30),
    },
    Column {
        header: "USERNAME",
        json_path: "username",
        max_width: Some(20),
    },
    Column {
        header: "IP ADDRESS",
        json_path: "ip_address",
        max_width: None,
    },
    Column {
        header: "CONNECTED",
        json_path: "connected",
        max_width: None,
    },
];

const LISTENER_COLUMNS: &[Column] = &[
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
        header: "BIND",
        json_path: "bind",
        max_width: None,
    },
    Column {
        header: "RUNNING",
        json_path: "running",
        max_width: None,
    },
];

const USER_COLUMNS: &[Column] = &[
    Column {
        header: "USER ID",
        json_path: "user_id",
        max_width: None,
    },
    Column {
        header: "IS SUPERUSER",
        json_path: "is_superuser",
        max_width: None,
    },
];

pub async fn execute(
    client: &EmqxClient,
    fmt: &OutputFormatter,
    cmd: &GatewayCommand,
) -> Result<()> {
    match cmd {
        GatewayCommand::List => {
            let value = client.get("/gateways").await?;
            let items = super::extract_items(&value);
            fmt.print_list(&items, GW_LIST_COLUMNS, None, None);
        }
        GatewayCommand::Get { name } => {
            let value = client.get(&format!("/gateways/{}", name)).await?;
            fmt.print_value(&value);
        }
        GatewayCommand::Enable { name } => {
            client
                .put(
                    &format!("/gateways/{}", name),
                    &serde_json::json!({"enable": true}),
                )
                .await?;
            fmt.print_success(&format!("Gateway '{}' enabled", name));
        }
        GatewayCommand::Disable { name } => {
            client
                .put(
                    &format!("/gateways/{}", name),
                    &serde_json::json!({"enable": false}),
                )
                .await?;
            fmt.print_success(&format!("Gateway '{}' disabled", name));
        }
        GatewayCommand::Update { name, file } => {
            super::handle_create_or_update(
                client,
                fmt,
                Method::PUT,
                &format!("/gateways/{}", name),
                file,
                &format!("Gateway '{}' updated", name),
            )
            .await?;
        }
        GatewayCommand::Clients { name, command } => {
            let base = format!("/gateways/{}/clients", name);
            match command {
                GatewayClientCommand::List { pagination } => {
                    super::handle_paginated_list(
                        client,
                        fmt,
                        &base,
                        &[],
                        pagination,
                        CLIENT_COLUMNS,
                        None,
                    )
                    .await?;
                }
                GatewayClientCommand::Get { clientid } => {
                    let value = client.get(&format!("{}/{}", base, clientid)).await?;
                    fmt.print_value(&value);
                }
                GatewayClientCommand::Kick { clientid } => {
                    client.delete(&format!("{}/{}", base, clientid)).await?;
                    fmt.print_success(&format!("Gateway client '{}' kicked", clientid));
                }
            }
        }
        GatewayCommand::Authn { name, command } => {
            let base = format!("/gateways/{}/authentication", name);
            match command {
                GatewayAuthnCommand::List => {
                    let value = client.get(&base).await?;
                    fmt.print_value(&value);
                }
                GatewayAuthnCommand::Create { file } => {
                    super::handle_create_or_update(
                        client,
                        fmt,
                        Method::POST,
                        &base,
                        file,
                        "Gateway authenticator created",
                    )
                    .await?;
                }
                GatewayAuthnCommand::Update { file } => {
                    super::handle_create_or_update(
                        client,
                        fmt,
                        Method::PUT,
                        &base,
                        file,
                        "Gateway authenticator updated",
                    )
                    .await?;
                }
                GatewayAuthnCommand::Delete => {
                    super::handle_delete(client, fmt, &base, "Gateway authenticator deleted")
                        .await?;
                }
                GatewayAuthnCommand::Users { command: users_cmd } => {
                    let users_base = format!("{}/users", base);
                    match users_cmd {
                        GatewayAuthnUsersCommand::List { pagination } => {
                            super::handle_paginated_list(
                                client,
                                fmt,
                                &users_base,
                                &[],
                                pagination,
                                USER_COLUMNS,
                                None,
                            )
                            .await?;
                        }
                        GatewayAuthnUsersCommand::Create { file } => {
                            super::handle_create_or_update(
                                client,
                                fmt,
                                Method::POST,
                                &users_base,
                                file,
                                "Gateway auth user created",
                            )
                            .await?;
                        }
                    }
                }
            }
        }
        GatewayCommand::Listeners { name, command } => {
            let base = format!("/gateways/{}/listeners", name);
            match command {
                GatewayListenerCommand::List => {
                    let value = client.get(&base).await?;
                    let items = super::extract_items(&value);
                    fmt.print_list(&items, LISTENER_COLUMNS, None, None);
                }
                GatewayListenerCommand::Get { id } => {
                    let value = client.get(&format!("{}/{}", base, id)).await?;
                    fmt.print_value(&value);
                }
                GatewayListenerCommand::Create { file } => {
                    super::handle_create_or_update(
                        client,
                        fmt,
                        Method::POST,
                        &base,
                        file,
                        "Gateway listener created",
                    )
                    .await?;
                }
                GatewayListenerCommand::Update { id, file } => {
                    super::handle_create_or_update(
                        client,
                        fmt,
                        Method::PUT,
                        &format!("{}/{}", base, id),
                        file,
                        "Gateway listener updated",
                    )
                    .await?;
                }
                GatewayListenerCommand::Delete { id } => {
                    super::handle_delete(
                        client,
                        fmt,
                        &format!("{}/{}", base, id),
                        &format!("Gateway listener '{}' deleted", id),
                    )
                    .await?;
                }
            }
        }
    }
    Ok(())
}
