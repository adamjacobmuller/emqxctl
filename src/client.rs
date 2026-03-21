use std::time::Duration;

use anyhow::Result;
use base64::Engine;
use reqwest::{Client, Method, StatusCode};
use serde::Deserialize;
use serde_json::Value;

use crate::config::{AuthMethod, Config, ResolvedContext};
use crate::error::AppError;

#[derive(Debug, Deserialize)]
pub struct PaginationMeta {
    pub page: Option<u64>,
    pub limit: Option<u64>,
    pub count: Option<u64>,
    pub hasnext: Option<bool>,
    pub position: Option<String>,
}

#[derive(Debug, Deserialize)]
struct EmqxErrorResponse {
    code: Option<String>,
    #[serde(alias = "message")]
    reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LoginResponse {
    token: String,
    #[serde(default)]
    #[allow(dead_code)]
    license: Option<Value>,
}

pub struct EmqxClient {
    http: Client,
    context: ResolvedContext,
    verbose: bool,
    bearer_token: std::sync::Mutex<Option<String>>,
}

impl EmqxClient {
    pub fn new(context: ResolvedContext, verbose: bool) -> Result<Self> {
        let http = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .danger_accept_invalid_certs(true)
            .build()?;

        // Load cached bearer token if using dashboard auth
        let bearer_token = if let AuthMethod::Dashboard { .. } = &context.auth {
            Config::load_token_cache(&context.name).map(|c| c.token)
        } else {
            None
        };

        Ok(Self {
            http,
            context,
            verbose,
            bearer_token: std::sync::Mutex::new(bearer_token),
        })
    }

    #[allow(dead_code)]
    pub fn base_url(&self) -> &str {
        &self.context.url
    }

    fn url(&self, path: &str) -> String {
        format!("{}/api/v5{}", self.context.url, path)
    }

    fn verbose_log(&self, msg: &str) {
        if self.verbose {
            eprintln!("{}", msg);
        }
    }

    async fn login(&self) -> Result<String> {
        if let AuthMethod::Dashboard {
            ref username,
            ref password,
        } = self.context.auth
        {
            let url = format!("{}/api/v5/login", self.context.url);
            self.verbose_log(&format!("> POST {}", url));

            let body = serde_json::json!({
                "username": username,
                "password": password,
            });

            let resp = self
                .http
                .post(&url)
                .json(&body)
                .send()
                .await
                .map_err(|e| -> anyhow::Error {
                    if e.is_connect() {
                        AppError::ConnectionFailed {
                            url: url.clone(),
                            source: e,
                        }
                        .into()
                    } else {
                        e.into()
                    }
                })?;

            let status = resp.status();
            if !status.is_success() {
                return Err(AppError::AuthFailed {
                    context: self.context.name.clone(),
                }
                .into());
            }

            let login: LoginResponse = resp.json().await?;

            // Cache with 2hr TTL
            let expires_at = chrono::Utc::now().timestamp() + 7200;
            Config::save_token_cache(&self.context.name, &login.token, expires_at)?;

            Ok(login.token)
        } else {
            anyhow::bail!("login() called on non-dashboard auth context");
        }
    }

    async fn ensure_bearer_token(&self) -> Result<String> {
        {
            let guard = self.bearer_token.lock().unwrap();
            if let Some(ref token) = *guard {
                return Ok(token.clone());
            }
        }
        let token = self.login().await?;
        {
            let mut guard = self.bearer_token.lock().unwrap();
            *guard = Some(token.clone());
        }
        Ok(token)
    }

    fn invalidate_bearer_token(&self) {
        let mut guard = self.bearer_token.lock().unwrap();
        *guard = None;
    }

