//! Archive traversal fixtures — call `dare_skills::extract_archive_safe` only.

use std::fs;
use std::path::Path;

use dare_core::{CoreError, CoreResult};
use dare_skills::extract_archive_safe;

const ZIP_SLIP: &str = "zip-slip.zip";
const TAR_SLIP: &str = "tar-slip.tar.gz";

/// Extract zip-slip / tar-slip fixtures from `dir`; both MUST return `Err`,
/// and the destination must not contain escaped `../` entries.
pub fn test_archive_traversal_fixtures(dir: &Path) -> CoreResult<()> {
    for name in [ZIP_SLIP, TAR_SLIP] {
        let archive = dir.join(name);
        if !archive.is_file() {
            return Err(CoreError::not_found(format!(
                "missing archive fixture: {}",
                archive.display()
            )));
        }

        let dest = dir.join(format!("out-{name}"));
        if dest.exists() {
            fs::remove_dir_all(&dest).map_err(|e| CoreError::io(e.to_string()))?;
        }
        fs::create_dir_all(&dest).map_err(|e| CoreError::io(e.to_string()))?;

        let result = extract_archive_safe(&archive, &dest);
        if result.is_ok() {
            return Err(CoreError::guard_fail(format!(
                "archive traversal fixture {name} must fail extract_archive_safe"
            )));
        }

        // Destination must not materialize escaped parent paths.
        if dest.join("evil.txt").is_file() {
            // file landed inside dest under a sanitized name — still check no ../
        }
        assert_no_parent_escape(&dest)?;
    }
    Ok(())
}

fn assert_no_parent_escape(dest: &Path) -> CoreResult<()> {
    if !dest.exists() {
        return Ok(());
    }
    for entry in walkdir_shallow(dest)? {
        let rel = entry
            .strip_prefix(dest)
            .unwrap_or(entry.as_path());
        let s = rel.to_string_lossy();
        if s.contains("..") {
            return Err(CoreError::guard_fail(format!(
                "archive dest contains parent escape path: {s}"
            )));
        }
    }
    // Also ensure nothing was written *outside* dest as a sibling evil.txt
    if let Some(parent) = dest.parent() {
        let outside = parent.join("evil.txt");
        if outside.is_file() {
            return Err(CoreError::guard_fail(
                "archive traversal wrote outside destination (evil.txt)",
            ));
        }
    }
    Ok(())
}

fn walkdir_shallow(root: &Path) -> CoreResult<Vec<std::path::PathBuf>> {
    let mut out = Vec::new();
    fn walk(dir: &Path, acc: &mut Vec<std::path::PathBuf>) -> CoreResult<()> {
        let rd = fs::read_dir(dir).map_err(|e| CoreError::io(e.to_string()))?;
        for ent in rd {
            let ent = ent.map_err(|e| CoreError::io(e.to_string()))?;
            let path = ent.path();
            acc.push(path.clone());
            if path.is_dir() {
                walk(&path, acc)?;
            }
        }
        Ok(())
    }
    walk(root, &mut out)?;
    Ok(out)
}
