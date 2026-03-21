use anyhow::Result;
use clap::Subcommand;
use crate::client::EmqxClient;
use crate::output::{Column, OutputFormatter};

#[derive(Subcommand)]
pub enum AdminCommand {
    List,
    Get { username: String },
    Create {
        #[arg(long)]
        username: String,
        #[arg(long)]
        password: String,
        #[arg(long)]
        role: Option<String>,
    },
    Update {
        username: String,
        #[arg(long)]
        role: String,
    },
    Delete { username: String },
    #[command(name = "change-password")]
    ChangePassword {
        username: String,
        #[arg(long, name = "old")]
        old_password: String,
        #[arg(long, name = "new")]
        new_password: String,
    },
}

const LIST_COLUMNS: &[Column] = &[
    Column { header: "USERNAME", json_path: "username", max_width: None },
    Column { header: "ROLE", json_path: "role", max_width: None },
    Column { header: "DESCRIPTION", json_path: "description", max_width: Some(40) },
];

pub async fn execute(client: &EmqxClient, fmt: &OutputFormatter, cmd: &AdminCommand) -> Result<()> {
    match cmd {
        AdminCommand::List => {
            let value = client.get("/users").await?;
            let items = super::extract_items(&value);
            fmt.print_list(&items, LIST_COLUMNS, None, None);
        }
        AdminCommand::Get { username } => {
            super::handle_get(client, fmt, &format!("/users/{}", username), LIST_COLUMNS).await?;
        }
        AdminCommand::Create { username, password, role } => {
            let mut body = serde_json::json!({
                "username": username,
                "password": password,
            });
            if let Some(r) = role {
                body["role"] = serde_json::Value::String(r.clone());
            }
            let result = client.post("/users", &body).await?;
            if result.is_null() {
                fmt.print_success(&format!("User '{}' created", username));
            } else {
                fmt.print_value(&result);
            }
        }
        AdminCommand::Update { username, role } => {
            let body = serde_json::json!({ "role": role });
            client.put(&format!("/users/{}", username), &body).await?;
            fmt.print_success(&format!("User '{}' updated", username));
        }
        AdminCommand::Delete { username } => {
            super::handle_delete(client, fmt, &format!("/users/{}", username), &format!("User '{}' deleted", username)).await?;
        }
        AdminCommand::ChangePassword { username, old_password, new_password } => {
            let body = serde_json::json!({
                "old_pwd": old_password,
                "new_pwd": new_password,
            });
            client.put(&format!("/users/{}/change_pwd", username), &body).await?;
            fmt.print_success(&format!("Password changed for '{}'", username));
        }
    }
    Ok(())
}
