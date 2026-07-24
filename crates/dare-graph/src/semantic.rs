//! Optional MiniLM embeddings (feature `semantic`) + doctor report (always).
//!
//! Cache: `{home}/.dare/models/all-minilm-l6-v2`. No graph.db schema changes.

#[cfg(feature = "semantic")]
use std::io::{self, IsTerminal, Write};
use std::path::{Component, Path, PathBuf};

use dare_core::{CoreError, CoreResult};
use serde::Serialize;

/// Cache directory segment under `{home}/.dare/models/`.
pub const SEMANTIC_MODEL_ID: &str = "all-minilm-l6-v2";
/// Human-facing model id (HuggingFace / display).
pub const SEMANTIC_MODEL_DISPLAY: &str = "all-MiniLM-L6-v2";
/// Embedding dimensionality for MiniLM-L6-v2.
pub const EMBED_DIM: usize = 384;
/// Cap on vector-rank candidates (union keyword ∪ BFS).
pub const MAX_CANDIDATES: usize = 512;
/// Max Unicode scalar chars for query text.
pub const MAX_QUERY_CHARS: usize = 8192;
/// Max Unicode scalar chars for passage text.
pub const MAX_PASSAGE_CHARS: usize = 2048;
/// Approximate model download size (~22 MB) for confirm UX.
pub const EXPECTED_MODEL_BYTES: u64 = 22_000_000;
/// Relative models root under home.
pub const MODELS_DIR_REL: &str = ".dare/models";
/// Prefix for soft-fail warnings (wired in mp042-003).
pub const MSG_SEMANTIC_UNAVAILABLE: &str = "semantic unavailable: ";
/// Typed cancel message for CLI (`enable` → exit 0).
pub const MSG_DOWNLOAD_CANCELLED: &str = "download cancelled";
/// Env that skips interactive confirm (equiv. `--yes`).
pub const ENV_SEMANTIC_YES: &str = "DARE_GRAPH_SEMANTIC_YES";
/// Env consumed by fastembed for cache location.
pub const ENV_FASTEMBED_CACHE_PATH: &str = "FASTEMBED_CACHE_PATH";

/// HTTPS hosts allowed for model download (RS-03).
pub const ALLOWLIST_HOSTS: &[&str] = &[
    "huggingface.co",
    "cdn-lfs.huggingface.co",
    "cdn-lfs-us-1.huggingface.co",
];

/// Options for `ensure_model` (feature `semantic`).
#[cfg(feature = "semantic")]
#[derive(Debug, Clone)]
pub struct SemanticOptions {
    /// Skip TTY confirm / allow non-TTY download.
    pub yes: bool,
    /// Candidate cap for vector path (clamped 1..=MAX_CANDIDATES).
    pub max_candidates: usize,
}

#[cfg(feature = "semantic")]
impl Default for SemanticOptions {
    fn default() -> Self {
        Self {
            yes: false,
            max_candidates: MAX_CANDIDATES,
        }
    }
}

#[cfg(feature = "semantic")]
impl SemanticOptions {
    /// Clamp `max_candidates` into `1..=MAX_CANDIDATES`.
    pub fn clamped(self) -> Self {
        Self {
            yes: self.yes,
            max_candidates: self.max_candidates.clamp(1, MAX_CANDIDATES),
        }
    }
}

/// Opaque MiniLM handle (single-threaded CLI use).
#[cfg(feature = "semantic")]
pub struct ModelHandle {
    inner: std::cell::RefCell<fastembed::TextEmbedding>,
}

/// Doctor JSON report (camelCase) — available without feature.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SemanticDoctorReport {
    pub semantic_compiled: bool,
    pub model_id: String,
    pub embed_dim: u32,
    pub cache_dir: String,
    pub model_present: bool,
    pub expected_bytes: u64,
    pub allowlist_hosts: Vec<String>,
    pub warnings: Vec<String>,
}

/// Resolve user home (`HOME` / `USERPROFILE`).
fn home_dir() -> CoreResult<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty())
        .ok_or_else(|| CoreError::invalid_input("home directory not found"))
}

