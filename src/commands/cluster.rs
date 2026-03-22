use crate::client::EmqxClient;
use crate::output::OutputFormatter;
use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum ClusterCommand {
    /// Cluster status
    Status,
    /// Cluster metrics
    Metrics {
        /// Latest N metrics
        #[arg(long)]
        latest: Option<u32>,
    },
}

pub async fn execute(
    client: &EmqxClient,
    fmt: &OutputFormatter,
    cmd: &ClusterCommand,
) -> Result<()> {
    match cmd {
        ClusterCommand::Status => {
            let value = client.get("/cluster").await?;
            fmt.print_value(&value);
        }
        ClusterCommand::Metrics { latest } => {
            let value = if let Some(n) = latest {
                let q = n.to_string();
                client
                    .get_with_query("/monitor_current", &[("latest", q.as_str())])
                    .await?
            } else {
                client.get("/monitor_current").await?
            };
            fmt.print_value(&value);
        }
    }
    Ok(())
}
