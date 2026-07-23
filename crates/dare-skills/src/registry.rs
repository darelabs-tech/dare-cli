//! Skill registries: mock, local, remote, composite merge + topo resolve.

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::time::Duration;

use dare_contracts::{load_skills_manifest, SkillsManifest};
use dare_core::{CoreError, CoreResult, ProjectRoot, SafeRelativePath};
use serde::Deserialize;
use tracing::warn;

use crate::model::{
    classify_skill, validate_skill_id, validate_version_segment, RegistrySkill, SkillKind,
    SkillManifest, SkillSource,
};

pub const REMOTE_BASE_URL_DEFAULT: &str = "https://dare-registry.vercel.app";
pub const REMOTE_TIMEOUT_SECS: u64 = 3;
pub const ENV_LOCAL_REGISTRY: &str = "DARE_LOCAL_REGISTRY";
pub const ENV_REMOTE_REGISTRY: &str = "DARE_REMOTE_REGISTRY";
pub const PROJECT_SKILLS_REL: &str = ".dare/skills.yml";

const MOCK_JSON: &str = include_str!("../data/registry-mock.json");

#[derive(Debug, Deserialize)]
struct MockFile {
    skills: Vec<MockSkillRaw>,
}

#[derive(Debug, Deserialize)]
struct MockSkillRaw {
    name: String,
    version: String,
    description: String,
    author: String,
    license: String,
    #[serde(default)]
    dare_version: Option<String>,
    #[serde(default, alias = "dependsOn")]
    depends_on: Vec<String>,
}

impl MockSkillRaw {
    fn into_registry(self) -> RegistrySkill {
        let kind = classify_skill(&self.name);
        RegistrySkill {
            name: self.name,
            version: self.version,
            description: self.description,
            author: self.author,
            license: self.license,
            dare_version: self.dare_version,
            depends_on: self.depends_on,
            kind,
            source: SkillSource::Mock,
        }
    }
}

/// Embedded mock registry (offline guarantee).
#[derive(Debug, Clone, Default)]
pub struct MockRegistry;

impl MockRegistry {
    pub fn list(&self) -> CoreResult<Vec<RegistrySkill>> {
        let file: MockFile = serde_json::from_str(MOCK_JSON)
            .map_err(|e| CoreError::config(format!("invalid embedded registry-mock.json: {e}")))?;
        let mut skills: Vec<RegistrySkill> =
            file.skills.into_iter().map(|s| s.into_registry()).collect();
        skills.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(skills)
    }

    pub fn get(&self, name: &str) -> CoreResult<Option<RegistrySkill>> {
        validate_skill_id(name)?;
        Ok(self.list()?.into_iter().find(|s| s.name == name))
    }
}

/// Local filesystem registry under `~/.dare/registry` or `DARE_LOCAL_REGISTRY`.
#[derive(Debug, Clone)]
pub struct LocalRegistry {
    root: PathBuf,
}

impl LocalRegistry {
    pub fn from_env() -> Option<Self> {
        local_registry_root().map(|root| Self { root })
    }

    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn list(&self) -> CoreResult<Vec<RegistrySkill>> {
        if !self.root.is_dir() {
            return Ok(Vec::new());
        }
        let index_path = self.root.join("index.json");
        if index_path.is_file() {
            return self.list_from_index(&index_path);
        }
        self.list_from_scan()
    }

    pub fn get(&self, name: &str) -> CoreResult<Option<RegistrySkill>> {
        validate_skill_id(name)?;
        Ok(self.list()?.into_iter().find(|s| s.name == name))
    }

