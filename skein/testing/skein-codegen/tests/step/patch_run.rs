#[cfg(test)]
mod tests {
    use skein_codegen::diff::Rename;
    use skein_codegen::patch::Outcome;

    #[test]
    fn report_displays_outcomes_requests_proposals_and_findings() {
        use skein_codegen::patch::{Finding, Report};
        let report = Report {
            outcomes: vec![
                Outcome::Current { family: "label".into() },
                Outcome::Patched { family: "shelf".into(), from: 1, to: 2 },
            ],
            patch: Some(3),
            moved: Some(2),
            restored: vec!["patches/0002.patch".into()],
            asked: vec!["patch 0003, shelf: written Book".into()],
            proposed: vec![
                ("shelf".into(), Rename::Declaration { old: "Old".into(), new: "Book".into() }),
                ("shelf".into(), Rename::Part { decl: "Book".into(), old: "id".into(), new: "key".into() }),
            ],
            findings: vec![Finding::PatchGap { missing: 1 }],
        };
        assert_eq!(
            report.to_string(),
            "label: current\nshelf: patched 1 to 2\npatch 0003 (moved from 0002)\nrestored patches/0002.patch\nasked: patch 0003, shelf: written Book\n--rename shelf Old Book\n--rename shelf Book.id key\npatch 0001 missing\n"
        );
    }
}
