use crate::client::EmqxClient;
use crate::output::{Column, OutputFormatter};
use anyhow::Result;
use clap::Subcommand;
use reqwest::Method;

#[derive(Subcommand)]
pub enum PluginCommand {
    List,
    Get {
        name: String,
    },
    Install {
        #[arg(short = 'f', long)]
        file: String,
    },
    Uninstall {
        name: String,
    },
    Start {
        name: String,
    },
    Stop {
        name: String,
    },
    Config {
        name: String,
        #[arg(short = 'f', long)]
        file: Option<String>,
    },
    Reorder {
        #[arg(short = 'f', long)]
        file: String,
    },
}

const LIST_COLUMNS: &[Column] = &[
    Column {
        header: "NAME",
        json_path: "name",
        max_width: Some(30),
    },
    Column {
        header: "VERSION",
        json_path: "rel_vsn",
        max_width: None,
    },
    Column {
        header: "STATUS",
        json_path: "running_status",
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
    cmd: &PluginCommand,
) -> Result<()> {
    match cmd {
        PluginCommand::List => {
            let value = client.get("/plugins").await?;
            let items = super::extract_items(&value);
            fmt.print_list(&items, LIST_COLUMNS, None, None);
        }
        PluginCommand::Get { name } => {
            let value = client.get(&format!("/plugins/{}", name)).await?;
            fmt.print_value(&value);
        }
        PluginCommand::Install { file } => {
            client.upload("/plugins/install", file).await?;
            fmt.print_success("Plugin installed");
        }
        PluginCommand::Uninstall { name } => {
            super::handle_delete(
                client,
                fmt,
                &format!("/plugins/{}", name),
                &format!("Plugin '{}' uninstalled", name),
            )
            .await?;
        }
        PluginCommand::Start { name } => {
            client
                .put(
                    &format!("/plugins/{}/start", name),
                    &serde_json::Value::Null,
                )
                .await?;
            fmt.print_success(&format!("Plugin '{}' started", name));
        }
        PluginCommand::Stop { name } => {
            client
                .put(&format!("/plugins/{}/stop", name), &serde_json::Value::Null)
                .await?;
            fmt.print_success(&format!("Plugin '{}' stopped", name));
        }
        PluginCommand::Config { name, file } => {
            if let Some(f) = file {
                super::handle_create_or_update(
                    client,
                    fmt,
                    Method::PUT,
                    &format!("/plugins/{}/config", name),
                    f,
                    &format!("Plugin '{}' config updated", name),
                )
                .await?;
            } else {
                let value = client.get(&format!("/plugins/{}/config", name)).await?;
                fmt.print_value(&value);
            }
        }
        PluginCommand::Reorder { file } => {
            super::handle_create_or_update(
                client,
                fmt,
                Method::POST,
                "/plugins/order",
                file,
                "Plugin order updated",
            )
            .await?;
        }
    }
    Ok(())
}
