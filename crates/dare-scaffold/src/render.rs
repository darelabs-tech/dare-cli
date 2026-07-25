//! Template substitution and secret scan (BLUEPRINT-046 §0.1 / mp046-002).

use dare_core::{CoreError, CoreResult};

const MSG_SECRET: &str = "template contains forbidden secret pattern";

const SECRET_NEEDLES: &[&str] = &["password=", "api_key=", "BEGIN PRIVATE KEY"];

/// Replace only `{{project_name}}` and `{{stack_id}}`, then scan for secrets.
pub fn render_template(text: &str, project_name: &str, stack_id: &str) -> CoreResult<String> {
    let rendered = text
        .replace("{{project_name}}", project_name)
        .replace("{{stack_id}}", stack_id);
    scan_secrets(&rendered)?;
    Ok(rendered)
}

/// Case-insensitive contains check for forbidden secret needles.
pub fn scan_secrets(text: &str) -> CoreResult<()> {
    let lower = text.to_ascii_lowercase();
    for needle in SECRET_NEEDLES {
        let needle_lower = needle.to_ascii_lowercase();
        if lower.contains(&needle_lower) {
            return Err(CoreError::InvalidInput(MSG_SECRET.to_string()));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::STACK_IDS;
    use dare_assets::EmbeddedAssets;
    use dare_core::CoreError;

    #[test]
    fn render_substitutes() {
        let tpl = r#"{"projectName":"{{project_name}}","stack":"{{stack_id}}"}"#;
        let out = render_template(tpl, "demo-app", "rust-axum").expect("render");
        assert_eq!(
            out,
            r#"{"projectName":"demo-app","stack":"rust-axum"}"#
        );
        assert!(!out.contains("{{project_name}}"));
        assert!(!out.contains("{{stack_id}}"));
    }

    #[test]
    fn secret_scan_rejects_api_key() {
        let err = match scan_secrets("config api_key=secret") {
            Ok(()) => panic!("expected secret scan to reject"),
            Err(e) => e,
        };
        match err {
            CoreError::InvalidInput(msg) => {
                assert_eq!(msg, MSG_SECRET);
            }
            other => panic!("expected InvalidInput, got {other:?}"),
        }

        let err = match render_template("API_KEY=x", "demo", "go-gin") {
            Ok(_) => panic!("expected render to reject after scan"),
            Err(e) => e,
        };
        match err {
            CoreError::InvalidInput(msg) => assert_eq!(msg, MSG_SECRET),
            other => panic!("expected InvalidInput, got {other:?}"),
        }
    }

    #[test]
    fn each_stack_template_root_exists() {
        for &id in STACK_IDS {
            let path = format!("stacks/{id}/dare.config.json.tpl");
            let file = EmbeddedAssets::get(&path);
            assert!(
                file.is_some(),
                "missing embedded template root file: {path}"
            );
            let bytes = file.expect("just checked").data;
            assert!(
                !bytes.is_empty(),
                "empty template at stacks/{id}/dare.config.json.tpl"
            );
        }
    }
}
