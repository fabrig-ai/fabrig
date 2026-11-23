//! Each row of data/codec/command.md 7, checked without writing.

#[cfg(test)]
mod tests {
    use skein_codegen::diff::Version;
    use skein_codegen::patch::{Error, Finding, Landed, Tree, check};
    use std::path::{Path, PathBuf};

    const V1: &str = "family shelf 1\nrecord Book { id: u8 }\nenum State { ready }\n";
    const V2: &str = "family shelf 2\nrecord Book { id: u8, score: u32 }\nenum State { ready }\n";
    const PATCH: &str = "patch 1\nowner shop 1\n\nfamily shelf 1 to 2\nadd Book.score u32 7\n";

    fn checked(files: &[(&str, &str)], landed: &Landed) -> Result<Vec<Finding>, Error> {
        let tree =
            Tree::new(files.iter().map(|(path, text)| (PathBuf::from(path), text.as_bytes().to_vec())).collect());
        let before = tree.clone();
        let result = check(&tree, landed);
        assert_eq!(tree, before, "the check writes nothing");
        result
    }

    fn findings(files: &[(&str, &str)], landed: &Landed, expected: &[Finding]) {
        assert_eq!(checked(files, landed), Ok(expected.to_vec()));
    }

    fn base(files: &[(&str, &str)]) -> Landed {
        Landed::new(files.iter().map(|(path, text)| (PathBuf::from(path), text.as_bytes().to_vec())).collect())
    }

    #[test]
    fn a_failed_check_leaves_no_crate_behind() {
        let result = checked(&[("schema/shelf-v1.schema", V1)], &Landed::none());
        assert_eq!(result, Ok(vec![Finding::Unkept { family: "shelf".into() }]));
    }

    #[test]
    fn current_crate_has_no_finding_and_is_never_written() {
        let files = [
            ("schema/shelf-v2.schema", V2),
            ("schema/kept/shelf-v1.schema", V1),
            ("schema/kept/shelf-v2.schema", V2),
            ("patches/0001.patch", PATCH),
        ];
        assert_eq!(checked(&files, &base(&files)), Ok(Vec::new()));
    }

    #[test]
    fn current_without_kept_text_is_unkept() {
        findings(&[("schema/shelf-v1.schema", V1)], &Landed::none(), &[Finding::Unkept { family: "shelf".into() }]);
    }

    #[test]
    fn missing_middle_kept_version_is_a_gap() {
        const V3: &str = "family shelf 3\nrecord Book { id: u8 }\nenum State { ready }\n";
        findings(
            &[("schema/kept/shelf-v1.schema", V1), ("schema/kept/shelf-v3.schema", V3)],
            &Landed::none(),
            &[
                Finding::KeptGap { family: "shelf".into(), missing: 2 },
                Finding::Unpatched { family: "shelf".into(), version: 3 },
            ],
        );
    }

    #[test]
    fn later_kept_version_requires_a_patch_to_it() {
        findings(
            &[("schema/kept/shelf-v1.schema", V1), ("schema/kept/shelf-v2.schema", V2)],
            &Landed::none(),
            &[Finding::Unpatched { family: "shelf".into(), version: 2 }],
        );
    }

    #[test]
    fn current_needing_patch_requires_one_to_its_version() {
        findings(
            &[("schema/shelf-v2.schema", V2), ("schema/kept/shelf-v1.schema", V1)],
            &Landed::none(),
            &[Finding::Unpatched { family: "shelf".into(), version: 2 }],
        );
    }

    #[test]
    fn unfrozen_comments_are_lagging_until_the_run_refreshes() {
        const COMMENTED: &str = "# current comment\nfamily shelf 1\nrecord Book { id: u8 }\nenum State { ready }\n";
        findings(
            &[("schema/shelf-v1.schema", COMMENTED), ("schema/kept/shelf-v1.schema", V1)],
            &Landed::none(),
            &[Finding::Lagging { family: "shelf".into(), version: 1 }],
        );
    }

    #[test]
    fn wrong_current_version_names_both_versions() {
        const V7: &str = "family shelf 7\nrecord Book { id: u8 }\nenum State { ready }\n";
        const V9: &str = "family shelf 9\nrecord Book { id: u8 }\nenum State { ready }\n";
        findings(
            &[("schema/shelf-v9.schema", V9), ("schema/kept/shelf-v7.schema", V7)],
            &Landed::none(),
            &[Finding::Version { family: "shelf".into(), verdict: Version::Wrong, kept: 7, current: 9 }],
        );
    }

    #[test]
    fn moved_version_without_layout_change_is_needless() {
        const SAME2: &str = "family shelf 2\nrecord Book { id: u8 }\nenum State { ready }\n";
        findings(
            &[("schema/shelf-v2.schema", SAME2), ("schema/kept/shelf-v1.schema", V1)],
            &Landed::none(),
            &[Finding::Version { family: "shelf".into(), verdict: Version::Needless, kept: 1, current: 2 }],
        );
    }

