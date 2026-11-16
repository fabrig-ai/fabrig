use super::fixtures;
#[test]
fn no_file_under_jig_names_temper_or_an_item_outside_jig() {
    for (path, text) in &fixtures::sources() {
        // The waiting line names the application's help page (client/ctl/results.md, 1).
        let names = text.to_ascii_lowercase();
        let names = if path == &std::path::PathBuf::from("src/jig/render/document/llm.rs") {
            names
                .replace("see temper help settings for held revisions", "")
                .replace("temper/docs/design/client/ctl/results.md, 1", "")
        } else if path == &std::path::PathBuf::from("src/jig/rows.rs") {
            // Sign-in help names the local recovery commands (client/ctl/README.md, 6.1).
            names
                .replace("temper ctl dir sign-out", "")
                .replace("temper does not know it", "")
                .replace("temper-box serve prints\\na fresh one", "")
        } else {
            names
        };
        assert!(!names.contains("temper"), "{path:?} names temper");
        for (at, _) in text.match_indices("crate::") {
            assert!(text[at..].starts_with("crate::jig"), "{path:?} names an item outside jig");
        }
    }
}
