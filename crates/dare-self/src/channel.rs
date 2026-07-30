//! Release channel for `dare self update`.

use thiserror::Error;

/// Default channel when the user omits `--channel` (product still prerelease).
pub const DEFAULT_CHANNEL: Channel = Channel::Beta;

/// Self-update release channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Channel {
    Beta,
    Stable,
}

/// Invalid channel string (only `beta` / `stable` after lowercase normalize).
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("unknown channel `{0}`; expected beta or stable")]
pub struct ChannelParseError(pub String);

impl Channel {
    /// Parse a channel name. Input is lowercased; only `beta` and `stable` are accepted.
    pub fn parse(s: &str) -> Result<Self, ChannelParseError> {
        match s.to_ascii_lowercase().as_str() {
            "beta" => Ok(Channel::Beta),
            "stable" => Ok(Channel::Stable),
            other => Err(ChannelParseError(other.to_string())),
        }
    }

    /// Canonical lowercase wire/CLI name.
    pub fn as_str(self) -> &'static str {
        match self {
            Channel::Beta => "beta",
            Channel::Stable => "stable",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_parse_ok() {
        assert_eq!(Channel::parse("beta").unwrap(), Channel::Beta);
        assert_eq!(Channel::parse("stable").unwrap(), Channel::Stable);
        assert_eq!(Channel::parse("Beta").unwrap(), Channel::Beta);
        assert_eq!(Channel::parse("STABLE").unwrap(), Channel::Stable);
        assert_eq!(DEFAULT_CHANNEL, Channel::Beta);
        assert_eq!(DEFAULT_CHANNEL.as_str(), "beta");
    }

    #[test]
    fn channel_parse_bad() {
        assert!(Channel::parse("alpha").is_err());
        assert!(Channel::parse("").is_err());
        assert!(Channel::parse("beta ").is_err());
        assert!(Channel::parse("nightly").is_err());
    }
}