    pub async fn request(
        &self,
        method: Method,
        path: &str,
        query: &[(&str, &str)],
        body: Option<&Value>,
    ) -> Result<Value> {
        let url = self.url(path);
        let max_retries = 3u32;

        for attempt in 0..=max_retries {
            let mut req = self.http.request(method.clone(), &url);

            // Auth
            match &self.context.auth {
                AuthMethod::ApiKey { key, secret } => {
                    let credentials = format!("{}:{}", key, secret);
                    let encoded =
                        base64::engine::general_purpose::STANDARD.encode(credentials.as_bytes());
                    req = req.header("Authorization", format!("Basic {}", encoded));
                }
                AuthMethod::Dashboard { .. } => {
                    let token = self.ensure_bearer_token().await?;
                    req = req.header("Authorization", format!("Bearer {}", token));
                }
            }

            // Query params
            if !query.is_empty() {
                req = req.query(query);
            }

            // Body
            if let Some(body) = body {
                req = req.json(body);
            }

            self.verbose_log(&format!("> {} {}", method, url));
            if !query.is_empty() {
                self.verbose_log(&format!(">   query: {:?}", query));
            }
            if let Some(body) = body {
                self.verbose_log(&format!(">   body: {}", serde_json::to_string(body)?));
            }

            let result = req.send().await;

            let resp = match result {
                Ok(resp) => resp,
                Err(e) => {
                    if e.is_connect() {
                        return Err(anyhow::Error::new(AppError::ConnectionFailed {
                            url: url.clone(),
                            source: e,
                        }));
                    }
                    if e.is_timeout() {
                        return Err(anyhow::Error::new(AppError::Timeout { url: url.clone() }));
                    }
                    // Retry on transient errors
                    if attempt < max_retries {
                        let delay = Duration::from_millis(500 * 2u64.pow(attempt));
                        self.verbose_log(&format!(
                            "  Retrying in {:?} (attempt {}/{})",
                            delay,
                            attempt + 1,
                            max_retries
                        ));
                        tokio::time::sleep(delay).await;
                        continue;
                    }
                    return Err(e.into());
                }
            };

            let status = resp.status();
            self.verbose_log(&format!("< {} {}", status.as_u16(), status.canonical_reason().unwrap_or("")));

            // Retry on transient server errors
            if matches!(
                status,
                StatusCode::BAD_GATEWAY | StatusCode::SERVICE_UNAVAILABLE | StatusCode::GATEWAY_TIMEOUT
            ) && attempt < max_retries
            {
                let delay = Duration::from_millis(500 * 2u64.pow(attempt));
                self.verbose_log(&format!(
                    "  Retrying in {:?} (attempt {}/{})",
                    delay,
                    attempt + 1,
                    max_retries
                ));
                tokio::time::sleep(delay).await;
                continue;
            }

            // 401 with bearer auth → refresh token and retry once
            if status == StatusCode::UNAUTHORIZED {
                if let AuthMethod::Dashboard { .. } = &self.context.auth {
                    if attempt == 0 {
                        self.verbose_log("  Bearer token expired, refreshing...");
                        self.invalidate_bearer_token();
                        continue;
                    }
                }
                return Err(AppError::AuthFailed {
                    context: self.context.name.clone(),
                }
                .into());
            }

            // 204 No Content
            if status == StatusCode::NO_CONTENT {
                return Ok(Value::Null);
            }

            let response_text = resp.text().await?;
            self.verbose_log(&format!("< body: {}", &response_text));

            if !status.is_success() {
                // Try to parse EMQX error response
                if let Ok(err_resp) = serde_json::from_str::<EmqxErrorResponse>(&response_text) {
                    return Err(AppError::EmqxApi {
                        status: status.as_u16(),
                        code: err_resp.code.unwrap_or_else(|| "UNKNOWN".into()),
                        reason: err_resp
                            .reason
                            .unwrap_or_else(|| response_text.clone()),
                    }
                    .into());
                }
                return Err(AppError::EmqxApi {
                    status: status.as_u16(),
                    code: "UNKNOWN".into(),
                    reason: response_text,
                }
                .into());
            }

            // Parse response
            if response_text.is_empty() {
                return Ok(Value::Null);
            }
            // Try JSON first; if it fails, return raw text as a string value
            // (some EMQX endpoints like /configs return HOCON, not JSON)
            let value: Value = match serde_json::from_str(&response_text) {
                Ok(v) => v,
                Err(_) => Value::String(response_text),
            };
            return Ok(value);
        }

        unreachable!()
    }

    pub async fn get(&self, path: &str) -> Result<Value> {
        self.request(Method::GET, path, &[], None).await
    }

