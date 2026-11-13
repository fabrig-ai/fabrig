//! The wire rule from data/codec/command.md, 2, over scratch owner crates.

#[cfg(test)]
mod tests {
    use skein_codec::patch::{Build, Halt, Old, Site};
    use skein_codec::value::Node;
    use skein_codegen::diff::{Rename, Version};
    use skein_codegen::patch::{Error, Finding, Landed, Mode, Options, Outcome, check, current, golden, run};
    use skein_scratch::Scratch;
    use std::fs;
    use std::path::{Path, PathBuf};

    const KEPT: &str = "family w 1\nenum E { a, b }\nrecord R { id: u8 }\n";
    const WIRES: &str = "# wire families\n\n  w 1  # channel bodies\n\n";
    const EDITS: &[(&str, &str, &str)] = &[
        ("variant", "enum E { a, b, c }\nrecord R { id: u8 }\n", "add E.c\n"),
        ("added", "enum E { a, b }\nrecord R { id: u8 }\nrecord S { flag: bool }\n", "add S\n"),
        ("dropped", "enum E { a, b }\n", "drop R\n"),
        ("reordered", "record R { id: u8 }\nenum E { a, b }\n", ""),
    ];

    fn put(root: &Path, path: &str, text: &str) {
        let path = root.join(path);
        fs::create_dir_all(path.parent().expect("file parent")).expect("directories");
        fs::write(path, text).expect("fixture file");
    }

    fn text(root: &Path, path: &str, expected: &str) {
        assert_eq!(fs::read_to_string(root.join(path)).expect(path), expected, "{path}");
    }

    fn fixture(name: &str, kept: &str, now: &str, wires: &str) -> (Scratch, Landed) {
        let root = Scratch::new(&format!("patch-wires-{name}"));
        put(root.path(), "schema/kept/w-v1.schema", kept);
        let base = Landed::read(root.path()).expect("landed kept text, no patch");
        put(root.path(), "schema/w-v1.schema", now);
        put(root.path(), "schema/wires", wires);
        (root, base)
    }

    fn options() -> Options {
        Options { owner: Some("sample".into()), ..Options::default() }
    }

    fn unwritten(site: Site, _old: &Old, _new: &mut Build, _at: Node, _cx: &mut ()) -> Result<(), Halt> {
        Err(Halt::NoSite(site))
    }

    #[test]
    fn wire_layout_edits_require_next_version() {
        for (name, body, lines) in EDITS {
            let now = format!("family w 1\n{body}");
            let (scratch, base) = fixture(name, KEPT, &now, WIRES);
            let root = scratch.path();
            let findings = vec![
                Finding::Unpatched { family: "w".into(), version: 1 },
                Finding::Version { family: "w".into(), verdict: Version::Unmoved, kept: 1, current: 1 },
            ];
            assert_eq!(check(root, &base), Ok(findings.clone()), "{name}");
            text(root, "schema/w-v1.schema", &now);
            text(root, "schema/kept/w-v1.schema", KEPT);
            let report = run(root, &base, &options()).expect("refused family");
            assert_eq!(report.outcomes, [Outcome::Refused { family: "w".into(), verdict: Version::Unmoved }]);
            assert_eq!(report.findings, findings);
            assert_eq!(report.patch, None);
            text(root, "schema/w-v1.schema", &now);
            text(root, "schema/kept/w-v1.schema", KEPT);
            assert!(!root.join("patches/0001.patch").exists());

            let moved = format!("family w 2\n{body}");
            fs::remove_file(root.join("schema/w-v1.schema")).expect("move current filename");
            put(root, "schema/w-v2.schema", &moved);
            let report = run(root, &base, &options()).expect("wire patch");
            assert_eq!(report.outcomes, [Outcome::Patched { family: "w".into(), from: 1, to: 2 }]);
            assert_eq!(report.patch, Some(1));
            assert_eq!(report.asked, Vec::<String>::new());
            assert_eq!(report.findings, [], "{name}: the block plans without a site");
            text(root, "patches/0001.patch", &format!("patch 1\nowner sample 1\n\nfamily w 1 to 2\n{lines}"));
            text(root, "schema/kept/w-v1.schema", KEPT);
            text(root, "schema/kept/w-v2.schema", &moved);
            text(root, "schema/w-v2.schema", &moved);
            assert!(!root.join("patches/p0001.rs").exists());
            assert_eq!(golden(root, 1, unwritten, Mode::Check), Ok(()));
            for (file, bytes) in [
                ("enum_e_a_full.bin", &[0_u8][..]),
                ("enum_e_a_smallest.bin", &[0_u8][..]),
                ("enum_e_b_full.bin", &[1_u8][..]),
                ("enum_e_b_smallest.bin", &[1_u8][..]),
            ] {
                assert_eq!(fs::read(root.join("patches/golden/0001/w").join(file)).expect(file), bytes);
            }
            current(root).expect("generated module");
            let written = Landed::read(root).expect("branch files");
            run(root, &base, &options()).expect("derive again");
            assert_eq!(Landed::read(root).expect("unchanged files"), written);
        }
    }

