//! CLI and env override bags.

use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CliOverrides {
    pub ide: Option<String>,
    pub block_enabled: BTreeMap<String, bool>,
    pub extra_string: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct EnvOverrides {
    pub ide: Option<String>,
    pub block_enabled: BTreeMap<String, bool>,
    pub extra_string: BTreeMap<String, String>,
}
