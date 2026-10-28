use super::fixtures;
#[test]
fn no_file_under_jig_names_temper_or_an_item_outside_jig() {
    for (path, text) in &fixtures::sources() {
        assert!(!text.to_ascii_lowercase().contains("temper"), "{path:?} names temper");
        for (at, _) in text.match_indices("crate::") {
            assert!(text[at..].starts_with("crate::jig"), "{path:?} names an item outside jig");
        }
    }
}
