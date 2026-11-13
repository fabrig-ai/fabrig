#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "test assertions inspect semantic trees and diagnostic redaction"
)]

#[cfg(target_pointer_width = "64")]
use temper_fixtures::memory_literal;

#[test]
fn english_phrase_declarations_match_the_vocabulary() {
    let path = skein_scratch::paths::package().join("src/words.rs");
    let source = std::fs::read_to_string(&path).unwrap();
    let variants = source.split("pub enum Word {\n").nth(1).unwrap().split("\n}").next().unwrap();
    let names: Vec<_> = variants.lines().map(str::trim).filter(|line| line.ends_with(',')).collect();
    let mut all = format!("const ALL: [Word; {}] = [\n", names.len());
    for name in names {
        all.push_str("    Word::");
        all.push_str(name);
        all.push('\n');
    }
    all.push_str("];");
    let start = source.find("const ALL: [Word;").unwrap();
    let end = start + source[start..].find("\n];").unwrap() + 3;
    if temper_test_env::rewrite() {
        let updated = format!("{}{}{}", &source[..start], all, &source[end..]);
        std::fs::write(&path, updated).unwrap();
    } else {
        assert_eq!(&source[start..end], all);
    }
    for (marker, expected, actual) in crate::words::english_declarations() {
        memory_literal::check(&path, marker, expected, actual);
    }
}