/// Reject `..` / empty / separator-bearing model id segments (path jail).
pub fn validate_model_id_segment(model_id: &str) -> CoreResult<()> {
    if model_id.is_empty()
        || model_id.contains("..")
        || model_id.contains('/')
        || model_id.contains('\\')
        || model_id.contains('\0')
    {
        return Err(CoreError::invalid_input(
            "model cache path escapes jail or contains invalid characters",
        ));
    }
    let as_path = Path::new(model_id);
    if as_path.is_absolute() {
        return Err(CoreError::invalid_input(
            "model cache path escapes jail or contains invalid characters",
        ));
    }
    for c in as_path.components() {
        match c {
            Component::Normal(_) => {}
            Component::CurDir => {}
            Component::ParentDir
            | Component::RootDir
            | Component::Prefix(_) => {
                return Err(CoreError::invalid_input(
                    "model cache path escapes jail or contains invalid characters",
                ));
            }
        }
    }
    Ok(())
}

/// Build `{home}/.dare/models/{model_id}` and enforce jail under `{home}/.dare/models`.
pub fn resolve_model_cache_dir(home: &Path, model_id: &str) -> CoreResult<PathBuf> {
    validate_model_id_segment(model_id)?;
    let models_root = home.join(".dare").join("models");
    let cache = models_root.join(model_id);
    ensure_under_models_jail(&models_root, &cache)?;
    Ok(cache)
}

fn ensure_under_models_jail(models_root: &Path, candidate: &Path) -> CoreResult<()> {
    // Lexical jail: candidate must equal root or be a descendant (no `..` components).
    for c in candidate.components() {
        if matches!(c, Component::ParentDir) {
            return Err(CoreError::invalid_input(
                "model cache path escapes jail or contains invalid characters",
            ));
        }
    }
    if candidate == models_root {
        return Ok(());
    }
    let mut prefix = models_root.components();
    let mut cand = candidate.components();
    loop {
        match (prefix.next(), cand.next()) {
            (None, None) => return Ok(()),
            (None, Some(_)) => return Ok(()), // candidate longer — under root
            (Some(_), None) => {
                return Err(CoreError::invalid_input(
                    "model cache path escapes jail or contains invalid characters",
                ));
            }
            (Some(a), Some(b)) if a == b => continue,
            (Some(_), Some(_)) => {
                return Err(CoreError::invalid_input(
                    "model cache path escapes jail or contains invalid characters",
                ));
            }
        }
    }
}

fn cache_dir_for_doctor() -> String {
    match home_dir().and_then(|h| resolve_model_cache_dir(&h, SEMANTIC_MODEL_ID)) {
        Ok(p) => p.display().to_string(),
        Err(_) => format!("~/{MODELS_DIR_REL}/{SEMANTIC_MODEL_ID}"),
    }
}

fn dir_has_onnx(dir: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if dir_has_onnx(&path) {
                return true;
            }
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("onnx"))
        {
            return true;
        }
    }
    false
}

/// Whether the MiniLM cache directory already contains ONNX weights.
pub fn model_files_present(cache_dir: &Path) -> bool {
    cache_dir.is_dir() && dir_has_onnx(cache_dir)
}

/// Doctor snapshot — works with or without feature `semantic`.
pub fn semantic_doctor() -> SemanticDoctorReport {
    let cache_dir = cache_dir_for_doctor();
    let model_present = Path::new(&cache_dir).is_dir() && model_files_present(Path::new(&cache_dir));
    #[cfg_attr(feature = "semantic", allow(unused_mut))]
    let mut warnings = Vec::new();
    #[cfg(not(feature = "semantic"))]
    {
        warnings.push("semantic feature not compiled into this binary".to_string());
    }
    SemanticDoctorReport {
        semantic_compiled: cfg!(feature = "semantic"),
        model_id: SEMANTIC_MODEL_DISPLAY.to_string(),
        embed_dim: EMBED_DIM as u32,
        cache_dir,
        model_present,
        expected_bytes: EXPECTED_MODEL_BYTES,
        allowlist_hosts: ALLOWLIST_HOSTS.iter().map(|s| (*s).to_string()).collect(),
        warnings,
    }
}

