//! Template rendering compatible with the npm `template-engine.ts`.
//!
//! npm picks the engine by extension (Handlebars / Nunjucks / Mustache / raw).
//! Every bundled stack template only uses plain `{{ var }}` interpolation, so a
//! strict interpolator reproduces all three engines byte-for-byte. Any other
//! tag (blocks, partials, comments) is rejected so a future template change is
//! caught by tests instead of being rendered differently.

use std::collections::BTreeMap;

use dare_core::{CoreError, CoreResult};

/// Engine selected from the template file name (npm `detectEngine`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateEngine {
    Handlebars,
    Nunjucks,
    Mustache,
    Raw,
}

pub fn detect_engine(template: &str) -> TemplateEngine {
    let lower = template.to_ascii_lowercase();
    if [".hbs", ".handlebars", ".tpl"]
        .iter()
        .any(|e| lower.ends_with(e))
    {
        TemplateEngine::Handlebars
    } else if [".j2", ".jinja2", ".tera", ".erb"]
        .iter()
        .any(|e| lower.ends_with(e))
    {
        TemplateEngine::Nunjucks
    } else if lower.ends_with(".mustache") {
        TemplateEngine::Mustache
    } else {
        TemplateEngine::Raw
    }
}

/// Template variables (npm `vars` object; values are already stringified).
pub type TemplateVars = BTreeMap<&'static str, String>;

fn is_ident(s: &str) -> bool {
    let mut chars = s.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Render `source` with `vars`. `name` is used in error messages only.
pub fn render_template_str(name: &str, source: &str, vars: &TemplateVars) -> CoreResult<String> {
    let engine = detect_engine(name);
    if engine == TemplateEngine::Raw {
        return Ok(source.to_string());
    }
    if engine == TemplateEngine::Nunjucks && (source.contains("{%") || source.contains("{#")) {
        return Err(CoreError::internal(format!(
            "unsupported template tag in '{name}' (only {{{{ var }}}} interpolation is ported)"
        )));
    }
    let mut out = String::with_capacity(source.len());
    let mut rest = source;
    while let Some(start) = rest.find("{{") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let end = after.find("}}").ok_or_else(|| {
            CoreError::internal(format!("unterminated '{{{{' in template '{name}'"))
        })?;
        let inner = after[..end].trim();
        if !is_ident(inner) {
            return Err(CoreError::internal(format!(
                "unsupported template tag '{{{{{}}}}}' in '{name}'",
                &after[..end]
            )));
        }
        let value = vars.get(inner).ok_or_else(|| {
            CoreError::internal(format!(
                "Failed to render '{name}': missing template variable '{inner}'"
            ))
        })?;
        out.push_str(value);
        rest = &after[end + 2..];
    }
    out.push_str(rest);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vars() -> TemplateVars {
        let mut v = TemplateVars::new();
        v.insert("projectName", "demo".into());
        v.insert("crateName", "demo_app".into());
        v
    }

    #[test]
    fn engine_detection_matches_npm() {
        assert_eq!(detect_engine("go.mod.tpl"), TemplateEngine::Handlebars);
        assert_eq!(detect_engine("Cargo.toml.tera"), TemplateEngine::Nunjucks);
        assert_eq!(detect_engine("x.ERB"), TemplateEngine::Nunjucks);
        assert_eq!(detect_engine(".env.example"), TemplateEngine::Raw);
    }

    #[test]
    fn interpolates_both_spacing_styles() {
        let got = render_template_str("a.hbs", "n={{projectName}} c={{ crateName }}", &vars())
            .expect("render");
        assert_eq!(got, "n=demo c=demo_app");
    }

    #[test]
    fn raw_templates_keep_braces() {
        let src = "{{ text }} stays";
        assert_eq!(render_template_str("p.txt", src, &vars()).unwrap(), src);
    }

    #[test]
    fn missing_var_and_blocks_fail() {
        assert!(render_template_str("a.hbs", "{{nope}}", &vars()).is_err());
        assert!(render_template_str("a.hbs", "{{#if x}}y{{/if}}", &vars()).is_err());
        assert!(render_template_str("a.j2", "{% if x %}", &vars()).is_err());
    }
}
