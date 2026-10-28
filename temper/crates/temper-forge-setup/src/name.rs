//! Names shared by setup and the fake forge.
/// Checks the forge repository name grammar.
#[must_use]
pub fn repository(b: &[u8]) -> bool {
    if b.is_empty() || !b.first().expect("nonempty").is_ascii_alphanumeric() {
        return false;
    }
    for c in b {
        if !c.is_ascii_alphanumeric() && !b"._-".contains(c) {
            return false;
        }
    }
    true
}
/// Checks a relative file path with no . or .. component.
#[must_use]
pub fn relative(b: &[u8]) -> bool {
    if b.is_empty() || b.starts_with(b"/") || b.contains(&0) {
        return false;
    }
    let mut start = 0_usize;
    for end in 0..=b.len() {
        if end == b.len() || b.get(end) == Some(&b'/') {
            let part = b.get(start..end).expect("within path");
            if part.is_empty() || part == b"." || part == b".." {
                return false;
            }
            start = end.saturating_add(1);
        }
    }
    true
}
/// Checks the branch name grammar.
#[must_use]
pub fn branch(b: &[u8]) -> bool {
    if !relative(b) || b == b"@" || b.ends_with(b".") {
        return false;
    }
    for c in b {
        if *c <= 32 || *c == 127 || b"~^:?*[\\".contains(c) {
            return false;
        }
    }
    for pair in b.windows(2) {
        if pair == b".." || pair == b"@{" {
            return false;
        }
    }
    let mut start = 0_usize;
    for end in 0..=b.len() {
        if end == b.len() || b.get(end) == Some(&b'/') {
            let part = b.get(start..end).expect("within path");
            if part.starts_with(b".") || part.ends_with(b".lock") {
                return false;
            }
            start = end.saturating_add(1);
        }
    }
    true
}
