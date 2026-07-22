use std::collections::BTreeMap;

use dare_core::{CoreError, CoreResult};

use crate::{ENRICHABLE, MARKER_BEGIN, MARKER_END_PREFIX};

fn marker_begin(section: &str) -> String {
    format!("{MARKER_BEGIN}{section}\" -->")
}

fn marker_end(section: &str) -> String {
    format!("{MARKER_END_PREFIX}{section}\" -->")
}

/// Replace only the interior of each ENRICHABLE marker block; preserve marker lines and all other text.
pub fn inject_enrichable(
    markdown: &str,
    sections: &BTreeMap<String, String>,
) -> CoreResult<String> {
    inject_sections(markdown, sections, ENRICHABLE)
}

pub fn inject_sections(
    markdown: &str,
    sections: &BTreeMap<String, String>,
    ids: &[&str],
) -> CoreResult<String> {
    let mut out = markdown.to_string();

    for id in ids {
        let body = sections
            .get(*id)
            .ok_or_else(|| CoreError::invalid_input(format!("missing enrichment section: {id}")))?;
        out = replace_marker_interior(&out, id, body)?;
    }

    Ok(out)
}

fn replace_marker_interior(content: &str, section_id: &str, new_body: &str) -> CoreResult<String> {
    let begin = marker_begin(section_id);
    let end = marker_end(section_id);

    let begin_pos = content.find(&begin).ok_or_else(|| {
        CoreError::invalid_input(format!("missing AGENT markers for section {section_id}"))
    })?;
    let after_begin = begin_pos + begin.len();

    let tail = &content[after_begin..];
    let end_rel = tail.find(&end).ok_or_else(|| {
        CoreError::invalid_input(format!("missing AGENT markers for section {section_id}"))
    })?;

    let interior_start = after_begin;
    let interior_end = after_begin + end_rel;

    let between = &content[interior_start..interior_end];
    if between.contains("<!-- AGENT:BEGIN") || between.contains("<!-- AGENT:END") {
        return Err(CoreError::invalid_input("malformed AGENT markers"));
    }

    let mut injected = String::with_capacity(content.len() + new_body.len());
    injected.push_str(&content[..interior_start]);
    injected.push('\n');
    injected.push_str(new_body);
    injected.push('\n');
    injected.push_str(&content[interior_end..]);

    Ok(injected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn sample_markdown() -> String {
        format!(
            "# Title\n\n\
             Unmanaged paragraph must survive.\n\n\
             {begin}\n\
             old description\n\
             {end}\n\n\
             ## Other\n\n\
             {obj_begin}\n\
             old objectives\n\
             {obj_end}\n\n\
             {fr_begin}\n\
             old fr\n\
             {fr_end}\n\n\
             {stack_begin}\n\
             old stack\n\
             {stack_end}\n",
            begin = marker_begin("description"),
            end = marker_end("description"),
            obj_begin = marker_begin("objectives"),
            obj_end = marker_end("objectives"),
            fr_begin = marker_begin("functional-requirements"),
            fr_end = marker_end("functional-requirements"),
            stack_begin = marker_begin("stack"),
            stack_end = marker_end("stack"),
        )
    }

    fn full_sections() -> BTreeMap<String, String> {
        let mut m = BTreeMap::new();
        m.insert("description".into(), "new description".into());
        m.insert("objectives".into(), "new objectives".into());
        m.insert(
            "functional-requirements".into(),
            "new functional requirements".into(),
        );
        m.insert("stack".into(), "new stack".into());
        m
    }

    #[test]
    fn inject_replaces_only_bodies() {
        let md = sample_markdown();
        let injected = inject_enrichable(&md, &full_sections()).unwrap();

        assert!(injected.contains("Unmanaged paragraph must survive."));
        assert!(injected.contains(&marker_begin("description")));
        assert!(injected.contains(&marker_end("description")));
        assert!(injected.contains("new description"));
        assert!(!injected.contains("old description"));
        assert!(injected.contains("new stack"));
        assert!(!injected.contains("old stack"));
    }

    #[test]
    fn inject_missing_marker_errors() {
        let md = format!(
            "{begin}\nbody\n{end}\n",
            begin = marker_begin("description"),
            end = marker_end("description"),
        );
        let mut sections = full_sections();
        let err = inject_enrichable(&md, &sections).unwrap_err();
        assert!(err.to_string().contains("missing AGENT markers"));

        sections.remove("objectives");
        let md = sample_markdown();
        let err = inject_enrichable(&md, &sections).unwrap_err();
        assert!(err.to_string().contains("missing enrichment section"));
    }
}
