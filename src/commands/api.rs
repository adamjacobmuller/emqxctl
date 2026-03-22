use crate::client::EmqxClient;
use crate::output::OutputFormatter;
use anyhow::Result;
use clap::Args;
use reqwest::Method;

#[derive(Args)]
pub struct ApiArgs {
    /// HTTP method (GET, POST, PUT, DELETE)
    pub method: String,
    /// API path (e.g. /topics)
    pub path: String,
    /// Request body (JSON string)
    #[arg(long)]
    pub data: Option<String>,
    /// Request body from file
    #[arg(short = 'f', long)]
    pub file: Option<String>,
}

pub async fn execute(client: &EmqxClient, fmt: &OutputFormatter, args: &ApiArgs) -> Result<()> {
    let method: Method = args
        .method
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid HTTP method: {}", args.method))?;

    let body = if let Some(ref data) = args.data {
        Some(serde_json::from_str::<serde_json::Value>(data)?)
    } else if let Some(ref file) = args.file {
        Some(crate::input::read_input_file(file)?)
    } else {
        None
    };

    let result = client
        .request(method, &args.path, &[], body.as_ref())
        .await?;
    fmt.print_value(&result);
    Ok(())
}
