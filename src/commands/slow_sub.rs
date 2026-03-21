use anyhow::Result;
use clap::Subcommand;
use reqwest::Method;
use crate::client::EmqxClient;
use crate::output::{Column, OutputFormatter};

#[derive(Subcommand)]
pub enum SlowSubCommand {
    Config {
        #[arg(short = 'f', long)]
        file: Option<String>,
    },
    List,
    Clear,
}

const LIST_COLUMNS: &[Column] = &[
    Column { header: "CLIENTID", json_path: "clientid", max_width: Some(30) },
    Column { header: "TOPIC", json_path: "topic", max_width: Some(40) },
    Column { header: "TIMESPAN", json_path: "timespan", max_width: None },
    Column { header: "NODE", json_path: "node", max_width: None },
];

pub async fn execute(client: &EmqxClient, fmt: &OutputFormatter, cmd: &SlowSubCommand) -> Result<()> {
    match cmd {
        SlowSubCommand::Config { file } => {
            if let Some(f) = file {
                super::handle_create_or_update(client, fmt, Method::PUT, "/slow_subscriptions/settings", f, "Slow subscriptions config updated").await?;
            } else {
                let value = client.get("/slow_subscriptions/settings").await?;
                fmt.print_value(&value);
            }
        }
        SlowSubCommand::List => {
            let value = client.get("/slow_subscriptions").await?;
            let items = super::extract_items(&value);
            fmt.print_list(&items, LIST_COLUMNS, None, None);
        }
        SlowSubCommand::Clear => {
            client.delete("/slow_subscriptions").await?;
            fmt.print_success("Slow subscription records cleared");
        }
    }
    Ok(())
}
