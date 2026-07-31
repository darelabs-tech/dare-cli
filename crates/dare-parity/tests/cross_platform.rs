//! Cross-platform path cases: separators and drive-letter casing.
//!
//! Fixtures live under repo-root `tests/cross-platform/windows-path-cases/`.

use std::path::PathBuf;

use dare_core::{SafeRelativePath, PATH_ESCAPE_MSG};
use dare_parity::{load_case, CaseSpec};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

fn windows_path_cases_dir() -> PathBuf {
    repo_root().join("tests/cross-platform/windows-path-cases")
}

#[test]
fn backslash_separator_normalizes_to_slash() {
    let ok = SafeRelativePath::new(r"foo\bar\baz").expect("relative with backslash");
    assert_eq!(ok.as_str(), "foo/bar/baz");
}

#[test]
fn forward_slash_relative_ok() {
    let ok = SafeRelativePath::new("foo/bar").expect("posix relative");
    assert_eq!(ok.as_str(), "foo/bar");
}

#[test]
fn drive_letter_upper_and_lower_rejected() {
    for raw in [r"C:\Windows", r"c:\Windows", r"D:/temp", r"d:/temp"] {
        let err = SafeRelativePath::new(raw).expect_err("drive letter must be rejected");
        assert!(
            err.message().contains(PATH_ESCAPE_MSG),
            "raw={raw:?} err={err:?}"
        );
    }
}

#[test]
fn unc_and_rooted_backslash_rejected() {
    for raw in [r"\\server\share", r"\abs", "/abs"] {
        assert!(
            SafeRelativePath::new(raw).is_err(),
            "must reject {raw:?}"
        );
    }
}

#[test]
fn windows_path_cases_yaml_loads() {
    let dir = windows_path_cases_dir();
    let spec = load_case(&dir).expect("windows-path-cases case.yaml");
    assert_eq!(spec.id, "xplat.windows.path-cases");
    assert!(spec.axes.contains(&dare_parity::CompareAxis::Exit));
}

#[test]
fn malformed_yaml_bytes_are_err_not_panic() {
    let junk: &[u8] = &[0xff, 0xfe, 0x00, b'{', b'}'];
    let err = CaseSpec::try_from_yaml_bytes(junk).expect_err("malformed → Err");
    let _ = format!("{err}");
}

#[cfg(windows)]
#[test]
fn windows_mixed_separators_normalize() {
    let ok = SafeRelativePath::new(r"a\b/c\d").expect("mixed separators");
    assert_eq!(ok.as_str(), "a/b/c/d");
}

#[cfg(unix)]
#[test]
fn unix_backslash_still_normalizes_as_separator() {
    // SafeRelativePath treats `\` as a separator on all platforms (N-05).
    let ok = SafeRelativePath::new(r"a\b").expect("backslash on unix");
    assert_eq!(ok.as_str(), "a/b");
}
