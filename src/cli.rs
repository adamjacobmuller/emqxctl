use clap::{Args, Parser, Subcommand};

use crate::output::OutputFormat;

#[derive(Parser)]
#[command(name = "emqxctl", version, about = "CLI for the EMQX Management REST API")]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalArgs,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Args)]
pub struct GlobalArgs {
    /// Target EMQX context
    #[arg(short = 'c', long, global = true, env = "EMQXCTL_CONTEXT")]
    pub context: Option<String>,

    /// Output format
    #[arg(short = 'o', long, global = true, default_value = "table", value_enum)]
    pub output: OutputFormat,

    /// Show HTTP request/response details
    #[arg(short = 'v', long, global = true)]
    pub verbose: bool,

    /// Disable colored output
    #[arg(long, global = true, env = "NO_COLOR")]
    pub no_color: bool,
}

#[derive(Args, Clone)]
pub struct PaginationArgs {
    /// Page number
    #[arg(long, default_value = "1")]
    pub page: u64,

    /// Items per page
    #[arg(long, default_value = "100")]
    pub limit: u64,

    /// Fetch all pages
    #[arg(long)]
    pub all: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Quick health check
    Status,

    /// Cluster-level broker summary
    Broker,

    /// Cluster-wide metrics
    Metrics {
        /// Show metrics for a specific node
        #[arg(long)]
        node: Option<String>,
    },

    /// Cluster-wide stats
    Stats {
        /// Show stats for a specific node
        #[arg(long)]
        node: Option<String>,
    },

    /// Node management
    Node {
        #[command(subcommand)]
        command: crate::commands::node::NodeCommand,
    },

    /// Cluster management
    Cluster {
        #[command(subcommand)]
        command: crate::commands::cluster::ClusterCommand,
    },

    /// Client management
    Client {
        #[command(subcommand)]
        command: crate::commands::clients::ClientCommand,
    },

    /// Topic management
    Topic {
        #[command(subcommand)]
        command: crate::commands::topic::TopicCommand,
    },

    /// Subscription management
    Subscription {
        #[command(subcommand)]
        command: crate::commands::subscription::SubscriptionCommand,
    },

    /// Publish messages
    Publish {
        #[command(flatten)]
        args: crate::commands::publish::PublishArgs,
    },

    /// Publish batch messages
    PublishBatch {
        /// JSON file with array of messages
        #[arg(short = 'f', long)]
        file: String,
    },

    /// Retained messages
    Retainer {
        #[command(subcommand)]
        command: crate::commands::retainer::RetainerCommand,
    },

    /// Rules engine
    Rule {
        #[command(subcommand)]
        command: crate::commands::rule::RuleCommand,
    },

    /// Data integration connectors
    Connector {
        #[command(subcommand)]
        command: crate::commands::connector::ConnectorCommand,
    },

    /// Data integration actions
    Action {
        #[command(subcommand)]
        command: crate::commands::action::ActionCommand,
    },

    /// Data integration sources
    Source {
        #[command(subcommand)]
        command: crate::commands::source::SourceCommand,
    },

    /// Authentication management
    Authn {
        #[command(subcommand)]
        command: crate::commands::authn::AuthnCommand,
    },

    /// Authorization management
    Authz {
        #[command(subcommand)]
        command: crate::commands::authz::AuthzCommand,
    },

    /// Banned clients management
    Ban {
        #[command(subcommand)]
        command: crate::commands::ban::BanCommand,
    },

    /// Listener management
    Listener {
        #[command(subcommand)]
        command: crate::commands::listener::ListenerCommand,
    },

    /// Alarm management
    Alarm {
        #[command(subcommand)]
        command: crate::commands::alarm::AlarmCommand,
    },

    /// Log tracing
    Trace {
        #[command(subcommand)]
        command: crate::commands::trace::TraceCommand,
    },

    /// Configuration management (local contexts + remote EMQX config)
    Config {
        #[command(subcommand)]
        command: crate::commands::config_cmd::ConfigCommand,
    },

    /// Plugin management
    Plugin {
        #[command(subcommand)]
        command: crate::commands::plugin::PluginCommand,
    },

    /// API key management
    Apikey {
        #[command(subcommand)]
        command: crate::commands::apikey::ApikeyCommand,
    },

    /// Dashboard user management
    Admin {
        #[command(subcommand)]
        command: crate::commands::admin::AdminCommand,
    },

    /// Gateway management
    Gateway {
        #[command(subcommand)]
        command: crate::commands::gateway::GatewayCommand,
    },

    /// Schema registry
    Schema {
        #[command(subcommand)]
        command: crate::commands::schema::SchemaCommand,
    },

    /// Slow subscriptions
    #[command(name = "slow-sub")]
    SlowSub {
        #[command(subcommand)]
        command: crate::commands::slow_sub::SlowSubCommand,
    },

    /// Topic metrics
    #[command(name = "topic-metrics")]
    TopicMetrics {
        #[command(subcommand)]
        command: crate::commands::topic_metrics::TopicMetricsCommand,
    },

    /// Data backup management
    Backup {
        #[command(subcommand)]
        command: crate::commands::backup::BackupCommand,
    },

    /// Certificate management
    Cert {
        #[command(subcommand)]
        command: crate::commands::cert::CertCommand,
    },

    /// Raw API escape hatch
    Api {
        #[command(flatten)]
        args: crate::commands::api::ApiArgs,
    },

    /// Generate shell completions
    Completion {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}
