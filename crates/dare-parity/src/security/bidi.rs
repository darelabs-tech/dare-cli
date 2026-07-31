//! Unicode bidi / path-escape rejection via `SafeRelativePath`.

use dare_core::{CoreError, CoreResult, ProjectRoot, SafeRelativePath, PATH_ESCAPE_MSG};

/// Assert that paths containing U+202E (RLO) together with path-escape (`..`) are rejected.
pub fn test_bidi_path_rejected(_root: &ProjectRoot) -> CoreResult<()> {
    let rlo = '\u{202E}';
    // All candidates embed U+202E and a `..` component that SafeRelativePath must reject.
    let candidates = [
        format!("safe/{rlo}/../escape"),
        format!("../{rlo}evil"),
        format!("foo/{rlo}bar/../../x"),
        format!("subdir/{rlo}/../outside"),
    ];

    for raw in &candidates {
        match SafeRelativePath::new(raw) {
            Err(e) => {
                if !matches!(e, CoreError::InvalidInput(_)) {
                    return Err(CoreError::internal(format!(
                        "bidi path {raw:?} rejected with unexpected kind: {e}"
                    )));
                }
                if !e.message().contains(PATH_ESCAPE_MSG) {
                    return Err(CoreError::internal(format!(
                        "bidi path {raw:?} expected PATH_ESCAPE_MSG, got {}",
                        e.message()
                    )));
                }
            }
            Ok(_) => {
                return Err(CoreError::guard_fail(format!(
                    "SafeRelativePath::new with U+202E and .. must return Err: {raw:?}"
                )));
            }
        }
    }

    Ok(())
}
