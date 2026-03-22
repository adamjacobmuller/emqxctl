use crate::client::EmqxClient;
use crate::output::{Column, OutputFormatter};
use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum BackupCommand {
    List,
    Create,
    Download {
        name: String,
        #[arg(short = 'o', long, default_value = ".")]
        output: String,
    },
    Upload {
        #[arg(short = 'f', long)]
        file: String,
    },
    Import {
        #[arg(short = 'f', long)]
        file: String,
    },
    Delete {
        name: String,
    },
}

const LIST_COLUMNS: &[Column] = &[
    Column {
        header: "FILENAME",
        json_path: "filename",
        max_width: None,
    },
    Column {
        header: "SIZE",
        json_path: "size",
        max_width: None,
    },
    Column {
        header: "CREATED AT",
        json_path: "created_at",
        max_width: None,
    },
    Column {
        header: "NODE",
        json_path: "node",
        max_width: None,
    },
];

pub async fn execute(
    client: &EmqxClient,
    fmt: &OutputFormatter,
    cmd: &BackupCommand,
) -> Result<()> {
    match cmd {
        BackupCommand::List => {
            let value = client.get("/data/export").await?;
            let items = super::extract_items(&value);
            fmt.print_list(&items, LIST_COLUMNS, None, None);
        }
        BackupCommand::Create => {
            let result = client.post("/data/export", &serde_json::json!({})).await?;
            if result.is_null() {
                fmt.print_success("Backup created");
            } else {
                fmt.print_value(&result);
            }
        }
        BackupCommand::Download { name, output } => {
            let dest = if output == "." {
                name.clone()
            } else {
                output.clone()
            };
            client
                .download(&format!("/data/export/{}", name), &dest)
                .await?;
            fmt.print_success(&format!("Backup '{}' downloaded to '{}'", name, dest));
        }
        BackupCommand::Upload { file } => {
            client.upload("/data/files", file).await?;
            fmt.print_success("Backup uploaded");
        }
        BackupCommand::Import { file } => {
            let body = crate::input::read_input_file(file)?;
            client.post("/data/import", &body).await?;
            fmt.print_success("Backup imported");
        }
        BackupCommand::Delete { name } => {
            super::handle_delete(
                client,
                fmt,
                &format!("/data/export/{}", name),
                &format!("Backup '{}' deleted", name),
            )
            .await?;
        }
    }
    Ok(())
}
