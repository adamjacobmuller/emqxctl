use colored::Colorize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Context '{0}' not found")]
    ContextNotFound(String),

    #[error("No context set. Use `emqxctl config set-context` or --context")]
    NoContext,

    #[error("EMQX API error ({status}): [{code}] {reason}")]
    EmqxApi {
        status: u16,
        code: String,
        reason: String,
    },

    #[error("Authentication failed for context '{context}'")]
    AuthFailed { context: String },

    #[error("Connection failed: {url}")]
    ConnectionFailed { url: String, source: reqwest::Error },

    #[error("Request timed out: {url}")]
    Timeout { url: String },

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

impl AppError {
    pub fn hint(&self) -> Option<String> {
        match self {
            AppError::ContextNotFound(name) => Some(format!(
                "Available contexts can be listed with: emqxctl config get-contexts\n\
                 Create one with: emqxctl config set-context {} --url <url> --api-key <key> --api-secret <secret>",
                name
            )),
            AppError::NoContext => Some(
                "Set a context with: emqxctl config set-context <name> --url <url> --api-key <key> --api-secret <secret>\n\
                 Or use --context <name> to specify one".to_string()
            ),
            AppError::AuthFailed { .. } => Some(
                "Check your credentials with: emqxctl config get-contexts\n\
                 Update with: emqxctl config set-context <name> --url <url> --api-key <key> --api-secret <secret>".to_string()
            ),
            AppError::ConnectionFailed { url, .. } => Some(format!(
                "Could not connect to {}. Check that:\n\
                 - The EMQX dashboard is running\n\
                 - The URL is correct (default port is 18083)\n\
                 - Network/firewall allows the connection",
                url
            )),
            _ => None,
        }
    }
}

pub fn format_error(err: &anyhow::Error) {
    if let Some(app_err) = err.downcast_ref::<AppError>() {
        eprintln!("{} {}", "error:".red().bold(), app_err);
        if let Some(hint) = app_err.hint() {
            eprintln!("\n{} {}", "hint:".yellow().bold(), hint);
        }
    } else {
        eprintln!("{} {}", "error:".red().bold(), err);
        // Print chain
        let mut source = err.source();
        while let Some(cause) = source {
            eprintln!("  {} {}", "caused by:".dimmed(), cause);
            source = std::error::Error::source(cause);
        }
    }
}

