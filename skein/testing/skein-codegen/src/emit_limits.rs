//! Rust names and limit names for data/codec/README.md, section 4.

pub(crate) use skein_codec::schema::LimitLevel;

pub(crate) fn bound_name(record: &str, field: &str) -> String {
    String::from_utf8(skein_codec::schema::limit_name(record.as_bytes(), field.as_bytes()).into_vec())
        .expect("schema identifiers are ASCII")
}

pub(crate) fn child_name(parent: &str, level: LimitLevel) -> String {
    String::from_utf8(skein_codec::schema::limit_child(parent.as_bytes(), level).into_vec())
        .expect("schema identifiers are ASCII")
}

/// Lowers capitals into words separated by underscores, mapping dots and hyphens to underscores.
#[must_use]
pub fn snake(name: &str) -> String {
    let mut out = String::new();
    for character in name.chars() {
        if character.is_ascii_uppercase() && !out.is_empty() {
            out.push('_');
        }
        out.push(if character == '.' || character == '-' { '_' } else { character.to_ascii_lowercase() });
    }
    out
}

/// Uppercases the first character and each character after an underscore, removing underscores.
#[must_use]
pub fn pascal(name: &str) -> String {
    let mut out = String::new();
    let mut upper = true;
    for character in name.chars() {
        if character == '_' {
            upper = true;
        } else if upper {
            out.push(character.to_ascii_uppercase());
            upper = false;
        } else {
            out.push(character);
        }
    }
    out
}

pub(crate) fn literal(value: impl ToString) -> String {
    let digits = value.to_string();
    let mut grouped = String::new();
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && digits.len().checked_sub(index).expect("index within digits").is_multiple_of(3) {
            grouped.push('_');
        }
        grouped.push(digit);
    }
    grouped
}