    #[test]
    fn unlisted_layout_edits_still_refresh() {
        for (name, body, _) in EDITS {
            let now = format!("family w 1\n{body}");
            for wires in [None, Some("# no wire families\n"), Some("other 1\n")] {
                let (scratch, base) = fixture(name, KEPT, &now, wires.unwrap_or_default());
                let root = scratch.path();
                if wires.is_none() {
                    fs::remove_file(root.join("schema/wires")).expect("absent wires");
                } else if wires == Some("other 1\n") {
                    put(root, "schema/other-v1.schema", "family other 1\nrecord Other {}\n");
                }
                let mut findings = Vec::new();
                if wires == Some("other 1\n") {
                    findings.push(Finding::Unkept { family: "other".into() });
                }
                findings.push(Finding::Lagging { family: "w".into(), version: 1 });
                assert_eq!(check(root, &base), Ok(findings));
                let report = run(root, &base, &options()).expect("nonwire refresh");
                assert!(report.outcomes.contains(&Outcome::Refreshed { family: "w".into(), version: 1 }));
                assert_eq!(report.patch, None);
                assert_eq!(report.findings, []);
                text(root, "schema/kept/w-v1.schema", &now);
                text(root, "schema/w-v1.schema", &now);
                assert!(!root.join("patches/0001.patch").exists());
            }
        }
    }

    #[test]
    fn wire_renames_wait_at_first_landed_text() {
        const RENAMED: &str = "family w 1\nenum State { a, b }\nrecord R { key: u8 }\n";
        let (scratch, base) = fixture("rename", KEPT, RENAMED, WIRES);
        let root = scratch.path();
        let renames = vec![
            Rename::Declaration { old: "E".into(), new: "State".into() },
            Rename::Part { decl: "R".into(), old: "id".into(), new: "key".into() },
        ];
        let report = run(
            root,
            &base,
            &Options { renames: renames.iter().cloned().map(|rename| ("w".into(), rename)).collect(), ..options() },
        )
        .expect("confirmed renames wait");
        assert_eq!(report.outcomes, [Outcome::Waiting { family: "w".into(), renames }]);
        assert_eq!(report.findings, []);
        assert_eq!(report.patch, None);
        text(root, "schema/kept/w-v1.schema", KEPT);
        text(root, "schema/w-v1.schema", RENAMED);
        text(root, "schema/kept/w.renames", "rename E State\nrename R.id key\n");
        assert_eq!(check(root, &base), Ok(Vec::new()));
        let written = Landed::read(root).expect("waiting files");
        run(root, &base, &options()).expect("waiting renames read again");
        assert_eq!(Landed::read(root).expect("unchanged files"), written);
    }

    #[test]
    fn wire_nonlayout_edits_need_no_patch() {
        const OLD: &str = "family w 1\nrecord R { id: u8 }\nenum E { r: R, empty }\n";
        const NOW: &str = "# comment\nfamily\nw # header\n1\nrecord R {\nid : u8\n}\nenum E { r: boxed R, empty } # boxed is not layout\n";
        let (scratch, base) = fixture("nonlayout", OLD, NOW, WIRES);
        let root = scratch.path();
        assert_eq!(check(root, &base), Ok(Vec::new()));
        let report = run(root, &base, &options()).expect("no layout change");
        assert_eq!(report.outcomes, [Outcome::Current { family: "w".into() }]);
        assert_eq!(report.findings, []);
        assert_eq!(report.patch, None);
        text(root, "schema/kept/w-v1.schema", OLD);
        text(root, "schema/w-v1.schema", NOW);
        assert!(!root.join("patches/0001.patch").exists());
    }

    #[test]
    fn wire_versioned_fingerprints_set_aside_header_version() {
        const OLD: &str = "family w 1\nversioned record R { id: u8 }\nenum E { a, b }\n";
        const SAME: &str = "# header comment\nfamily\nw # family\n2\nversioned record R { id: u8 }\nenum E { a, b }\n";
        const NEXT: &str =
            "# header comment\nfamily\nw # family\n2\nversioned record R { id: u8 }\nenum E { a, b, c }\n";
        let (scratch, base) = fixture("versioned", OLD, OLD, WIRES);
        let root = scratch.path();
        fs::remove_file(root.join("schema/w-v1.schema")).expect("move current filename");
        put(root, "schema/w-v2.schema", SAME);
        assert_eq!(
            check(root, &base),
            Ok(vec![Finding::Version { family: "w".into(), verdict: Version::Needless, kept: 1, current: 2 }])
        );
        let report = run(root, &base, &options()).expect("version alone is needless");
        assert_eq!(report.outcomes, [Outcome::Refused { family: "w".into(), verdict: Version::Needless }]);
        text(root, "schema/kept/w-v1.schema", OLD);
        text(root, "schema/w-v2.schema", SAME);
        put(root, "schema/w-v2.schema", NEXT);
        let report = run(root, &base, &options()).expect("layout changed");
        assert_eq!(report.outcomes, [Outcome::Patched { family: "w".into(), from: 1, to: 2 }]);
        assert_eq!(report.findings, []);
        text(root, "patches/0001.patch", "patch 1\nowner sample 1\n\nfamily w 1 to 2\nadd E.c\n");
        text(root, "schema/kept/w-v2.schema", NEXT);
        assert_eq!(golden(root, 1, unwritten, Mode::Check), Ok(()));
    }

