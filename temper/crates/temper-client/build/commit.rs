//! The build's commit word (temper/docs/design/release.md, section 3).

pub fn word(head: Option<&str>, dirty: bool) -> String {
    let Some(head) = head else {
        return "unknown".into();
    };
    if !head.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)) {
        return "unknown".into();
    }
    let Some(short) = head.get(..12) else {
        return "unknown".into();
    };
    if dirty { format!("{short}-dirty") } else { short.into() }
}
