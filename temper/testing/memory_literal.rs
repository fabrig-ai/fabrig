//! Literal prices regenerate only under the same opt-in as codec drift.
pub fn check(path: &std::path::Path, marker: &str, expected: u64, actual: u64) {
    if std::env::var_os("SKEIN_CODEGEN_WRITE").is_some_and(|value| value == "1") {
        let source = std::fs::read_to_string(path).expect("memory assertion source");
        let start = source.find(marker).expect("literal marker").checked_add(marker.len()).expect("literal start fits");
        let end = start.checked_add(source[start..].find(';').expect("literal end")).expect("literal end fits");
        std::fs::write(path, format!("{}{}{}", &source[..start], actual, &source[end..]))
            .expect("regenerate memory literal");
    } else {
        assert_eq!(actual, expected, "regenerate with SKEIN_CODEGEN_WRITE=1");
    }
}