    fn list_from_index(&self, index_path: &Path) -> CoreResult<Vec<RegistrySkill>> {
        let text = std::fs::read_to_string(index_path).map_err(|e| CoreError::io(e.to_string()))?;
        let raw: Vec<MockSkillRaw> = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(e) => {
                warn!(error = %e, "local registry index.json parse failed");
                return Ok(Vec::new());
            }
        };
        let mut out = Vec::new();
        for item in raw {
            if validate_skill_id(&item.name).is_err() {
                continue;
            }
            let mut skill = item.into_registry();
            skill.source = SkillSource::Local;
            out.push(skill);
        }
        out.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(out)
    }

    fn list_from_scan(&self) -> CoreResult<Vec<RegistrySkill>> {
        let mut out = Vec::new();
        let entries = match std::fs::read_dir(&self.root) {
            Ok(e) => e,
            Err(_) => return Ok(Vec::new()),
        };
        for entry in entries.flatten() {
            let name_os = entry.file_name();
            let Some(name) = name_os.to_str() else {
                continue;
            };
            if name == "index.json" || validate_skill_id(name).is_err() {
                continue;
            }
            let skill_dir = entry.path();
            if !skill_dir.is_dir() {
                continue;
            }
            if let Some(skill) = self.load_latest_version(&skill_dir, name)? {
                out.push(skill);
            }
        }
        out.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(out)
    }

    fn load_latest_version(
        &self,
        skill_dir: &Path,
        name: &str,
    ) -> CoreResult<Option<RegistrySkill>> {
        let mut versions: Vec<PathBuf> = Vec::new();
        let Ok(entries) = std::fs::read_dir(skill_dir) else {
            return Ok(None);
        };
        for entry in entries.flatten() {
            let ver_os = entry.file_name();
            let Some(ver) = ver_os.to_str() else {
                continue;
            };
            if validate_version_segment(ver).is_err() {
                continue;
            }
            let p = entry.path();
            if p.is_dir() {
                versions.push(p);
            }
        }
        versions.sort_by(|a, b| {
            let va = a.file_name().and_then(|s| s.to_str()).unwrap_or("");
            let vb = b.file_name().and_then(|s| s.to_str()).unwrap_or("");
            va.cmp(vb)
        });
        let Some(latest) = versions.pop() else {
            return Ok(None);
        };
        let yml = latest.join("skill.yml");
        if !yml.is_file() {
            return Ok(None);
        }
        let text = match std::fs::read_to_string(&yml) {
            Ok(t) => t,
            Err(e) => {
                warn!(error = %e, path = %yml.display(), "skip local skill.yml");
                return Ok(None);
            }
        };
        let manifest: SkillManifest = match serde_yaml::from_str(&text) {
            Ok(m) => m,
            Err(e) => {
                warn!(error = %e, name, "skip invalid local skill.yml");
                return Ok(None);
            }
        };
        if manifest.name != name {
            warn!(expected = name, got = %manifest.name, "local skill name mismatch");
        }
        Ok(Some(manifest.into_registry_skill(SkillSource::Local)))
    }
}

/// HTTP GET abstraction (injectable for tests).
pub trait HttpGet: Send + Sync {
    fn get_text(&self, url: &str) -> Result<String, String>;
}

/// Default ureq client with 3s timeout.
#[derive(Debug, Clone)]
pub struct UreqHttpGet {
    timeout: Duration,
}

impl Default for UreqHttpGet {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(REMOTE_TIMEOUT_SECS),
        }
    }
}

impl HttpGet for UreqHttpGet {
    fn get_text(&self, url: &str) -> Result<String, String> {
        let agent = ureq::AgentBuilder::new().timeout(self.timeout).build();
        let resp = agent.get(url).call().map_err(|e| e.to_string())?;
        resp.into_string().map_err(|e| e.to_string())
    }
}

/// Always-failing client (tests / offline).
#[derive(Debug, Clone, Default)]
pub struct FailingHttpGet;

impl HttpGet for FailingHttpGet {
    fn get_text(&self, _url: &str) -> Result<String, String> {
        Err("remote unavailable".into())
    }
}

/// Remote registry (soft-fail: never hard-errors the command).
pub struct RemoteRegistry {
    base_url: String,
    http: Box<dyn HttpGet>,
}

impl std::fmt::Debug for RemoteRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RemoteRegistry")
            .field("base_url", &self.base_url)
            .finish_non_exhaustive()
    }
}

