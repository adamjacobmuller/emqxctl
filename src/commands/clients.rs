use anyhow::Result;
use clap::{Args, Subcommand};
use crate::cli::PaginationArgs;
use crate::client::EmqxClient;
use crate::output::{Column, OutputFormatter};

#[derive(Subcommand)]
pub enum ClientCommand {
    /// List clients
    List {
        #[command(flatten)]
        pagination: PaginationArgs,
        #[command(flatten)]
        filters: ClientFilters,
    },
    /// Get client details
    Get { clientid: String },
    /// Kick (disconnect) a client
    Kick { clientid: String },
    /// List client subscriptions
    Subscriptions { clientid: String },
    /// Subscribe a client to a topic
    Subscribe {
        clientid: String,
        #[arg(short = 't', long)]
        topic: String,
        #[arg(short = 'q', long, default_value = "0")]
        qos: u8,
    },
    /// Unsubscribe a client from a topic
    Unsubscribe {
        clientid: String,
        #[arg(short = 't', long)]
        topic: String,
    },
    /// List client message queue
    Mqueue {
        clientid: String,
        #[arg(long, default_value = "100")]
        limit: u64,
        #[arg(long)]
        position: Option<String>,
    },
    /// List client inflight messages
    Inflight {
        clientid: String,
        #[arg(long, default_value = "100")]
        limit: u64,
        #[arg(long)]
        position: Option<String>,
    },
}

#[derive(Args)]
pub struct ClientFilters {
    #[arg(long)]
    pub username: Option<String>,
    #[arg(long)]
    pub ip_address: Option<String>,
    #[arg(long)]
    pub conn_state: Option<String>,
    #[arg(long)]
    pub clean_start: Option<bool>,
    #[arg(long)]
    pub proto_ver: Option<String>,
    #[arg(long)]
    pub like_clientid: Option<String>,
    #[arg(long)]
    pub like_username: Option<String>,
}

const LIST_COLUMNS: &[Column] = &[
    Column { header: "CLIENTID", json_path: "clientid", max_width: Some(30) },
    Column { header: "USERNAME", json_path: "username", max_width: Some(20) },
    Column { header: "IP ADDRESS", json_path: "ip_address", max_width: None },
    Column { header: "CONNECTED", json_path: "connected", max_width: None },
    Column { header: "PROTO", json_path: "proto_ver", max_width: None },
    Column { header: "CLEAN START", json_path: "clean_start", max_width: None },
];

const WIDE_COLUMNS: &[Column] = &[
    Column { header: "PORT", json_path: "port", max_width: None },
    Column { header: "KEEPALIVE", json_path: "keepalive", max_width: None },
    Column { header: "SUBSCRIPTIONS", json_path: "subscriptions_cnt", max_width: None },
    Column { header: "CREATED AT", json_path: "created_at", max_width: None },
    Column { header: "CONNECTED AT", json_path: "connected_at", max_width: None },
];

const DETAIL_COLUMNS: &[Column] = &[
    Column { header: "CLIENTID", json_path: "clientid", max_width: None },
    Column { header: "USERNAME", json_path: "username", max_width: None },
    Column { header: "IP ADDRESS", json_path: "ip_address", max_width: None },
    Column { header: "PORT", json_path: "port", max_width: None },
    Column { header: "CONNECTED", json_path: "connected", max_width: None },
    Column { header: "PROTO VER", json_path: "proto_ver", max_width: None },
    Column { header: "PROTO NAME", json_path: "proto_name", max_width: None },
    Column { header: "CLEAN START", json_path: "clean_start", max_width: None },
    Column { header: "KEEPALIVE", json_path: "keepalive", max_width: None },
    Column { header: "EXPIRY INTERVAL", json_path: "expiry_interval", max_width: None },
    Column { header: "SUBSCRIPTIONS", json_path: "subscriptions_cnt", max_width: None },
    Column { header: "INFLIGHT", json_path: "inflight_cnt", max_width: None },
    Column { header: "MQUEUE LEN", json_path: "mqueue_len", max_width: None },
    Column { header: "CREATED AT", json_path: "created_at", max_width: None },
    Column { header: "CONNECTED AT", json_path: "connected_at", max_width: None },
];

