//! Load and resolve steering candidates for list/show.

use std::fs;

use dare_core::{CoreError, CoreResult, ProjectRoot, SafeRelativePath};
use globset::Glob;

use crate::env_deny::is_env_excluded;
use crate::frontmatter::{parse_steering_markdown, ParseOutcome, ParsedSteering};
use crate::report::{
    SteeringBlock, SteeringListItem, SteeringListReport, SteeringShowReport,
};
use crate::{
    MSG_ENV_EXCLUDED, MSG_PATH_ESCAPE, PATTERNS_REL, PROJECT_DNA_REL, STEERING_DIR_REL,
    STEERING_LIST_SCHEMA, STEERING_SHOW_SCHEMA,
};

#[derive(Debug, Clone)]
struct Candidate {
    path: String,
    scope: String,
    glob: Option<String>,
    priority: i32,
    body: String,
}

/// List all steering sources under `root` (DNA, PATTERNS, `.dare/steering/*.md`).
pub fn list_steering(root: &ProjectRoot) -> CoreResult<SteeringListReport> {
    let (candidates, warnings) = load_candidates(root, false)?;
    let mut files: Vec<SteeringListItem> = candidates
        .into_iter()
        .map(|c| SteeringListItem {
            path: c.path,
            scope: c.scope,
            glob: c.glob,
            priority: c.priority,
        })
        .collect();
    sort_items(&mut files);
    Ok(SteeringListReport {
        schema_version: STEERING_LIST_SCHEMA,
        files,
        warnings,
    })
}

/// Show steering blocks applicable to `target_rel` under `root`.
pub fn show_steering(root: &ProjectRoot, target_rel: &str) -> CoreResult<SteeringShowReport> {
    let normalized = target_rel.replace('\\', "/");
    let safe = SafeRelativePath::new(&normalized)
        .map_err(|_| CoreError::invalid_input(MSG_PATH_ESCAPE))?;
    let target = safe.as_str().to_string();

    let basename = target
        .rsplit('/')
        .next()
        .unwrap_or(target.as_str());
    if is_env_excluded(basename) {
        return Err(CoreError::invalid_input(MSG_ENV_EXCLUDED));
    }

    let abs = root.resolve(&safe)?;
    if !abs.as_path().is_file() {
        return Err(CoreError::not_found(format!("file not found: {target}")));
    }

    let (candidates, _) = load_candidates(root, true)?;
    let mut blocks: Vec<SteeringBlock> = Vec::new();

    for c in candidates {
        match c.scope.as_str() {
            "project" => {
                blocks.push(SteeringBlock {
                    path: c.path,
                    scope: c.scope,
                    glob: c.glob,
                    priority: c.priority,
                    body: c.body,
                });
            }
            "glob" => {
                let Some(pattern) = c.glob.as_deref() else {
                    continue;
                };
                let Ok(glob) = Glob::new(pattern) else {
                    continue;
                };
                if glob.compile_matcher().is_match(&target) {
                    blocks.push(SteeringBlock {
                        path: c.path,
                        scope: c.scope,
                        glob: c.glob,
                        priority: c.priority,
                        body: c.body,
                    });
                }
            }
            _ => {}
        }
    }

    sort_blocks(&mut blocks);

    Ok(SteeringShowReport {
        schema_version: STEERING_SHOW_SCHEMA,
        target,
        blocks,
    })
}

fn load_candidates(
    root: &ProjectRoot,
    include_body: bool,
) -> CoreResult<(Vec<Candidate>, Vec<String>)> {
    let mut out: Vec<Candidate> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    try_push_base(
        root,
        PROJECT_DNA_REL,
        0,
        include_body,
        &mut out,
    )?;
    try_push_base(
        root,
        PATTERNS_REL,
        1,
        include_body,
        &mut out,
    )?;

    let steering_rel = SafeRelativePath::new(STEERING_DIR_REL)
        .map_err(|_| CoreError::invalid_input(MSG_PATH_ESCAPE))?;
    let steering_abs = root.resolve(&steering_rel)?;
    let steering_dir = steering_abs.as_path();
    if steering_dir.is_dir() {
        let mut entries: Vec<_> = fs::read_dir(steering_dir.as_std_path())
            .map_err(|e| CoreError::io(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| CoreError::io(e.to_string()))?;
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if !name_str.ends_with(".md") {
                continue;
            }
            if !entry
                .file_type()
                .map_err(|e| CoreError::io(e.to_string()))?
                .is_file()
            {
                continue;
            }
            let rel_path = format!("{STEERING_DIR_REL}/{name_str}").replace('\\', "/");
            let text = fs::read_to_string(entry.path())
                .map_err(|e| CoreError::io(e.to_string()))?;
            match parse_steering_markdown(&text) {
                ParseOutcome::Ok(parsed) => {
                    push_parsed(rel_path, parsed, include_body, &mut out);
                }
                ParseOutcome::Skip { warning } => {
                    warnings.push(format!("{rel_path}: {warning}"));
                }
            }
        }
    }

    Ok((out, warnings))
}