impl RemoteRegistry {
    pub fn from_env() -> Self {
        match std::env::var(ENV_REMOTE_REGISTRY) {
            Ok(v) if v.is_empty() || v.eq_ignore_ascii_case("off") => {
                Self::with_http(REMOTE_BASE_URL_DEFAULT, Box::new(FailingHttpGet))
            }
            Ok(url) => Self::with_http(url, Box::new(UreqHttpGet::default())),
            Err(_) => Self {
                base_url: REMOTE_BASE_URL_DEFAULT.to_string(),
                http: Box::new(UreqHttpGet::default()),
            },
        }
    }

    pub fn with_http(base_url: impl Into<String>, http: Box<dyn HttpGet>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            http,
        }
    }

    pub fn list(&self) -> CoreResult<Vec<RegistrySkill>> {
        let url = format!("{}/api/skills", self.base_url);
        match self.http.get_text(&url) {
            Ok(text) => Ok(parse_remote_list(&text)),
            Err(e) => {
                warn!(error = %e, "remote registry list failed; falling back");
                Ok(Vec::new())
            }
        }
    }

    pub fn get(&self, name: &str) -> CoreResult<Option<RegistrySkill>> {
        validate_skill_id(name)?;
        let url = format!("{}/api/skills/{name}", self.base_url);
        match self.http.get_text(&url) {
            Ok(text) => Ok(parse_remote_one(&text)),
            Err(e) => {
                warn!(error = %e, name, "remote registry get failed; falling back");
                Ok(None)
            }
        }
    }
}

fn parse_remote_list(text: &str) -> Vec<RegistrySkill> {
    #[derive(Deserialize)]
    struct Wrap {
        #[serde(default)]
        skills: Vec<MockSkillRaw>,
    }
    let parsed: Result<Wrap, _> = serde_json::from_str(text);
    let raw = match parsed {
        Ok(w) => w.skills,
        Err(_) => match serde_json::from_str::<Vec<MockSkillRaw>>(text) {
            Ok(v) => v,
            Err(e) => {
                warn!(error = %e, "remote registry list parse failed");
                return Vec::new();
            }
        },
    };
    let mut out: Vec<RegistrySkill> = raw
        .into_iter()
        .filter(|s| validate_skill_id(&s.name).is_ok())
        .map(|s| {
            let mut skill = s.into_registry();
            skill.source = SkillSource::Remote;
            skill
        })
        .collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

fn parse_remote_one(text: &str) -> Option<RegistrySkill> {
    let raw: MockSkillRaw = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(e) => {
            warn!(error = %e, "remote registry skill parse failed");
            return None;
        }
    };
    if validate_skill_id(&raw.name).is_err() {
        return None;
    }
    let mut skill = raw.into_registry();
    skill.source = SkillSource::Remote;
    Some(skill)
}

/// Merged view: remote > local > mock.
pub struct CompositeRegistry {
    mock: MockRegistry,
    local: Option<LocalRegistry>,
    remote: RemoteRegistry,
}

impl CompositeRegistry {
    pub fn from_env() -> Self {
        Self {
            mock: MockRegistry,
            local: LocalRegistry::from_env(),
            remote: RemoteRegistry::from_env(),
        }
    }

    /// Test/helper constructor.
    pub fn new(mock: MockRegistry, local: Option<LocalRegistry>, remote: RemoteRegistry) -> Self {
        Self {
            mock,
            local,
            remote,
        }
    }

    pub fn list(&self) -> CoreResult<Vec<RegistrySkill>> {
        let mut map: HashMap<String, RegistrySkill> = HashMap::new();
        // Lowest priority first so higher overwrites.
        for skill in self.mock.list()? {
            map.insert(skill.name.clone(), skill);
        }
        if let Some(local) = &self.local {
            for skill in local.list()? {
                map.insert(skill.name.clone(), skill);
            }
        }
        for skill in self.remote.list()? {
            map.insert(skill.name.clone(), skill);
        }
        let mut out: Vec<RegistrySkill> = map.into_values().collect();
        out.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(out)
    }

