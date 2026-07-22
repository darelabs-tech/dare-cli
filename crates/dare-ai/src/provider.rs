use dare_core::{CoreError, CoreResult};

use crate::codex::CodexCliProvider;
use crate::mock::MockProvider;
use crate::request::{EnrichRaw, EnrichRequest};

pub trait AiProvider: Send + Sync {
    fn id(&self) -> ProviderId;
    fn enrich(&self, req: &EnrichRequest) -> CoreResult<EnrichRaw>;
}

pub fn resolve_provider(id: ProviderId) -> CoreResult<Box<dyn AiProvider>> {
    match id {
        ProviderId::Mock => Ok(Box::new(MockProvider)),
        ProviderId::Codex => Ok(Box::new(CodexCliProvider::from_env()?)),
        ProviderId::ClaudeCode | ProviderId::CursorCli | ProviderId::AntigravityCli => Err(
            CoreError::invalid_input(format!("provider not implemented: {}", id.as_str())),
        ),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderId {
    Mock,
    Codex,
    ClaudeCode,
    CursorCli,
    AntigravityCli,
}

impl ProviderId {
    pub fn parse(s: &str) -> CoreResult<Self> {
        match s {
            "mock" => Ok(Self::Mock),
            "codex" => Ok(Self::Codex),
            "claude-code" => Ok(Self::ClaudeCode),
            "cursor-cli" => Ok(Self::CursorCli),
            "antigravity-cli" => Ok(Self::AntigravityCli),
            _ => Err(CoreError::invalid_input(format!("unknown provider: {s}"))),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Mock => "mock",
            Self::Codex => "codex",
            Self::ClaudeCode => "claude-code",
            Self::CursorCli => "cursor-cli",
            Self::AntigravityCli => "antigravity-cli",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dare_core::ErrorKind;

    #[test]
    fn provider_id_parse() {
        assert_eq!(ProviderId::parse("mock").unwrap(), ProviderId::Mock);
        assert_eq!(ProviderId::parse("codex").unwrap(), ProviderId::Codex);
        assert_eq!(
            ProviderId::parse("claude-code").unwrap(),
            ProviderId::ClaudeCode
        );
        assert_eq!(
            ProviderId::parse("cursor-cli").unwrap(),
            ProviderId::CursorCli
        );
        assert_eq!(
            ProviderId::parse("antigravity-cli").unwrap(),
            ProviderId::AntigravityCli
        );

        assert_eq!(ProviderId::Mock.as_str(), "mock");
        assert_eq!(ProviderId::Codex.as_str(), "codex");

        let err = ProviderId::parse("unknown-provider").unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidInput);
        assert!(err.to_string().contains("unknown provider"));
    }

    #[test]
    fn resolve_unimplemented_provider_errors() {
        for id in [
            ProviderId::ClaudeCode,
            ProviderId::CursorCli,
            ProviderId::AntigravityCli,
        ] {
            let err = resolve_provider(id).err().expect("expected resolve error");
            assert_eq!(err.kind(), ErrorKind::InvalidInput);
            assert!(err
                .to_string()
                .contains(&format!("provider not implemented: {}", id.as_str())));
        }
    }

    #[test]
    fn resolve_codex_provider_ok() {
        let provider = resolve_provider(ProviderId::Codex).unwrap();
        assert_eq!(provider.id(), ProviderId::Codex);
    }

    #[test]
    fn resolve_mock_provider_ok() {
        let provider = resolve_provider(ProviderId::Mock).unwrap();
        assert_eq!(provider.id(), ProviderId::Mock);
    }
}
