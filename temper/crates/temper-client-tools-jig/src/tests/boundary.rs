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
        } else if path == &std::path::PathBuf::from("src/jig/render/goal_title.rs") {
            // The goal title warning names the application's help page (client/ctl/areas.md), area 2.
            names.replace("temper ctl help goal start", "")
        } else if path == &std::path::PathBuf::from("src/jig/rows.rs") {
            // Help names local recovery and the worker's secret procedure (client/ctl/README.md, 6.1; client/ctl/workers.md, 1).
            names
                .replace("temper help server", "")
                .replace("temper ctl dir sign-out", "")
                .replace("temper does not know it", "")
                .replace("temper-box serve or\\nask with", "")
                .replace("temper ctl directory link --out file", "")
                .replace("temper ctl directory party reissue party --out file", "")
        } else {
            names
        };
        // Milestone identity is shared with the pages (client/pages/milestone.md, 1).
        // These three call sites name only its template and chip predicate.
        let names = if [
            std::path::PathBuf::from("src/jig/runner/milestone.rs"),
            std::path::PathBuf::from("src/jig/runner/acts/milestone.rs"),
            std::path::PathBuf::from("src/jig/runner/acts/goal.rs"),
        ]
        .contains(path)
        {
            names
                .replace("temper_client_domain_forge::milestones::template", "")
                .replace("temper_client_domain_forge::milestones::is", "")
        } else {
            names
        };
        assert!(!names.contains("temper"), "{path:?} names temper");
        for (at, _) in text.match_indices("crate::") {
            assert!(text[at..].starts_with("crate::jig"), "{path:?} names an item outside jig");
        }
    }
}