    pub fn get(&self, name: &str) -> CoreResult<Option<RegistrySkill>> {
        validate_skill_id(name)?;
        if let Some(s) = self.remote.get(name)? {
            return Ok(Some(s));
        }
        if let Some(local) = &self.local {
            if let Some(s) = local.get(name)? {
                return Ok(Some(s));
            }
        }
        self.mock.get(name)
    }
}

/// Load project `.dare/skills.yml`; missing file → empty manifest.
pub fn load_project_skills(root: &ProjectRoot) -> CoreResult<SkillsManifest> {
    let rel = SafeRelativePath::new(PROJECT_SKILLS_REL)?;
    let path = root.resolve(&rel)?;
    if !path.as_path().is_file() {
        return Ok(SkillsManifest::default());
    }
    load_skills_manifest(root, &rel)
}

/// Topological resolve of `roots` using `skills` catalog; detects cycles.
pub fn resolve_dependencies(
    catalog: &[RegistrySkill],
    roots: &[String],
) -> CoreResult<Vec<String>> {
    let by_name: HashMap<&str, &RegistrySkill> =
        catalog.iter().map(|s| (s.name.as_str(), s)).collect();

    let mut needed: HashSet<String> = HashSet::new();
    let mut stack: Vec<String> = roots.to_vec();
    while let Some(name) = stack.pop() {
        validate_skill_id(&name)?;
        if !needed.insert(name.clone()) {
            continue;
        }
        let Some(skill) = by_name.get(name.as_str()) else {
            return Err(CoreError::not_found(format!("skill not found: {name}")));
        };
        for dep in &skill.depends_on {
            validate_skill_id(dep)?;
            if !needed.contains(dep) {
                stack.push(dep.clone());
            }
        }
    }

    // Kahn on induced subgraph.
    let mut indegree: HashMap<&str, usize> = HashMap::new();
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
    for name in &needed {
        indegree.entry(name.as_str()).or_insert(0);
        let skill = by_name[name.as_str()];
        for dep in &skill.depends_on {
            if needed.contains(dep) {
                adj.entry(dep.as_str()).or_default().push(name.as_str());
                *indegree.entry(name.as_str()).or_insert(0) += 1;
            }
        }
    }

    let mut ready: Vec<&str> = indegree
        .iter()
        .filter(|(_, d)| **d == 0)
        .map(|(n, _)| *n)
        .collect();
    ready.sort();
    let mut queue: VecDeque<&str> = ready.into();
    let mut order: Vec<String> = Vec::new();

    while let Some(n) = queue.pop_front() {
        order.push(n.to_string());
        let mut next_ready = Vec::new();
        if let Some(children) = adj.get(n) {
            for child in children {
                if let Some(d) = indegree.get_mut(child) {
                    *d = d.saturating_sub(1);
                    if *d == 0 {
                        next_ready.push(*child);
                    }
                }
            }
        }
        next_ready.sort();
        for c in next_ready {
            queue.push_back(c);
        }
    }

    if order.len() != needed.len() {
        return Err(CoreError::invalid_input("dependency cycle detected"));
    }
    Ok(order)
}

fn local_registry_root() -> Option<PathBuf> {
    if let Ok(p) = std::env::var(ENV_LOCAL_REGISTRY) {
        let pb = PathBuf::from(p);
        if !pb.as_os_str().is_empty() {
            return Some(pb);
        }
    }
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))?;
    Some(PathBuf::from(home).join(".dare").join("registry"))
}

