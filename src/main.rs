mod cli;
mod client;
mod commands;
mod config;
mod error;
mod input;
mod output;

use clap::Parser;

use cli::{Cli, Commands};
use client::EmqxClient;
use config::Config;
use error::format_error;
use output::OutputFormatter;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cli = Cli::parse();

    if cli.global.no_color {
        colored::control::set_override(false);
    }

    if let Err(err) = run(cli).await {
        format_error(&err);
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> anyhow::Result<()> {
    let fmt = OutputFormatter::new(cli.global.output);

    // Handle commands that don't need a client
    match &cli.command {
        Commands::Config { command } => {
            if commands::config_cmd::is_local_command(command) {
                return commands::config_cmd::execute_local(command);
            }
        }
        Commands::Completion { shell } => {
            commands::completion::execute(*shell);
            return Ok(());
        }
        _ => {}
    }

    // All other commands need a client
    let config = Config::load()?;
    let resolved = config.resolve_context(cli.global.context.as_deref())?;
    let client = EmqxClient::new(resolved, cli.global.verbose)?;

    match &cli.command {
        Commands::Status => commands::status::execute(&client, &fmt).await,
        Commands::Broker => {
            let value = client.get("/nodes").await?;
            fmt.print_value(&value);
            Ok(())
        }
        Commands::Metrics { node } => {
            commands::metrics::execute_metrics(&client, &fmt, node.as_deref()).await
        }
        Commands::Stats { node } => {
            commands::metrics::execute_stats(&client, &fmt, node.as_deref()).await
        }
        Commands::Node { command } => commands::node::execute(&client, &fmt, command).await,
        Commands::Cluster { command } => commands::cluster::execute(&client, &fmt, command).await,
        Commands::Client { command } => commands::clients::execute(&client, &fmt, command).await,
        Commands::Topic { command } => commands::topic::execute(&client, &fmt, command).await,
        Commands::Subscription { command } => {
            commands::subscription::execute(&client, &fmt, command).await
        }
        Commands::Publish { args } => commands::publish::execute(&client, &fmt, args).await,
        Commands::PublishBatch { file } => {
            commands::publish::execute_batch(&client, &fmt, file).await
        }
        Commands::Retainer { command } => commands::retainer::execute(&client, &fmt, command).await,
        Commands::Rule { command } => commands::rule::execute(&client, &fmt, command).await,
        Commands::Connector { command } => {
            commands::connector::execute(&client, &fmt, command).await
        }
        Commands::Action { command } => commands::action::execute(&client, &fmt, command).await,
        Commands::Source { command } => commands::source::execute(&client, &fmt, command).await,
        Commands::Authn { command } => commands::authn::execute(&client, &fmt, command).await,
        Commands::Authz { command } => commands::authz::execute(&client, &fmt, command).await,
        Commands::Ban { command } => commands::ban::execute(&client, &fmt, command).await,
        Commands::Listener { command } => commands::listener::execute(&client, &fmt, command).await,
        Commands::Alarm { command } => commands::alarm::execute(&client, &fmt, command).await,
        Commands::Trace { command } => commands::trace::execute(&client, &fmt, command).await,
        Commands::Config { command } => {
            commands::config_cmd::execute_remote(&client, &fmt, command).await
        }
        Commands::Plugin { command } => commands::plugin::execute(&client, &fmt, command).await,
        Commands::Apikey { command } => commands::apikey::execute(&client, &fmt, command).await,
        Commands::Admin { command } => commands::admin::execute(&client, &fmt, command).await,
        Commands::Gateway { command } => commands::gateway::execute(&client, &fmt, command).await,
        Commands::Schema { command } => commands::schema::execute(&client, &fmt, command).await,
        Commands::SlowSub { command } => commands::slow_sub::execute(&client, &fmt, command).await,
        Commands::TopicMetrics { command } => {
            commands::topic_metrics::execute(&client, &fmt, command).await
        }
        Commands::Backup { command } => commands::backup::execute(&client, &fmt, command).await,
        Commands::Cert { command } => commands::cert::execute(&client, &fmt, command).await,
        Commands::Api { args } => commands::api::execute(&client, &fmt, args).await,
        Commands::Completion { .. } => unreachable!(),
    }
}