#[cfg(feature = "semantic")]
fn env_semantic_yes() -> bool {
    matches!(
        std::env::var(ENV_SEMANTIC_YES).as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE") | Ok("yes") | Ok("YES")
    )
}

#[cfg(feature = "semantic")]
fn confirm_download(opts: &SemanticOptions) -> CoreResult<()> {
    let hosts = ALLOWLIST_HOSTS.join(", ");
    let msg = format!(
        "Download embedding model {SEMANTIC_MODEL_DISPLAY} (~{} MB)?\n\
         Allowlist hosts: {hosts}\n\
         Expected size: ~{EXPECTED_MODEL_BYTES} bytes\n\
         Proceed [y/N]? ",
        EXPECTED_MODEL_BYTES / 1_000_000
    );

    if opts.yes || env_semantic_yes() {
        return Ok(());
    }

    let stdin = io::stdin();
    if !stdin.is_terminal() {
        return Err(CoreError::invalid_input(
            "non-TTY: pass --yes or set DARE_GRAPH_SEMANTIC_YES=1 to download the model",
        ));
    }

    {
        let mut stdout = io::stdout();
        write!(stdout, "{msg}").map_err(|e| CoreError::io(e.to_string()))?;
        stdout.flush().map_err(|e| CoreError::io(e.to_string()))?;
    }

    let mut line = String::new();
    stdin
        .read_line(&mut line)
        .map_err(|e| CoreError::io(e.to_string()))?;
    let answer = line.trim();
    if answer.eq_ignore_ascii_case("y") || answer.eq_ignore_ascii_case("yes") {
        Ok(())
    } else {
        Err(CoreError::invalid_input(MSG_DOWNLOAD_CANCELLED))
    }
}

#[cfg(feature = "semantic")]
fn init_embedding(cache_dir: &Path) -> CoreResult<fastembed::TextEmbedding> {
    // Prefer quantized weights when the fastembed API exposes AllMiniLML6V2Q.
    let model = fastembed::EmbeddingModel::AllMiniLML6V2Q;
    let options = fastembed::InitOptions::new(model)
        .with_cache_dir(cache_dir.to_path_buf())
        .with_show_download_progress(false);
    fastembed::TextEmbedding::try_new(options).map_err(|e| {
        CoreError::io(format!(
            "{MSG_SEMANTIC_UNAVAILABLE}failed to init model: {e}"
        ))
    })
}

/// `{home}/.dare/models/all-minilm-l6-v2` — creates parents if needed.
#[cfg(feature = "semantic")]
pub fn models_cache_dir() -> CoreResult<PathBuf> {
    let home = home_dir()?;
    let cache = resolve_model_cache_dir(&home, SEMANTIC_MODEL_ID)?;
    std::fs::create_dir_all(&cache).map_err(|e| CoreError::io(e.to_string()))?;
    Ok(cache)
}

/// True when ONNX weights already exist under the cache dir.
#[cfg(feature = "semantic")]
pub fn model_is_cached() -> bool {
    match models_cache_dir() {
        Ok(dir) => model_files_present(&dir),
        Err(_) => false,
    }
}

/// Confirm (if needed) + download/init MiniLM. Idempotent when already cached.
#[cfg(feature = "semantic")]
pub fn ensure_model(opts: &SemanticOptions) -> CoreResult<ModelHandle> {
    let opts = opts.clone().clamped();
    let cache = models_cache_dir()?;
    let cached = model_files_present(&cache);

    if !cached {
        confirm_download(&opts)?;
    }

    // SAFETY: process-local cache path for fastembed; value is a jail-checked path.
    std::env::set_var(ENV_FASTEMBED_CACHE_PATH, &cache);

    let inner = init_embedding(&cache)?;
    Ok(ModelHandle {
        inner: std::cell::RefCell::new(inner),
    })
}

