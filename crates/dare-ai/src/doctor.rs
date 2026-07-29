//! Provider doctor diagnostics (schemaVersion 1) — PATH probe only, no enrich spawn.

use std::path::Path;

use dare_core::CoreResult;
use serde::{Deserialize, Serialize};

use crate::capabilities::{
    default_program, default_timeout_secs, env_override_name, is_implemented, PROVIDER_ORDER,
};
use crate::codex::parse_argv_override;
use crate::provider::ProviderId;

/// Frozen JSON schema version for [`DoctorReport`].
pub const DOCTOR_SCHEMA_VERSION: u32 = 1;

/// Doctor status per provider (BLUEPRINT-050 §0.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DoctorStatus {
    Ready,
    Missing,
    Invalid,
    NotImplemented,
}

/// One provider row in [`DoctorReport`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProviderDoctorEntry {
    pub id: String,
    pub status: DoctorStatus,
    pub implemented: bool,
    pub program: String,
    pub env_override: Option<String>,
    pub reason: Option<String>,
    pub default_timeout_secs: u64,
}

/// Doctor aggregate report (`schemaVersion` 1, camelCase JSON).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DoctorReport {
    pub schema_version: u32,
    pub ok: bool,
    pub providers: Vec<ProviderDoctorEntry>,
}

/// Diagnose a single provider. Never calls [`AiProvider::enrich`](crate::AiProvider::enrich).
pub fn diagnose_provider(id: ProviderId) -> CoreResult<ProviderDoctorEntry> {
    Ok(diagnose_provider_inner(id))
}

/// Diagnose all providers in canonical order.
pub fn diagnose_all() -> CoreResult<DoctorReport> {
    let providers: Vec<ProviderDoctorEntry> = PROVIDER_ORDER
        .iter()
        .copied()
        .map(diagnose_provider_inner)
        .collect();
    Ok(DoctorReport {
        schema_version: DOCTOR_SCHEMA_VERSION,
        ok: true,
        providers,
    })
}

fn diagnose_provider_inner(id: ProviderId) -> ProviderDoctorEntry {
    let timeout = default_timeout_secs();
    let env_name = env_override_name(id);
    let implemented = is_implemented(id);

    if id == ProviderId::Mock {
        return ProviderDoctorEntry {
            id: id.as_str().to_string(),
            status: DoctorStatus::Ready,
            implemented: true,
            program: default_program(id).to_string(),
            env_override: None,
            reason: None,
            default_timeout_secs: timeout,
        };
    }

    if !implemented {
        return ProviderDoctorEntry {
            id: id.as_str().to_string(),
            status: DoctorStatus::NotImplemented,
            implemented: false,
            program: resolve_declared_program(id, env_name),
            env_override: env_name.map(str::to_string),
            reason: Some(format!("provider not implemented: {}", id.as_str())),
            default_timeout_secs: timeout,
        };
    }

    // Implemented CLI (codex / claude / cursor / antigravity): parse override / default, then PATH probe only.
    let (program, status, reason) = match env_name.and_then(|n| std::env::var(n).ok()) {
        Some(val) => match parse_argv_override(&val) {
            Ok((program, _)) => probe_program(program),
            Err(_) => (
                default_program(id).to_string(),
                DoctorStatus::Invalid,
                Some("command override must not be empty".to_string()),
            ),
        },
        None => probe_program(default_program(id).to_string()),
    };

    ProviderDoctorEntry {
        id: id.as_str().to_string(),
        status,
        implemented: true,
        program,
        env_override: env_name.map(str::to_string),
        reason,
        default_timeout_secs: timeout,
    }
}

fn resolve_declared_program(id: ProviderId, env_name: Option<&str>) -> String {
    match env_name.and_then(|n| std::env::var(n).ok()) {
        Some(val) => parse_argv_override(&val)
            .map(|(p, _)| p)
            .unwrap_or_else(|_| default_program(id).to_string()),
        None => default_program(id).to_string(),
    }
}

fn probe_program(program: String) -> (String, DoctorStatus, Option<String>) {
    if program_resolves(&program) {
        (program, DoctorStatus::Ready, None)
    } else {
        let reason = format!("provider executable not found: {program}");
        (program, DoctorStatus::Missing, Some(reason))
    }
}

