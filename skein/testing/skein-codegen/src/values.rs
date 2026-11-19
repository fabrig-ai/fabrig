//! Shared current-text values (data/codec/README.md, section 6).

use alloc::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

use crate::{Golden, Schema, goldens, parse};

/// A text's parsed schema and fixed-rule encodings, built once for the process.
#[derive(Debug)]
pub struct Values {
    pub schema: Schema,
    pub goldens: Vec<Golden>,
}

/// Returns the process's values for this text, panicking with its parse error.
#[must_use]
#[expect(
    clippy::panic,
    reason = "invalid author-owned schema text panics with its parse error (data/codec/README.md, 6)"
)]
pub fn values(text: &'static str) -> &'static Values {
    static TEXTS: OnceLock<Mutex<BTreeMap<&'static str, &'static OnceLock<Values>>>> = OnceLock::new();
    let cell = {
        let mut texts = TEXTS.get_or_init(Mutex::default).lock().expect("schema texts lock");
        *texts.entry(text).or_insert_with(|| Box::leak(Box::new(OnceLock::new())))
    };
    cell.get_or_init(|| {
        let schema = parse(text).unwrap_or_else(|why| panic!("{why}"));
        let goldens = goldens(&schema);
        Values { schema, goldens }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEXT: &str = "family cached 1 record Item { value: u8 }";

    #[test]
    fn one_text_has_one_schema_and_its_fixed_rule_values() {
        let first = values(TEXT);
        assert!(core::ptr::eq(first, values(TEXT)));
        let copied = Box::leak(TEXT.to_owned().into_boxed_str());
        assert!(core::ptr::eq(first, values(copied)), "the key is text, not its address");
        let schema = parse(TEXT).expect("test schema");
        assert_eq!(first.schema, schema);
        assert_eq!(first.goldens, goldens(&schema));
        let other = values("family other 1 record Item { value: bool }");
        assert!(!core::ptr::eq(first, other));
        assert_eq!(other.schema.family, "other");
    }

    #[test]
    #[should_panic(expected = "line 1:")]
    fn an_invalid_text_panics_with_its_parse_error() {
        let _invalid = values("not a schema");
    }
}
