use anyhow::Result;
use clap::Subcommand;
use crate::client::EmqxClient;
use crate::output::{Column, OutputFormatter};

#[derive(Subcommand)]
pub enum TraceCommand {
    List,
    Get { name: String },
    Create {
        #[arg(long)]
        name: String,
        #[arg(long, name = "type")]
        trace_type: String,
        #[arg(long)]
        target: String,
        #[arg(long)]
        start: Option<String>,
        #[arg(long)]
        end: Option<String>,
    },
    Delete { name: String },
    Stop { name: String },
    Log { name: String },
    Download {
        name: String,
        #[arg(short = 'o', long, default_value = ".")]
        output: String,
    },
    Clear,
}

const LIST_COLUMNS: &[Column] = &[
    Column { header: "NAME", json_path: "name", max_width: None },
    Column { header: "TYPE", json_path: "type", max_width: None },
    Column { header: "STATUS", json_path: "status", max_width: None },
    Column { header: "START", json_path: "start_at", max_width: None },
    Column { header: "END", json_path: "end_at", max_width: None },
];

const DETAIL_COLUMNS: &[Column] = &[
    Column { header: "NAME", json_path: "name", max_width: None },
    Column { header: "TYPE", json_path: "type", max_width: None },
    Column { header: "TARGET", json_path: "target", max_width: None },
    Column { header: "STATUS", json_path: "status", max_width: None },
    Column { header: "START", json_path: "start_at", max_width: None },
    Column { header: "END", json_path: "end_at", max_width: None },
    Column { header: "LOG SIZE", json_path: "log_size", max_width: None },
];

pub async fn execute(client: &EmqxClient, fmt: &OutputFormatter, cmd: &TraceCommand) -> Result<()> {
    match cmd {
        TraceCommand::List => {
            let value = client.get("/trace").await?;
            let items = super::extract_items(&value);
            fmt.print_list(&items, LIST_COLUMNS, None, None);
        }
        TraceCommand::Get { name } => {
            super::handle_get(client, fmt, &format!("/trace/{}", name), DETAIL_COLUMNS).await?;
        }
        TraceCommand::Create { name, trace_type, target, start, end } => {
            let mut body = serde_json::json!({
                "name": name,
                "type": trace_type,
                trace_type.as_str(): target,
            });
            if let Some(s) = start {
                body["start_at"] = serde_json::Value::String(s.clone());
            }
            if let Some(e) = end {
                body["end_at"] = serde_json::Value::String(e.clone());
            }
            let result = client.post("/trace", &body).await?;
            if result.is_null() {
                fmt.print_success(&format!("Trace '{}' created", name));
            } else {
                fmt.print_value(&result);
            }
        }
        TraceCommand::Delete { name } => {
            super::handle_delete(client, fmt, &format!("/trace/{}", name), &format!("Trace '{}' deleted", name)).await?;
        }
        TraceCommand::Stop { name } => {
            client.put(&format!("/trace/{}/stop", name), &serde_json::Value::Null).await?;
            fmt.print_success(&format!("Trace '{}' stopped", name));
        }
        TraceCommand::Log { name } => {
            let text = client.get_text(&format!("/trace/{}/log", name)).await?;
            println!("{}", text);
        }
        TraceCommand::Download { name, output } => {
            let dest = if output == "." {
                format!("{}.zip", name)
            } else {
                output.clone()
            };
            client.download(&format!("/trace/{}/download", name), &dest).await?;
            fmt.print_success(&format!("Trace '{}' downloaded to '{}'", name, dest));
        }
        TraceCommand::Clear => {
            client.delete("/trace").await?;
            fmt.print_success("All traces cleared");
        }
    }
    Ok(())
}