/// Helper: whether name is one of the six generics.
pub fn is_generic_skill(name: &str) -> bool {
    matches!(classify_skill(name), SkillKind::Generic)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct StaticHttpGet {
        body: Mutex<Option<String>>,
    }

    impl HttpGet for StaticHttpGet {
        fn get_text(&self, _url: &str) -> Result<String, String> {
            self.body
                .lock()
                .expect("lock")
                .clone()
                .ok_or_else(|| "empty".into())
        }
    }

    #[test]
    fn mock_has_seven_skills() {
        let list = MockRegistry.list().expect("list");
        assert_eq!(list.len(), 7);
        assert!(list.windows(2).all(|w| w[0].name <= w[1].name));
        let generics = list.iter().filter(|s| s.kind == SkillKind::Generic).count();
        assert_eq!(generics, 6);
        assert!(list.iter().any(|s| s.name == "skill-nestjs-api"
            && s.kind == SkillKind::Stack
            && s.source == SkillSource::Mock));
    }

    #[test]
    fn remote_fail_does_not_break_list() {
        let remote = RemoteRegistry::with_http("https://example.invalid", Box::new(FailingHttpGet));
        let reg = CompositeRegistry::new(MockRegistry, None, remote);
        let list = reg.list().expect("list");
        assert_eq!(list.len(), 7);
        assert!(list.iter().all(|s| s.source == SkillSource::Mock));
    }

    #[test]
    fn remote_wins_over_mock() {
        let body = r#"{"name":"dare-ax","version":"9.9.9","description":"remote","author":"x","license":"MIT","depends_on":[]}"#;
        let remote = RemoteRegistry::with_http(
            "https://example.test",
            Box::new(StaticHttpGet {
                body: Mutex::new(Some(body.to_string())),
            }),
        );
        let reg = CompositeRegistry::new(MockRegistry, None, remote);
        let ax = reg.get("dare-ax").expect("get").expect("some");
        assert_eq!(ax.version, "9.9.9");
        assert_eq!(ax.source, SkillSource::Remote);
    }

    #[test]
    fn local_wins_over_mock() {
        let dir = tempfile::tempdir().expect("tmp");
        let index = dir.path().join("index.json");
        std::fs::write(
            &index,
            r#"[{"name":"dare-ax","version":"2.0.0","description":"local","author":"x","license":"MIT","depends_on":[]}]"#,
        )
        .expect("write");
        let local = LocalRegistry::new(dir.path());
        let remote = RemoteRegistry::with_http("https://example.invalid", Box::new(FailingHttpGet));
        let reg = CompositeRegistry::new(MockRegistry, Some(local), remote);
        let ax = reg.get("dare-ax").expect("get").expect("some");
        assert_eq!(ax.version, "2.0.0");
        assert_eq!(ax.source, SkillSource::Local);
    }

    #[test]
    fn resolve_topo_puts_dare_ax_first() {
        let catalog = MockRegistry.list().expect("list");
        let order =
            resolve_dependencies(&catalog, &["dare-frontend-design".into()]).expect("resolve");
        assert_eq!(order.first().map(String::as_str), Some("dare-ax"));
        assert!(order.iter().any(|n| n == "dare-frontend-design"));
    }

    #[test]
    fn resolve_detects_cycle() {
        let catalog = vec![
            RegistrySkill {
                name: "a".into(),
                version: "1".into(),
                description: String::new(),
                author: String::new(),
                license: "MIT".into(),
                dare_version: None,
                depends_on: vec!["b".into()],
                kind: SkillKind::Stack,
                source: SkillSource::Mock,
            },
            RegistrySkill {
                name: "b".into(),
                version: "1".into(),
                description: String::new(),
                author: String::new(),
                license: "MIT".into(),
                dare_version: None,
                depends_on: vec!["a".into()],
                kind: SkillKind::Stack,
                source: SkillSource::Mock,
            },
        ];
        let err = resolve_dependencies(&catalog, &["a".into()]).expect_err("cycle");
        assert!(err.message().contains("cycle"));
    }

    #[test]
    fn load_project_skills_missing_ok() {
        let dir = tempfile::tempdir().expect("tmp");
        let root = ProjectRoot::new(dir.path()).expect("root");
        let m = load_project_skills(&root).expect("load");
        assert!(m.skills.is_empty());
    }
}