    #[test]
    fn wire_branch_kept_texts_are_kept_afresh() {
        let scratch = Scratch::new("patch-wires-afresh");
        let root = scratch.path();
        put(root, "schema/wires", WIRES);
        put(root, "schema/w-v1.schema", KEPT);
        let report = run(root, &Landed::none(), &Options { baseline: true, ..options() }).expect("baseline");
        assert_eq!(report.outcomes, [Outcome::Kept { family: "w".into(), version: 1 }]);
        text(root, "schema/kept/w-v1.schema", KEPT);
        for version in [1, 2] {
            let now = format!("family w {version}\nenum E {{ a, b, c }}\n");
            put(root, &format!("schema/w-v{version}.schema"), &now);
            put(root, "schema/wires", &format!("w {version}\n"));
            let report = run(root, &Landed::none(), &options()).expect("not landed: keep afresh");
            assert_eq!(report.outcomes, [Outcome::Kept { family: "w".into(), version }]);
            assert_eq!(report.findings, []);
            assert_eq!(report.patch, None);
            text(root, &format!("schema/kept/w-v{version}.schema"), &now);
            assert!(!root.join("patches/0001.patch").exists());
        }
        assert!(!root.join("schema/kept/w-v1.schema").exists());
    }

    #[test]
    fn wire_unknown_family_is_an_error_without_writes() {
        let (scratch, base) = fixture("unknown", KEPT, KEPT, "w 1\nunknown 1\n");
        let root = scratch.path();
        let before = Landed::read(root).expect("snapshot includes schema/wires");
        let expected = Error::Wire { family: "unknown".into() };
        assert_eq!(expected.to_string(), "schema/wires: unknown: no family of the crate");
        assert_eq!(check(root, &base), Err(expected.clone()));
        assert_eq!(run(root, &base, &options()), Err(expected));
        text(root, "schema/wires", "w 1\nunknown 1\n");
        text(root, "schema/kept/w-v1.schema", KEPT);
        text(root, "schema/w-v1.schema", KEPT);
        assert_eq!(Landed::read(root).expect("no command files written"), before);
    }

    #[test]
    fn wire_landed_kept_edit_is_changed_and_restored() {
        let (scratch, base) = fixture("changed", KEPT, KEPT, WIRES);
        let root = scratch.path();
        let commented = format!("# comment\n{KEPT}");
        put(root, "schema/kept/w-v1.schema", &commented);
        put(root, "schema/w-v1.schema", &commented);
        assert_eq!(check(root, &base), Ok(vec![Finding::Changed { path: PathBuf::from("schema/kept/w-v1.schema") }]));
        text(root, "schema/kept/w-v1.schema", &commented);
        let report = run(root, &base, &options()).expect("restore wire baseline");
        assert_eq!(report.restored, [PathBuf::from("schema/kept/w-v1.schema")]);
        assert_eq!(report.outcomes, [Outcome::Current { family: "w".into() }]);
        assert_eq!(report.findings, []);
        text(root, "schema/kept/w-v1.schema", KEPT);
        text(root, "schema/w-v1.schema", &commented);
    }
    #[test]
    fn wire_lines_require_a_numeric_floor_or_build() {
        for (index, line) in ["w", "w nope", "w 65536", "w -1", "w +1", "w 1 extra", "missing 1"].iter().enumerate() {
            let (scratch, _) = fixture(&format!("invalid-floor-{index}"), KEPT, KEPT, line);
            let error = skein_codegen::patch::module(scratch.path()).expect_err("invalid wire line");
            assert_eq!(error, Error::Wire { family: if line.starts_with("missing") { "missing" } else { "w" }.into() });
        }
        for (index, line) in ["w 1", "w build", " # comment\n w 1 # floor\n"].iter().enumerate() {
            let (scratch, _) = fixture(&format!("valid-floor-{index}"), KEPT, KEPT, line);
            skein_codegen::patch::module(scratch.path()).expect("floor or build reads");
        }
    }
}
