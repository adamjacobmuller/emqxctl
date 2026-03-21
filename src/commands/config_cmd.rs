use anyhow::Result;
use clap::Subcommand;
use colored::Colorize;
use reqwest::Method;
use crate::client::EmqxClient;
use crate::config::Config;
use crate::output::OutputFormatter;

#[derive(Subcommand)]
pub enum ConfigCommand {
    /// Set/create a context
    #[command(name = "set-context")]
    SetContext {
        name: String,
        #[arg(long)]
        url: String,
        #[arg(long)]
        api_key: Option<String>,
        #[arg(long)]
        api_secret: Option<String>,
        #[arg(long)]
        username: Option<String>,
        #[arg(long)]
        password: Option<String>,
    },
    /// Switch current context
    #[command(name = "use-context")]
    UseContext { name: String },
    /// List all contexts
    #[command(name = "get-contexts")]
    GetContexts,
    /// Show current context
    #[command(name = "current-context")]
    CurrentContext,
    /// Delete a context
    #[command(name = "delete-context")]
    DeleteContext { name: String },
    /// Get remote EMQX configuration
    Get {
        /// Root config key (e.g. mqtt, listeners)
        root_key: Option<String>,
    },
    /// Update remote EMQX configuration
    Update {
        root_key: String,
        #[arg(short = 'f', long)]
        file: String,
    },
    /// Reset remote EMQX configuration
    Reset { root_key: String },
    /// Get global zone configuration
    Global,
}

/// Returns true if this is a local-only config command (no client needed)
pub fn is_local_command(cmd: &ConfigCommand) -> bool {
    matches!(
        cmd,
        ConfigCommand::SetContext { .. }
            | ConfigCommand::UseContext { .. }
            | ConfigCommand::GetContexts
            | ConfigCommand::CurrentContext
            | ConfigCommand::DeleteContext { .. }
    )
}

pub fn execute_local(cmd: &ConfigCommand) -> Result<()> {
    match cmd {
        ConfigCommand::SetContext { name, url, api_key, api_secret, username, password } => {
            let mut config = Config::load()?;
            let ctx = crate::config::ContextConfig {
                url: url.clone(),
                api_key: api_key.clone(),
                api_secret: api_secret.clone(),
                username: username.clone(),
                password: password.clone(),
            };
            config.contexts.insert(name.clone(), ctx);
            if config.current_context.is_none() {
                config.current_context = Some(name.clone());
            }
            config.save()?;
            println!("Context '{}' set.", name.green());
        }
        ConfigCommand::UseContext { name } => {
            let mut config = Config::load()?;
            if !config.contexts.contains_key(name) {
                anyhow::bail!("Context '{}' not found. Use `emqxctl config get-contexts` to see available contexts.", name);
            }
            config.current_context = Some(name.clone());
            config.save()?;
            println!("Switched to context '{}'.", name.green());
        }
        ConfigCommand::GetContexts => {
            let config = Config::load()?;
            if config.contexts.is_empty() {
                println!("No contexts configured.");
                return Ok(());
            }
            let current = config.current_context.as_deref().unwrap_or("");
            println!("{:<4} {:<20} {}", "", "NAME", "URL");
            for (name, ctx) in &config.contexts {
                let marker = if name == current { "*" } else { "" };
                let auth_type = if ctx.api_key.is_some() { "api-key" } else { "dashboard" };
                println!("{:<4} {:<20} {} ({})", marker, name, ctx.url, auth_type);
            }
        }
        ConfigCommand::CurrentContext => {
            let config = Config::load()?;
            match config.current_context {
                Some(name) => println!("{}", name),
                None => println!("No current context set."),
            }
        }
        ConfigCommand::DeleteContext { name } => {
            let mut config = Config::load()?;
            if config.contexts.remove(name).is_none() {
                anyhow::bail!("Context '{}' not found.", name);
            }
            if config.current_context.as_deref() == Some(name.as_str()) {
                config.current_context = None;
            }
            config.save()?;
            println!("Context '{}' deleted.", name);
        }
        _ => unreachable!(),
    }
    Ok(())
}

pub async fn execute_remote(client: &EmqxClient, fmt: &OutputFormatter, cmd: &ConfigCommand) -> Result<()> {
    match cmd {
        ConfigCommand::Get { root_key } => {
            let path = if let Some(key) = root_key {
                format!("/configs/{}", key)
            } else {
                "/configs".to_string()
            };
            let value = client.get(&path).await?;
            fmt.print_value(&value);
        }
        ConfigCommand::Update { root_key, file } => {
            super::handle_create_or_update(client, fmt, Method::PUT, &format!("/configs/{}", root_key), file, &format!("Config '{}' updated", root_key)).await?;
        }
        ConfigCommand::Reset { root_key } => {
            client.post(&format!("/configs/{}/reset", root_key), &serde_json::json!({})).await?;
            fmt.print_success(&format!("Config '{}' reset", root_key));
        }
        ConfigCommand::Global => {
            let value = client.get("/configs/global_zone").await?;
            fmt.print_value(&value);
        }
        _ => unreachable!(),
    }
    Ok(())
}