/// Embed texts → vectors of length [`EMBED_DIM`].
#[cfg(feature = "semantic")]
pub fn embed_texts(handle: &ModelHandle, texts: &[String]) -> CoreResult<Vec<Vec<f32>>> {
    if texts.is_empty() {
        return Ok(Vec::new());
    }
    let embeddings = handle
        .inner
        .try_borrow_mut()
        .map_err(|_| CoreError::internal("ModelHandle already borrowed"))?
        .embed(texts.to_vec(), None)
        .map_err(|e| CoreError::io(format!("{MSG_SEMANTIC_UNAVAILABLE}embed failed: {e}")))?;
    for (i, emb) in embeddings.iter().enumerate() {
        if emb.len() != EMBED_DIM {
            return Err(CoreError::internal(format!(
                "embedding dim mismatch at index {i}: got {}, expected {EMBED_DIM}",
                emb.len()
            )));
        }
    }
    Ok(embeddings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doctor_compiled_flag_matches_cfg() {
        let report = semantic_doctor();
        assert_eq!(report.semantic_compiled, cfg!(feature = "semantic"));
        assert_eq!(report.model_id, SEMANTIC_MODEL_DISPLAY);
        assert_eq!(report.embed_dim, EMBED_DIM as u32);
        assert_eq!(report.expected_bytes, EXPECTED_MODEL_BYTES);
        assert_eq!(report.allowlist_hosts.len(), ALLOWLIST_HOSTS.len());
        let cache = report.cache_dir.replace('\\', "/");
        assert!(
            cache.ends_with(".dare/models/all-minilm-l6-v2"),
            "cache_dir={cache}"
        );
    }

    #[test]
    fn cache_path_joins_under_dare_models() {
        let home = Path::new("/tmp/fake-home");
        let dir = resolve_model_cache_dir(home, SEMANTIC_MODEL_ID).expect("ok");
        let s = dir.to_string_lossy().replace('\\', "/");
        assert!(
            s.ends_with(".dare/models/all-minilm-l6-v2"),
            "got {s}"
        );
    }

    #[test]
    fn reject_dotdot_model_id() {
        let home = Path::new("/tmp/fake-home");
        let err = resolve_model_cache_dir(home, "../escape").unwrap_err();
        assert!(matches!(err, CoreError::InvalidInput(_)), "{err:?}");
        let err2 = validate_model_id_segment("..").unwrap_err();
        assert!(matches!(err2, CoreError::InvalidInput(_)));
        let err3 = validate_model_id_segment("foo/../bar").unwrap_err();
        assert!(matches!(err3, CoreError::InvalidInput(_)));
    }

    #[test]
    fn reject_separator_and_absolute_segments() {
        assert!(validate_model_id_segment("a/b").is_err());
        assert!(validate_model_id_segment("a\\b").is_err());
    }

    #[cfg(feature = "semantic")]
    #[test]
    fn semantic_options_clamp_candidates() {
        let o = SemanticOptions {
            yes: true,
            max_candidates: 0,
        }
        .clamped();
        assert_eq!(o.max_candidates, 1);
        let o2 = SemanticOptions {
            yes: false,
            max_candidates: 10_000,
        }
        .clamped();
        assert_eq!(o2.max_candidates, MAX_CANDIDATES);
    }

    #[cfg(feature = "semantic")]
    #[test]
    fn models_cache_dir_creates_and_ends_with_model_id() {
        let dir = models_cache_dir().expect("cache dir");
        let s = dir.to_string_lossy().replace('\\', "/");
        assert!(s.ends_with(".dare/models/all-minilm-l6-v2"), "got {s}");
        assert!(dir.is_dir());
    }

    /// Network: downloads MiniLM via fastembed. Opt-in only (`cargo test -- --ignored`).
    #[cfg(feature = "semantic")]
    #[test]
    #[ignore = "network: downloads model from HuggingFace"]
    fn ensure_model_network_download() {
        let opts = SemanticOptions {
            yes: true,
            max_candidates: MAX_CANDIDATES,
        };
        let handle = ensure_model(&opts).expect("ensure_model");
        let vecs = embed_texts(&handle, &[String::from("hello world")]).expect("embed");
        assert_eq!(vecs.len(), 1);
        assert_eq!(vecs[0].len(), EMBED_DIM);
    }
}
