//! Server configuration and env parsing.

use std::net::IpAddr;
use std::path::Path;
use std::str::FromStr;

use dare_core::{CoreError, CoreResult, ProjectRoot};
use uuid::Uuid;

use crate::mode::AppMode;

pub const DEFAULT_DASHBOARD_BIND: &str = "127.0.0.1";
pub const DEFAULT_DASHBOARD_PORT: u16 = 4100;
pub const DEFAULT_REST_BIND: &str = "127.0.0.1";
pub const DEFAULT_REST_PORT: u16 = 3000;
pub const DEFAULT_BODY_LIMIT: usize = 1_048_576;

pub const ENV_BIND: &str = "DARE_MCP_BIND";
pub const ENV_PORT: &str = "DARE_MCP_PORT";
pub const ENV_TOKEN: &str = "DARE_MCP_TOKEN";
pub const ENV_BODY_LIMIT: &str = "DARE_MCP_BODY_LIMIT";
pub const ENV_PROJECT: &str = "DARE_PROJECT_PATH";
pub const ENV_LOG_TOKEN: &str = "DARE_MCP_LOG_TOKEN";

pub const CSP_DASHBOARD: &str = "default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:; frame-ancestors 'none'; base-uri 'self'; form-action 'self'";

const BODY_LIMIT_MIN: usize = 1024;
const BODY_LIMIT_MAX: usize = 16 * 1024 * 1024;
const TOKEN_ENV_MIN_LEN: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenSource {
    Env,
    Generated,
}

#[derive(Debug)]
pub struct ServerConfig {
    pub bind: IpAddr,
    pub port: u16,
    pub project_root: ProjectRoot,
    pub token: String,
    pub token_source: TokenSource,
    pub body_limit: usize,
    pub open_browser: bool,
    pub log_token_value: bool,
}

/// Parse server config from env with optional CLI overrides.
///
/// Priority: bind/port overrides > env > mode defaults.
pub fn parse_server_config_from_env(
    mode: AppMode,
    bind_override: Option<&str>,
    port_override: Option<u16>,
    project: &Path,
    open_browser: bool,
) -> CoreResult<ServerConfig> {
    let project_path = std::env::var(ENV_PROJECT)
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| project.to_path_buf());
    let project_root = ProjectRoot::new(&project_path)?;

    let default_bind = match mode {
        AppMode::Dashboard => DEFAULT_DASHBOARD_BIND,
        AppMode::Rest => DEFAULT_REST_BIND,
    };
    let bind_str = bind_override
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .or_else(|| std::env::var(ENV_BIND).ok().filter(|s| !s.trim().is_empty()))
        .unwrap_or_else(|| default_bind.to_string());
    let bind = IpAddr::from_str(bind_str.trim())
        .map_err(|_| CoreError::invalid_input(format!("invalid bind address: {bind_str}")))?;

    let default_port = match mode {
        AppMode::Dashboard => DEFAULT_DASHBOARD_PORT,
        AppMode::Rest => DEFAULT_REST_PORT,
    };
    let port = if let Some(p) = port_override {
        p
    } else if let Ok(raw) = std::env::var(ENV_PORT) {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            default_port
        } else {
            trimmed
                .parse::<u16>()
                .map_err(|_| CoreError::invalid_input(format!("invalid port: {raw}")))?
        }
    } else {
        default_port
    };
    if port == 0 {
        return Err(CoreError::invalid_input("port must be in 1..=65535"));
    }

    let (token, token_source) = match std::env::var(ENV_TOKEN) {
        Ok(t) if !t.trim().is_empty() => {
            let t = t.trim().to_string();
            if t.len() < TOKEN_ENV_MIN_LEN {
                return Err(CoreError::invalid_input(format!(
                    "{ENV_TOKEN} must be at least {TOKEN_ENV_MIN_LEN} characters"
                )));
            }
            (t, TokenSource::Env)
        }
        _ => (Uuid::new_v4().to_string(), TokenSource::Generated),
    };

    let body_limit = match std::env::var(ENV_BODY_LIMIT) {
        Ok(raw) if !raw.trim().is_empty() => parse_body_limit(raw.trim())?,
        _ => DEFAULT_BODY_LIMIT,
    };

    let log_token_value = match std::env::var(ENV_LOG_TOKEN) {
        Ok(v) => {
            let v = v.trim().to_ascii_lowercase();
            v == "1" || v == "true"
        }
        Err(_) => false,
    };

    Ok(ServerConfig {
        bind,
        port,
        project_root,
        token,
        token_source,
        body_limit,
        open_browser,
        log_token_value,
    })
}

fn parse_body_limit(raw: &str) -> CoreResult<usize> {
    let lower = raw.trim().to_ascii_lowercase();
    let bytes = if let Some(n) = lower.strip_suffix("mib") {
        let n = parse_usize_part(n, raw)?;
        n.checked_mul(1024 * 1024)
            .ok_or_else(|| CoreError::invalid_input(format!("body limit overflow: {raw}")))?
    } else if let Some(n) = lower.strip_suffix("mb") {
        let n = parse_usize_part(n, raw)?;
        n.checked_mul(1024 * 1024)
            .ok_or_else(|| CoreError::invalid_input(format!("body limit overflow: {raw}")))?
    } else if let Some(n) = lower.strip_suffix('k') {
        let n = parse_usize_part(n, raw)?;
        n.checked_mul(1024)
            .ok_or_else(|| CoreError::invalid_input(format!("body limit overflow: {raw}")))?
    } else {
        parse_usize_part(&lower, raw)?
    };

    if !(BODY_LIMIT_MIN..=BODY_LIMIT_MAX).contains(&bytes) {
        return Err(CoreError::invalid_input(format!(
            "body limit must be between {BODY_LIMIT_MIN} and {BODY_LIMIT_MAX} bytes"
        )));
    }
    Ok(bytes)
}

fn parse_usize_part(part: &str, raw: &str) -> CoreResult<usize> {
    part.trim()
        .parse::<usize>()
        .map_err(|_| CoreError::invalid_input(format!("invalid body limit: {raw}")))
}
