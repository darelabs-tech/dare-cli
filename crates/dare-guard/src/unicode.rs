//! Unicode threat detection and sanitization.

use crate::report::FindingSeverity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnicodeKind {
    ZeroWidth,
    Bidi,
    VariationSelector,
    Tag,
    Homoglyph,
}

impl UnicodeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            UnicodeKind::ZeroWidth => "zero-width",
            UnicodeKind::Bidi => "bidi",
            UnicodeKind::VariationSelector => "variation-selector",
            UnicodeKind::Tag => "tag",
            UnicodeKind::Homoglyph => "homoglyph",
        }
    }

    pub fn severity(self) -> FindingSeverity {
        match self {
            UnicodeKind::Homoglyph => FindingSeverity::Warn,
            _ => FindingSeverity::Fail,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnicodeHit {
    pub kind: UnicodeKind,
    pub offset: usize,
    pub ch: char,
}

/// Detect dangerous unicode code points.
pub fn analyze_unicode(input: &str) -> Vec<UnicodeHit> {
    let mut hits = Vec::new();
    for (offset, ch) in input.char_indices() {
        if let Some(kind) = classify_char(ch) {
            hits.push(UnicodeHit { kind, offset, ch });
        }
    }
    hits
}

/// Remove characters classified as unicode threats (keeps homoglyphs — only strips control-like).
pub fn strip_unicode(input: &str) -> String {
    input
        .chars()
        .filter(|ch| match classify_char(*ch) {
            Some(UnicodeKind::Homoglyph) => true,
            Some(_) => false,
            None => true,
        })
        .collect()
}

fn classify_char(ch: char) -> Option<UnicodeKind> {
    let cp = ch as u32;
    if is_zero_width(cp) {
        return Some(UnicodeKind::ZeroWidth);
    }
    if is_bidi(cp) {
        return Some(UnicodeKind::Bidi);
    }
    if is_variation_selector(cp) {
        return Some(UnicodeKind::VariationSelector);
    }
    if is_tag(cp) {
        return Some(UnicodeKind::Tag);
    }
    if is_homoglyph(ch) {
        return Some(UnicodeKind::Homoglyph);
    }
    None
}

fn is_zero_width(cp: u32) -> bool {
    matches!(
        cp,
        0x200B | 0x200C | 0x200D | 0x2060 | 0xFEFF | 0x180E | 0x00AD
    )
}

fn is_bidi(cp: u32) -> bool {
    matches!(
        cp,
        0x202A..=0x202E | 0x2066..=0x2069 | 0x200E | 0x200F
    )
}

fn is_variation_selector(cp: u32) -> bool {
    matches!(cp, 0xFE00..=0xFE0F | 0xE0100..=0xE01EF)
}

fn is_tag(cp: u32) -> bool {
    matches!(cp, 0xE0001 | 0xE0020..=0xE007F)
}

/// Conservative homoglyph set: Cyrillic lookalikes of Latin letters commonly used in injection.
fn is_homoglyph(ch: char) -> bool {
    matches!(
        ch,
        'а' | 'е' | 'о' | 'р' | 'с' | 'у' | 'х' | 'і' | 'ј' // Cyrillic / lookalikes
            | 'Α' | 'Β' | 'Ε' | 'Ζ' | 'Η' | 'Ι' | 'Κ' | 'Μ' | 'Ν' | 'Ο' | 'Ρ' | 'Τ' | 'Υ' | 'Χ'
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_zero_width() {
        let s = format!("hello{}world", '\u{200B}');
        let hits = analyze_unicode(&s);
        assert!(hits.iter().any(|h| h.kind == UnicodeKind::ZeroWidth));
    }

    #[test]
    fn detects_bidi() {
        let s = format!("ab{}cd", '\u{202E}');
        let hits = analyze_unicode(&s);
        assert!(hits.iter().any(|h| h.kind == UnicodeKind::Bidi));
    }

    #[test]
    fn strip_removes_zw() {
        let s = format!("a{}b", '\u{200B}');
        assert_eq!(strip_unicode(&s), "ab");
    }

    #[test]
    fn clean_ascii_no_hits() {
        assert!(analyze_unicode("plain ascii text").is_empty());
    }
}
