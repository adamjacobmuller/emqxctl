pub mod action;
pub mod admin;
pub mod alarm;
pub mod api;
pub mod apikey;
pub mod authn;
pub mod authz;
pub mod backup;
pub mod ban;
pub mod cert;
pub mod clients;
pub mod cluster;
pub mod completion;
pub mod config_cmd;
pub mod connector;
pub mod gateway;
pub mod listener;
pub mod metrics;
pub mod node;
pub mod plugin;
pub mod publish;
pub mod retainer;
pub mod rule;
pub mod schema;
pub mod slow_sub;
pub mod source;
pub mod status;
pub mod subscription;
pub mod topic;
pub mod topic_metrics;
pub mod trace;

// Shared helpers
use crate::cli::PaginationArgs;
use crate::client::EmqxClient;
use crate::input::read_input_file;
use crate::output::{Column, OutputFormatter};
use anyhow::Result;
use reqwest::Method;

pub async fn handle_paginated_list(
    client: &EmqxClient,
    fmt: &OutputFormatter,
    path: &str,
    query: &[(&str, String)],
    pagination: &PaginationArgs,
    columns: &[Column],
    wide_columns: Option<&[Column]>,
) -> Result<()> {
    if pagination.all {
        let items = client.get_all_pages(path, query, pagination.limit).await?;
        fmt.print_list(&items, columns, wide_columns, None);
    } else {
        let (items, meta) = client
            .get_paginated(path, query, pagination.page, pagination.limit)
            .await?;
        fmt.print_list(&items, columns, wide_columns, Some(&meta));
    }
    Ok(())
}

pub async fn handle_get(
    client: &EmqxClient,
    fmt: &OutputFormatter,
    path: &str,
    columns: &[Column],
) -> Result<()> {
    let value = client.get(path).await?;
    fmt.print_item(&value, columns);
    Ok(())
}

pub async fn handle_create_or_update(
    client: &EmqxClient,
    fmt: &OutputFormatter,
    method: Method,
    path: &str,
    file: &str,
    success_msg: &str,
) -> Result<()> {
    let body = read_input_file(file)?;
    let result = client.request(method, path, &[], Some(&body)).await?;
    if result.is_null() {
        fmt.print_success(success_msg);
    } else {
        fmt.print_value(&result);
    }
    Ok(())
}

/// Extract array items from a response that may be a bare array or `{data: [...], meta: {...}}`
pub fn extract_items(value: &serde_json::Value) -> Vec<serde_json::Value> {
    if let Some(arr) = value.as_array() {
        return arr.clone();
    }
    if let Some(obj) = value.as_object() {
        if let Some(data) = obj.get("data") {
            if let Some(arr) = data.as_array() {
                return arr.clone();
            }
        }
    }
    vec![value.clone()]
}

pub async fn handle_delete(
    client: &EmqxClient,
    fmt: &OutputFormatter,
    path: &str,
    success_msg: &str,
) -> Result<()> {
    client.delete(path).await?;
    fmt.print_success(success_msg);
    Ok(())
}