    pub async fn get_with_query(&self, path: &str, query: &[(&str, &str)]) -> Result<Value> {
        self.request(Method::GET, path, query, None).await
    }

    pub async fn post(&self, path: &str, body: &Value) -> Result<Value> {
        self.request(Method::POST, path, &[], Some(body)).await
    }

    pub async fn put(&self, path: &str, body: &Value) -> Result<Value> {
        self.request(Method::PUT, path, &[], Some(body)).await
    }

    pub async fn delete(&self, path: &str) -> Result<Value> {
        self.request(Method::DELETE, path, &[], None).await
    }

    #[allow(dead_code)]
    pub async fn delete_with_query(&self, path: &str, query: &[(&str, &str)]) -> Result<Value> {
        self.request(Method::DELETE, path, query, None).await
    }

    /// Get a paginated list. Returns (items, meta).
    pub async fn get_paginated(
        &self,
        path: &str,
        extra_query: &[(&str, String)],
        page: u64,
        limit: u64,
    ) -> Result<(Vec<Value>, PaginationMeta)> {
        let page_str = page.to_string();
        let limit_str = limit.to_string();
        let mut query: Vec<(&str, &str)> = vec![("page", &page_str), ("limit", &limit_str)];
        for (k, v) in extra_query {
            query.push((k, v.as_str()));
        }
        let resp = self.request(Method::GET, path, &query, None).await?;

        // EMQX paginated response: { "data": [...], "meta": { ... } }
        if let Some(obj) = resp.as_object() {
            if let (Some(data), Some(meta)) = (obj.get("data"), obj.get("meta")) {
                let items = data.as_array().cloned().unwrap_or_default();
                let pagination: PaginationMeta = serde_json::from_value(meta.clone())?;
                return Ok((items, pagination));
            }
        }

        // Some endpoints return bare arrays
        if let Some(arr) = resp.as_array() {
            let meta = PaginationMeta {
                page: Some(1),
                limit: None,
                count: Some(arr.len() as u64),
                hasnext: Some(false),
                position: None,
            };
            return Ok((arr.clone(), meta));
        }

        Ok((vec![resp], PaginationMeta {
            page: Some(1),
            limit: None,
            count: Some(1),
            hasnext: Some(false),
            position: None,
        }))
    }

    /// Iterate all pages and return all items.
    pub async fn get_all_pages(
        &self,
        path: &str,
        extra_query: &[(&str, String)],
        limit: u64,
    ) -> Result<Vec<Value>> {
        let mut all_items = Vec::new();
        let mut page = 1u64;
        loop {
            let (items, meta) = self.get_paginated(path, extra_query, page, limit).await?;
            all_items.extend(items);
            if meta.hasnext == Some(true) {
                page += 1;
            } else {
                break;
            }
        }
        Ok(all_items)
    }

    /// Cursor-based pagination (for mqueue, inflight).
    pub async fn get_cursor_paginated(
        &self,
        path: &str,
        extra_query: &[(&str, String)],
        limit: u64,
        position: Option<&str>,
    ) -> Result<(Vec<Value>, PaginationMeta)> {
        let limit_str = limit.to_string();
        let mut query: Vec<(&str, &str)> = vec![("limit", &limit_str)];
        if let Some(pos) = position {
            query.push(("position", pos));
        }
        for (k, v) in extra_query {
            query.push((k, v.as_str()));
        }
        let resp = self.request(Method::GET, path, &query, None).await?;

        if let Some(obj) = resp.as_object() {
            if let (Some(data), Some(meta)) = (obj.get("data"), obj.get("meta")) {
                let items = data.as_array().cloned().unwrap_or_default();
                let pagination: PaginationMeta = serde_json::from_value(meta.clone())?;
                return Ok((items, pagination));
            }
        }

        if let Some(arr) = resp.as_array() {
            let meta = PaginationMeta {
                page: None,
                limit: None,
                count: Some(arr.len() as u64),
                hasnext: Some(false),
                position: None,
            };
            return Ok((arr.clone(), meta));
        }

        Ok((vec![], PaginationMeta {
            page: None,
            limit: None,
            count: Some(0),
            hasnext: Some(false),
            position: None,
        }))
    }