fn try_push_base(
    root: &ProjectRoot,
    rel: &str,
    priority: i32,
    include_body: bool,
    out: &mut Vec<Candidate>,
) -> CoreResult<()> {
    let safe = SafeRelativePath::new(rel)
        .map_err(|_| CoreError::invalid_input(MSG_PATH_ESCAPE))?;
    let abs = root.resolve(&safe)?;
    if !abs.as_path().is_file() {
        return Ok(());
    }
    let body = if include_body {
        fs::read_to_string(abs.as_path().as_std_path())
            .map_err(|e| CoreError::io(e.to_string()))?
    } else {
        String::new()
    };
    out.push(Candidate {
        path: rel.replace('\\', "/"),
        scope: "project".to_string(),
        glob: None,
        priority,
        body,
    });
    Ok(())
}

fn push_parsed(path: String, parsed: ParsedSteering, include_body: bool, out: &mut Vec<Candidate>) {
    out.push(Candidate {
        path,
        scope: parsed.scope,
        glob: parsed.glob,
        priority: parsed.priority,
        body: if include_body {
            parsed.body
        } else {
            String::new()
        },
    });
}

fn sort_items(files: &mut [SteeringListItem]) {
    files.sort_by(|a, b| {
        a.priority
            .cmp(&b.priority)
            .then_with(|| a.path.cmp(&b.path))
    });
}

fn sort_blocks(blocks: &mut [SteeringBlock]) {
    blocks.sort_by(|a, b| {
        a.priority
            .cmp(&b.priority)
            .then_with(|| a.path.cmp(&b.path))
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::Path;
    use tempfile::tempdir;

    fn write_file(root: &Path, rel: &str, contents: &str) {
        let path = root.join(rel.replace('/', std::path::MAIN_SEPARATOR_STR));
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let mut f = fs::File::create(&path).unwrap();
        f.write_all(contents.as_bytes()).unwrap();
    }

    #[test]
    fn priority_sort() {
        let dir = tempdir().unwrap();
        let root_path = dir.path();
        write_file(root_path, "DARE/PROJECT-DNA.md", "dna");
        write_file(root_path, "DARE/PATTERNS.md", "patterns");
        write_file(
            root_path,
            ".dare/steering/high.md",
            "---\nscope: project\npriority: 5\n---\nhigh\n",
        );
        write_file(
            root_path,
            ".dare/steering/low.md",
            "---\nscope: project\npriority: 50\n---\nlow\n",
        );
        write_file(
            root_path,
            ".dare/steering/tie-b.md",
            "---\nscope: project\npriority: 50\n---\ntie-b\n",
        );
        write_file(
            root_path,
            ".dare/steering/tie-a.md",
            "---\nscope: project\npriority: 50\n---\ntie-a\n",
        );

        let root = ProjectRoot::new(root_path).unwrap();
        let report = list_steering(&root).unwrap();
        let paths: Vec<&str> = report.files.iter().map(|f| f.path.as_str()).collect();
        assert_eq!(
            paths,
            vec![
                "DARE/PROJECT-DNA.md",
                "DARE/PATTERNS.md",
                ".dare/steering/high.md",
                ".dare/steering/low.md",
                ".dare/steering/tie-a.md",
                ".dare/steering/tie-b.md",
            ]
        );
        assert_eq!(report.files[0].priority, 0);
        assert_eq!(report.files[1].priority, 1);
        assert_eq!(report.files[2].priority, 5);
    }

    #[test]
    fn glob_match() {
        let dir = tempdir().unwrap();
        let root_path = dir.path();
        write_file(root_path, "crates/foo/src/lib.rs", "fn main() {}");
        write_file(root_path, "README.md", "# readme");
        write_file(
            root_path,
            ".dare/steering/rust.md",
            "---\nscope: glob\nglob: \"crates/**/*.rs\"\npriority: 10\n---\nrust rules\n",
        );
        write_file(
            root_path,
            ".dare/steering/all.md",
            "---\nscope: project\npriority: 20\n---\nalways\n",
        );

        let root = ProjectRoot::new(root_path).unwrap();
        let show_rs = show_steering(&root, "crates/foo/src/lib.rs").unwrap();
        let rs_paths: Vec<&str> = show_rs.blocks.iter().map(|b| b.path.as_str()).collect();
        assert!(rs_paths.contains(&".dare/steering/rust.md"));
        assert!(rs_paths.contains(&".dare/steering/all.md"));
        assert_eq!(show_rs.blocks[0].path, ".dare/steering/rust.md");

        let show_md = show_steering(&root, "README.md").unwrap();
        let md_paths: Vec<&str> = show_md.blocks.iter().map(|b| b.path.as_str()).collect();
        assert!(!md_paths.contains(&".dare/steering/rust.md"));
        assert!(md_paths.contains(&".dare/steering/all.md"));
    }

    #[test]
    fn show_excludes_env() {
        let dir = tempdir().unwrap();
        let root_path = dir.path();
        write_file(root_path, ".env", "SECRET=do-not-read");
        write_file(
            root_path,
            ".dare/steering/base.md",
            "---\nscope: project\n---\nok\n",
        );

        let root = ProjectRoot::new(root_path).unwrap();
        let err = show_steering(&root, ".env").unwrap_err();
        assert!(matches!(err, CoreError::InvalidInput(_)), "{err:?}");
        assert_eq!(err.message(), MSG_ENV_EXCLUDED);
    }
}