    #[test]
    fn needed_layout_change_at_kept_version_is_unmoved() {
        const WIDE1: &str = "family shelf 1\nrecord Book { id: u16 }\nenum State { ready }\n";
        findings(
            &[("schema/shelf-v1.schema", WIDE1), ("schema/kept/shelf-v1.schema", V1)],
            &Landed::none(),
            &[
                Finding::Unpatched { family: "shelf".into(), version: 1 },
                Finding::Version { family: "shelf".into(), verdict: Version::Unmoved, kept: 1, current: 1 },
            ],
        );
    }

    #[test]
    fn changed_or_missing_landed_files_are_reported_by_path() {
        findings(
            &[("patches/golden/0001/shelf/book", "edited")],
            &base(&[
                ("schema/kept/shelf-v1.schema", V1),
                ("patches/0001.patch", PATCH),
                ("patches/golden/0001/shelf/book", "landed"),
            ]),
            &[
                Finding::Changed { path: "patches/0001.patch".into() },
                Finding::Changed { path: "patches/golden/0001/shelf/book".into() },
                Finding::Changed { path: "schema/kept/shelf-v1.schema".into() },
            ],
        );
    }

    #[test]
    fn refreshed_unfrozen_kept_text_is_exempt_from_changed() {
        const RENAMED: &str = "family shelf 1\nrecord Book { key: u8 }\nenum State { ready }\n";
        let files = [("schema/shelf-v1.schema", RENAMED), ("schema/kept/shelf-v1.schema", RENAMED)];
        assert_eq!(checked(&files, &base(&[("schema/kept/shelf-v1.schema", V1)])), Ok(Vec::new()));
    }

    #[test]
    fn missing_patch_number_is_reported_without_writing_it() {
        findings(
            &[("patches/0002.patch", "patch 1\nowner shop 2\n")],
            &Landed::none(),
            &[Finding::PatchGap { missing: 1 }],
        );
    }

    #[test]
    fn patch_level_must_match_its_filename_number() {
        findings(
            &[("patches/0001.patch", "patch 1\nowner shop 7\n")],
            &Landed::none(),
            &[Finding::Misnumbered { patch: 1, level: 7 }],
        );
    }

    #[test]
    fn asked_value_is_unplanned_with_line_span_and_variant_name() {
        const ASKED: &str = "patch 1\nowner shop 1\n\nfamily shelf 1 to 2\nadd Book.score u32 ?\n";
        let result = checked(
            &[("schema/kept/shelf-v1.schema", V1), ("schema/kept/shelf-v2.schema", V2), ("patches/0001.patch", ASKED)],
            &Landed::none(),
        )
        .expect("the crate reads");
        assert_eq!(
            result,
            [Finding::Unplanned { patch: 1, family: "shelf".into(), error: "line 5 at 61..62: Asked".into() }]
        );
    }

    #[test]
    fn missing_kept_text_prevents_planning_its_block() {
        findings(
            &[("schema/kept/shelf-v1.schema", V1), ("patches/0001.patch", PATCH)],
            &Landed::none(),
            &[Finding::Unplanned { patch: 1, family: "shelf".into(), error: "line 4 at 22..28: Unkept".into() }],
        );
    }

    #[test]
    fn a_crate_the_check_cannot_read_is_its_error_not_a_finding() {
        let error = checked(&[("schema/shelf-v1.schema", "invalid")], &Landed::none()).expect_err("schema read failed");
        let Error::Schema { path, .. } = error else { unreachable!("a schema that does not parse") };
        assert_eq!(path, Path::new("schema/shelf-v1.schema"));
    }

    #[test]
    fn each_finding_displays_the_required_fix_or_refusal() {
        for (finding, expected) in [
            (Finding::Unkept { family: "shelf".into() }, "shelf: no kept text; run `skein-codegen patch <crate>`"),
            (
                Finding::KeptGap { family: "shelf".into(), missing: 2 },
                "shelf: kept version 2 missing; kept versions run without a gap",
            ),
            (
                Finding::Unpatched { family: "shelf".into(), version: 2 },
                "shelf: no patch moves it to version 2; run `skein-codegen patch <crate>`",
            ),
            (
                Finding::Lagging { family: "shelf".into(), version: 1 },
                "shelf: kept version 1 lags the current text; run `skein-codegen patch <crate>`",
            ),
            (
                Finding::Version { family: "shelf".into(), verdict: Version::Wrong, kept: 7, current: 9 },
                "shelf: version 9 over kept 7 is Wrong (data/codec/command.md, 5)",
            ),
            (Finding::Changed { path: "patches/0001.patch".into() }, "patches/0001.patch: landed, and changed or gone"),
            (Finding::PatchGap { missing: 1 }, "patch 0001 missing"),
            (Finding::Misnumbered { patch: 1, level: 7 }, "patch 0001: its header's level is 7"),
            (
                Finding::Unplanned { patch: 1, family: "shelf".into(), error: "line 5: Asked".into() },
                "patch 0001, shelf: does not plan: line 5: Asked",
            ),
        ] {
            assert_eq!(finding.to_string(), expected);
        }
    }
}