const SUB_COLUMNS: &[Column] = &[
    Column { header: "TOPIC", json_path: "topic", max_width: None },
    Column { header: "QOS", json_path: "qos", max_width: None },
    Column { header: "NL", json_path: "nl", max_width: None },
    Column { header: "RAP", json_path: "rap", max_width: None },
    Column { header: "RH", json_path: "rh", max_width: None },
];

pub async fn execute(client: &EmqxClient, fmt: &OutputFormatter, cmd: &ClientCommand) -> Result<()> {
    match cmd {
        ClientCommand::List { pagination, filters } => {
            let mut query: Vec<(&str, String)> = Vec::new();
            if let Some(ref v) = filters.username { query.push(("username", v.clone())); }
            if let Some(ref v) = filters.ip_address { query.push(("ip_address", v.clone())); }
            if let Some(ref v) = filters.conn_state { query.push(("conn_state", v.clone())); }
            if let Some(v) = filters.clean_start { query.push(("clean_start", v.to_string())); }
            if let Some(ref v) = filters.proto_ver { query.push(("proto_ver", v.clone())); }
            if let Some(ref v) = filters.like_clientid { query.push(("like_clientid", v.clone())); }
            if let Some(ref v) = filters.like_username { query.push(("like_username", v.clone())); }

            super::handle_paginated_list(
                client, fmt, "/clients", &query, pagination,
                LIST_COLUMNS, Some(WIDE_COLUMNS),
            ).await?;
        }
        ClientCommand::Get { clientid } => {
            let value = client.get(&format!("/clients/{}", clientid)).await?;
            fmt.print_item(&value, DETAIL_COLUMNS);
        }
        ClientCommand::Kick { clientid } => {
            client.delete(&format!("/clients/{}", clientid)).await?;
            fmt.print_success(&format!("Client '{}' kicked", clientid));
        }
        ClientCommand::Subscriptions { clientid } => {
            let value = client.get(&format!("/clients/{}/subscriptions", clientid)).await?;
            let items = super::extract_items(&value);
            fmt.print_list(&items, SUB_COLUMNS, None, None);
        }
        ClientCommand::Subscribe { clientid, topic, qos } => {
            let body = serde_json::json!({
                "topic": topic,
                "qos": qos,
            });
            client.post(&format!("/clients/{}/subscribe", clientid), &body).await?;
            fmt.print_success(&format!("Client '{}' subscribed to '{}'", clientid, topic));
        }
        ClientCommand::Unsubscribe { clientid, topic } => {
            let body = serde_json::json!({ "topic": topic });
            client.post(&format!("/clients/{}/unsubscribe", clientid), &body).await?;
            fmt.print_success(&format!("Client '{}' unsubscribed from '{}'", clientid, topic));
        }
        ClientCommand::Mqueue { clientid, limit, position } => {
            let (items, meta) = client.get_cursor_paginated(
                &format!("/clients/{}/mqueue_messages", clientid),
                &[], *limit, position.as_deref(),
            ).await?;
            fmt.print_list(&items, &[
                Column { header: "MSGID", json_path: "msgid", max_width: Some(20) },
                Column { header: "TOPIC", json_path: "topic", max_width: Some(30) },
                Column { header: "QOS", json_path: "qos", max_width: None },
                Column { header: "PUBLISH AT", json_path: "publish_at", max_width: None },
            ], None, Some(&meta));
            if let Some(pos) = &meta.position {
                eprintln!("Next position: {}", pos);
            }
        }
        ClientCommand::Inflight { clientid, limit, position } => {
            let (items, meta) = client.get_cursor_paginated(
                &format!("/clients/{}/inflight_messages", clientid),
                &[], *limit, position.as_deref(),
            ).await?;
            fmt.print_list(&items, &[
                Column { header: "MSGID", json_path: "msgid", max_width: Some(20) },
                Column { header: "TOPIC", json_path: "topic", max_width: Some(30) },
                Column { header: "QOS", json_path: "qos", max_width: None },
                Column { header: "PUBLISH AT", json_path: "publish_at", max_width: None },
            ], None, Some(&meta));
            if let Some(pos) = &meta.position {
                eprintln!("Next position: {}", pos);
            }
        }
    }
    Ok(())
}