    /// Upload a file via multipart POST.
    pub async fn upload(&self, path: &str, filepath: &str) -> Result<Value> {
        let url = self.url(path);
        let file_bytes = tokio::fs::read(filepath).await?;
        let filename = std::path::Path::new(filepath)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let part = reqwest::multipart::Part::bytes(file_bytes).file_name(filename);
        let form = reqwest::multipart::Form::new().part("filename", part);

        let mut req = self.http.post(&url).multipart(form);

        match &self.context.auth {
            AuthMethod::ApiKey { key, secret } => {
                let credentials = format!("{}:{}", key, secret);
                let encoded =
                    base64::engine::general_purpose::STANDARD.encode(credentials.as_bytes());
                req = req.header("Authorization", format!("Basic {}", encoded));
            }
            AuthMethod::Dashboard { .. } => {
                let token = self.ensure_bearer_token().await?;
                req = req.header("Authorization", format!("Bearer {}", token));
            }
        }

        self.verbose_log(&format!("> POST {} (multipart upload)", url));

        let resp = req.send().await?;
        let status = resp.status();

        if status == StatusCode::NO_CONTENT {
            return Ok(Value::Null);
        }

        let text = resp.text().await?;

        if !status.is_success() {
            if let Ok(err_resp) = serde_json::from_str::<EmqxErrorResponse>(&text) {
                return Err(AppError::EmqxApi {
                    status: status.as_u16(),
                    code: err_resp.code.unwrap_or_else(|| "UNKNOWN".into()),
                    reason: err_resp.reason.unwrap_or(text),
                }
                .into());
            }
            return Err(AppError::EmqxApi {
                status: status.as_u16(),
                code: "UNKNOWN".into(),
                reason: text,
            }
            .into());
        }

        if text.is_empty() {
            return Ok(Value::Null);
        }
        Ok(serde_json::from_str(&text)?)
    }

    /// Download a binary file.
    pub async fn download(&self, path: &str, dest: &str) -> Result<()> {
        let url = self.url(path);
        let mut req = self.http.get(&url);

        match &self.context.auth {
            AuthMethod::ApiKey { key, secret } => {
                let credentials = format!("{}:{}", key, secret);
                let encoded =
                    base64::engine::general_purpose::STANDARD.encode(credentials.as_bytes());
                req = req.header("Authorization", format!("Basic {}", encoded));
            }
            AuthMethod::Dashboard { .. } => {
                let token = self.ensure_bearer_token().await?;
                req = req.header("Authorization", format!("Bearer {}", token));
            }
        }

        self.verbose_log(&format!("> GET {} (download)", url));

        let resp = req.send().await?;
        let status = resp.status();

        if !status.is_success() {
            let text = resp.text().await?;
            return Err(AppError::EmqxApi {
                status: status.as_u16(),
                code: "DOWNLOAD_FAILED".into(),
                reason: text,
            }
            .into());
        }

        let bytes = resp.bytes().await?;
        tokio::fs::write(dest, &bytes).await?;

        Ok(())
    }

    /// Get raw text response (for trace logs).
    pub async fn get_text(&self, path: &str) -> Result<String> {
        let url = self.url(path);
        let mut req = self.http.get(&url);

        match &self.context.auth {
            AuthMethod::ApiKey { key, secret } => {
                let credentials = format!("{}:{}", key, secret);
                let encoded =
                    base64::engine::general_purpose::STANDARD.encode(credentials.as_bytes());
                req = req.header("Authorization", format!("Basic {}", encoded));
            }
            AuthMethod::Dashboard { .. } => {
                let token = self.ensure_bearer_token().await?;
                req = req.header("Authorization", format!("Bearer {}", token));
            }
        }

        self.verbose_log(&format!("> GET {} (text)", url));

        let resp = req.send().await?;
        let status = resp.status();

        if !status.is_success() {
            let text = resp.text().await?;
            return Err(AppError::EmqxApi {
                status: status.as_u16(),
                code: "UNKNOWN".into(),
                reason: text,
            }
            .into());
        }

        Ok(resp.text().await?)
    }
}