/// Resolve program on PATH or as a filesystem path — no process spawn.
fn program_resolves(program: &str) -> bool {
    let path = Path::new(program);
    if path.is_absolute() || program.contains('/') || program.contains('\\') {
        return path.is_file();
    }

    let Ok(path_env) = std::env::var("PATH") else {
        return false;
    };
    let sep = if cfg!(windows) { ';' } else { ':' };

    #[cfg(windows)]
    let extensions: Vec<String> = std::env::var("PATHEXT")
        .unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".into())
        .split(';')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    for dir in path_env.split(sep).filter(|d| !d.is_empty()) {
        let base = Path::new(dir).join(program);
        if base.is_file() {
            return true;
        }
        #[cfg(windows)]
        {
            let has_ext = Path::new(program)
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| {
                    extensions
                        .iter()
                        .any(|ext| ext.eq_ignore_ascii_case(&format!(".{e}")) || ext.eq_ignore_ascii_case(e))
                });
            if !has_ext {
                for ext in &extensions {
                    let candidate = Path::new(dir).join(format!("{program}{ext}"));
                    if candidate.is_file() {
                        return true;
                    }
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ENV_CLAUDE, ENV_CODEX, ENV_CURSOR};

    #[test]
    fn mock_ready() {
        let entry = diagnose_provider(ProviderId::Mock).expect("diagnose");
        assert_eq!(entry.status, DoctorStatus::Ready);
        assert!(entry.implemented);
        assert_eq!(entry.program, "mock");
        assert!(entry.reason.is_none());
        assert_eq!(entry.default_timeout_secs, 1200);
    }

    #[test]
    fn claude_missing_when_override_points_nowhere() {
        crate::with_env_lock(|| {
            std::env::set_var(
                ENV_CLAUDE,
                "dare-ai-doctor-missing-claude-xyzzy-9f3a2c1b",
            );
            let entry = diagnose_provider(ProviderId::ClaudeCode).expect("diagnose");
            std::env::remove_var(ENV_CLAUDE);
            assert!(entry.implemented);
            assert_eq!(entry.status, DoctorStatus::Missing);
            assert_ne!(entry.status, DoctorStatus::NotImplemented);
            assert!(entry
                .reason
                .as_deref()
                .unwrap_or("")
                .contains("provider executable not found:"));
        });
    }

    #[test]
    fn cursor_invalid_override() {
        crate::with_env_lock(|| {
            std::env::set_var(ENV_CURSOR, "   ");
            let entry = diagnose_provider(ProviderId::CursorCli).expect("diagnose");
            std::env::remove_var(ENV_CURSOR);
            assert_eq!(entry.status, DoctorStatus::Invalid);
            assert!(entry.implemented);
            assert_eq!(
                entry.reason.as_deref(),
                Some("command override must not be empty")
            );
        });
    }

    #[test]
    fn invalid_override() {
        crate::with_env_lock(|| {
            std::env::set_var(ENV_CODEX, "   ");
            let entry = diagnose_provider(ProviderId::Codex).expect("diagnose");
            std::env::remove_var(ENV_CODEX);
            assert_eq!(entry.status, DoctorStatus::Invalid);
            assert!(entry.implemented);
            assert_eq!(
                entry.reason.as_deref(),
                Some("command override must not be empty")
            );
        });
    }

    #[test]
    fn diagnose_all_order_stable() {
        crate::with_env_lock(|| {
            let report = diagnose_all().expect("diagnose_all");
            assert_eq!(report.schema_version, 1);
            assert!(report.ok);
            let ids: Vec<&str> = report.providers.iter().map(|p| p.id.as_str()).collect();
            assert_eq!(
                ids,
                vec![
                    "mock",
                    "codex",
                    "claude-code",
                    "cursor-cli",
                    "antigravity-cli"
                ]
            );
            assert_eq!(report.providers[0].status, DoctorStatus::Ready);
            for entry in &report.providers[2..] {
                assert!(entry.implemented);
                assert_ne!(entry.status, DoctorStatus::NotImplemented);
                assert!(matches!(
                    entry.status,
                    DoctorStatus::Ready | DoctorStatus::Missing | DoctorStatus::Invalid
                ));
            }
        });
    }

    #[test]
    fn doctor_status_serde_snake_case() {
        assert_eq!(
            serde_json::to_string(&DoctorStatus::NotImplemented).unwrap(),
            "\"not_implemented\""
        );
        assert_eq!(
            serde_json::to_string(&DoctorStatus::Ready).unwrap(),
            "\"ready\""
        );
    }

    #[test]
    fn codex_missing_when_override_points_nowhere() {
        crate::with_env_lock(|| {
            std::env::set_var(
                ENV_CODEX,
                "dare-ai-doctor-missing-bin-xyzzy-9f3a2c1b",
            );
            let entry = diagnose_provider(ProviderId::Codex).expect("diagnose");
            std::env::remove_var(ENV_CODEX);
            assert_eq!(entry.status, DoctorStatus::Missing);
            assert!(entry
                .reason
                .as_deref()
                .unwrap_or("")
                .contains("provider executable not found:"));
        });
    }
}
