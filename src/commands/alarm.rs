use crate::cli::PaginationArgs;
use crate::client::EmqxClient;
use crate::output::{Column, OutputFormatter};
use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum AlarmCommand {
    List {
        #[command(flatten)]
        pagination: PaginationArgs,
        #[arg(long)]
        activated: bool,
        #[arg(long)]
        deactivated: bool,
    },
    Clear,
}

const LIST_COLUMNS: &[Column] = &[
    Column {
        header: "NAME",
        json_path: "name",
        max_width: None,
    },
    Column {
        header: "NODE",
        json_path: "node",
        max_width: None,
    },
    Column {
        header: "MESSAGE",
        json_path: "message",
        max_width: Some(40),
    },
    Column {
        header: "ACTIVATE AT",
        json_path: "activate_at",
        max_width: None,
    },
    Column {
        header: "DEACTIVATE AT",
        json_path: "deactivate_at",
        max_width: None,
    },
];

pub async fn execute(client: &EmqxClient, fmt: &OutputFormatter, cmd: &AlarmCommand) -> Result<()> {
    match cmd {
        AlarmCommand::List {
            pagination,
            activated,
            deactivated,
        } => {
            let mut query: Vec<(&str, String)> = Vec::new();
            if *activated {
                query.push(("activated", "true".to_string()));
            } else if *deactivated {
                query.push(("activated", "false".to_string()));
            }
            super::handle_paginated_list(
                client,
                fmt,
                "/alarms",
                &query,
                pagination,
                LIST_COLUMNS,
                None,
            )
            .await?;
        }
        AlarmCommand::Clear => {
            client.delete("/alarms").await?;
            fmt.print_success("All alarms cleared");
        }
    }
    Ok(())
}
