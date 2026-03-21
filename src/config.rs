use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub contexts: BTreeMap<String, ContextConfig>,
    #[serde(rename = "current-context", skip_serializing_if = "Option::is_none")]
    pub current_context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_secret: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

#[derive(Debug, Clone)]
pub enum AuthMethod {
    ApiKey { key: String, secret: String },
    Dashboard { username: String, password: String },
}

#[derive(Debug, Clone)]
pub struct ResolvedContext {
    pub name: String,
    pub url: String,
    pub auth: AuthMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenCache {
    pub token: String,
    pub expires_at: i64,
}

impl Config {
    pub fn config_dir() -> PathBuf {
        dirs::home_dir()
            .expect("Could not determine home directory")
            .join(".emqxctl")
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.yaml")
    }

    pub fn cache_dir() -> PathBuf {
        Self::config_dir().join("cache")
    }

    pub fn load() -> Result<Config> {
        let path = Self::config_path();
        if !path.exists() {
            return Ok(Config::default());
        }
        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        let config: Config = serde_yaml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let dir = Self::config_dir();
        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))?;
            }
        }
        let path = Self::config_path();
        let yaml = serde_yaml::to_string(self)?;
        std::fs::write(&path, yaml)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
        }
        Ok(())
    }

    pub fn resolve_context(&self, explicit: Option<&str>) -> Result<ResolvedContext> {
        // Priority: --context flag > EMQXCTL_CONTEXT env > current-context field
        let name = if let Some(name) = explicit {
            name.to_string()
        } else if let Ok(env_ctx) = std::env::var("EMQXCTL_CONTEXT") {
            if !env_ctx.is_empty() {
                env_ctx
            } else {
                return Err(AppError::NoContext.into());
            }
        } else if let Some(ref current) = self.current_context {
            current.clone()
        } else {
            return Err(AppError::NoContext.into());
        };

        let ctx_config = self
            .contexts
            .get(&name)
            .ok_or_else(|| AppError::ContextNotFound(name.clone()))?;

        let url = ctx_config.url.trim_end_matches('/').to_string();

        let auth = if let (Some(key), Some(secret)) =
            (&ctx_config.api_key, &ctx_config.api_secret)
        {
            AuthMethod::ApiKey {
                key: key.clone(),
                secret: secret.clone(),
            }
        } else if let (Some(username), Some(password)) =
            (&ctx_config.username, &ctx_config.password)
        {
            AuthMethod::Dashboard {
                username: username.clone(),
                password: password.clone(),
            }
        } else {
            return Err(AppError::InvalidConfig(format!(
                "Context '{}' has no valid auth configuration. Provide api_key+api_secret or username+password.",
                name
            ))
            .into());
        };

        Ok(ResolvedContext { name, url, auth })
    }

    pub fn load_token_cache(context_name: &str) -> Option<TokenCache> {
        let path = Self::cache_dir().join(format!("{}_token.json", context_name));
        let contents = std::fs::read_to_string(path).ok()?;
        let cache: TokenCache = serde_json::from_str(&contents).ok()?;
        let now = chrono::Utc::now().timestamp();
        if cache.expires_at > now {
            Some(cache)
        } else {
            None
        }
    }

    pub fn save_token_cache(context_name: &str, token: &str, expires_at: i64) -> Result<()> {
        let dir = Self::cache_dir();
        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))?;
            }
        }
        let cache = TokenCache {
            token: token.to_string(),
            expires_at,
        };
        let path = dir.join(format!("{}_token.json", context_name));
        let json = serde_json::to_string(&cache)?;
        std::fs::write(&path, json)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
        }
        Ok(())
    }
}
