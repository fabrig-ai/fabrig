//! Goal names shared by forge filings and pages (temper/docs/design/client/pages/work.md, 1).
/// The leading goal id (temper/docs/design/client/pages/work.md, 1, A goal's id).
#[must_use]
pub fn id(title: &[u8]) -> Option<&[u8]> {
    let mut letters = 0_usize;
    for byte in title {
        if !byte.is_ascii_uppercase() {
            break;
        }
        letters = letters.checked_add(1).expect("title fits");
    }
    if !(1..=3).contains(&letters) {
        return None;
    }
    let rest = title.get(letters..)?;
    let mut digits = 0_usize;
    for byte in rest {
        if !byte.is_ascii_digit() {
            break;
        }
        digits = digits.checked_add(1).expect("title fits");
    }
    if !(1..=3).contains(&digits) {
        return None;
    }
    let end = letters.checked_add(digits).expect("goal id fits");
    if let Some(byte) = title.get(end)
        && byte.is_ascii_alphanumeric()
    {
        return None;
    }
    title.get(..end)
}

#[cfg(test)]
mod tests {
    use super::id;
    #[test]
    fn a_goals_id_is_its_leading_capitals_and_digits() {
        for (title, expected) in [
            ("S7 history in task logs", Some(b"S7".as_slice())),
            ("K10 · prepare reconnect statements once", Some(b"K10".as_slice())),
            ("H1, Home's parts", Some(b"H1".as_slice())),
            ("ABC123", Some(b"ABC123".as_slice())),
            ("A0", Some(b"A0".as_slice())),
            ("Fix 3 bugs", None),
            ("s7 history", None),
            ("SS7a", None),
            ("ABCD1", None),
            ("A1234", None),
            ("A", None),
            ("1", None),
            ("", None),
        ] {
            assert_eq!(id(title.as_bytes()), expected, "{title}");
        }
    }
}
