//! Anti-tamper: detect removal of tests / asserts from a unified diff.

use crate::report::{AdvancedAspect, AspectResult, AspectStatus};

/// Patterns watched on changed lines (documented heuristics, §0.9):
/// - `#[test]` — primary gate: net removal with **zero** test additions → fail `removed_tests`
/// - `assert!` / `assert_eq!` / `assert_ne!` — counted for diagnostics; excess net removal
///   of asserts alone does **not** fail in this phase (informational via stderr_tail)
/// - case-insensitive mentions of `dare review` / `ralph` are noted in comments for future
///   soft checks; not hard-fail here beyond the `#[test]` rule above
///
/// Counts only hunk body lines (`+` / `-`), ignoring file headers (`+++` / `---`).
pub fn check_anti_tamper(unified_diff: &str) -> AspectResult {
    let mut test_added = 0i64;
    let mut test_removed = 0i64;
    let mut assert_added = 0i64;
    let mut assert_removed = 0i64;

    for line in unified_diff.lines() {
        if line.starts_with("+++") || line.starts_with("---") {
            continue;
        }
        let (is_add, is_del, body) = if let Some(rest) = line.strip_prefix('+') {
            (true, false, rest)
        } else if let Some(rest) = line.strip_prefix('-') {
            (false, true, rest)
        } else {
            continue;
        };

        if body_has_test_attr(body) {
            if is_add {
                test_added += 1;
            } else if is_del {
                test_removed += 1;
            }
        }
        if body_has_assert(body) {
            if is_add {
                assert_added += 1;
            } else if is_del {
                assert_removed += 1;
            }
        }
    }

    let net_test_removal = test_removed - test_added;
    let diag = format!(
        "tests +{test_added}/-{test_removed}; asserts +{assert_added}/-{assert_removed}"
    );

    // §0.9: remoção líquida de #[test] > 0 e zero adições de test → removed_tests
    if net_test_removal > 0 && test_added == 0 {
        return AspectResult {
            aspect: AdvancedAspect::AntiTamper,
            status: AspectStatus::Fail,
            score: None,
            reason: Some("removed_tests".into()),
            exit_code: Some(1),
            duration_ms: 0,
            stdout_tail: String::new(),
            stderr_tail: diag,
        };
    }

    AspectResult {
        aspect: AdvancedAspect::AntiTamper,
        status: AspectStatus::Pass,
        score: None,
        reason: None,
        exit_code: Some(0),
        duration_ms: 0,
        stdout_tail: String::new(),
        stderr_tail: diag,
    }
}

fn body_has_test_attr(body: &str) -> bool {
    // Match #[test] as in Rust source (case-sensitive attr name).
    body.contains("#[test]")
}

fn body_has_assert(body: &str) -> bool {
    let lower = body.to_ascii_lowercase();
    // (?i)assert!|assert_eq!|assert_ne! — substring heuristics
    lower.contains("assert!(")
        || lower.contains("assert_eq!(")
        || lower.contains("assert_ne!(")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anti_tamper_removed_tests() {
        let diff = "\
diff --git a/src/lib.rs b/src/lib.rs
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,8 +1,3 @@
-#[test]
-fn important() {
-    assert_eq!(1, 1);
-}
 fn keep() {}
";
        let r = check_anti_tamper(diff);
        assert_eq!(r.aspect, AdvancedAspect::AntiTamper);
        assert_eq!(r.status, AspectStatus::Fail);
        assert_eq!(r.reason.as_deref(), Some("removed_tests"));
    }

    #[test]
    fn anti_tamper_pass_when_test_replaced() {
        let diff = "\
--- a/t.rs
+++ b/t.rs
@@ -1,4 +1,4 @@
-#[test]
-fn old() {}
+#[test]
+fn new() {}
";
        let r = check_anti_tamper(diff);
        assert_eq!(r.status, AspectStatus::Pass);
        assert!(r.reason.is_none());
    }

    #[test]
    fn anti_tamper_pass_clean_diff() {
        let diff = "\
--- a/a.rs
+++ b/a.rs
@@ -1 +1 @@
-let x = 1;
+let x = 2;
";
        let r = check_anti_tamper(diff);
        assert_eq!(r.status, AspectStatus::Pass);
    }
}
