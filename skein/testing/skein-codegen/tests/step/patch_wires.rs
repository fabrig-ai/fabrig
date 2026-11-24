//! The wire rule from data/codec/command.md, 2, over owners' trees in memory.

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeMap;
    use skein_codec::patch::{Build, Halt, Old, Site};
    use skein_codec::value::Node;
    use skein_codegen::diff::{Rename, Version};
    use skein_codegen::patch::{
        Error, Finding, Landed, Mode, Options, Outcome, Tree, check, current_in, golden, module, run,
    };
    use std::path::{Path, PathBuf};

    const KEPT: &str = "family w 1\nenum E { a, b }\nrecord R { id: u8 }\n";
    const WIRES: &str = "# wire families\n\n  w 1  # channel bodies\n\n";
    const EDITS: &[(&str, &str, &str)] = &[
        ("variant", "enum E { a, b, c }\nrecord R { id: u8 }\n", "add E.c\n"),
        ("added", "enum E { a, b }\nrecord R { id: u8 }\nrecord S { flag: bool }\n", "add S\n"),
        ("dropped", "enum E { a, b }\n", "drop R\n"),
        ("reordered", "record R { id: u8 }\nenum E { a, b }\n", ""),
    ];

    fn put(tree: &mut Tree, path: &str, text: &str) {
        tree.write(Path::new(path), text.as_bytes()).expect(path);
    }

    fn text(tree: &Tree, path: &str, expected: &str) {
        assert_eq!(tree.read(Path::new(path)).expect(path).as_deref(), Some(expected.as_bytes()), "{path}");
    }

    fn fixture(kept: &str, now: &str, wires: &str) -> (Tree, Landed) {
        let mut tree = Tree::new(BTreeMap::new());
        put(&mut tree, "schema/kept/w-v1.schema", kept);
        let base = tree.landed().expect("landed kept text, no patch");
        put(&mut tree, "schema/w-v1.schema", now);
        put(&mut tree, "schema/wires", wires);
        (tree, base)
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
            let (mut tree, base) = fixture(KEPT, &now, WIRES);
            let findings = vec![
                Finding::Unpatched { family: "w".into(), version: 1 },
                Finding::Version { family: "w".into(), verdict: Version::Unmoved, kept: 1, current: 1 },
            ];
            assert_eq!(check(&tree, &base), Ok(findings.clone()), "{name}");
            text(&tree, "schema/w-v1.schema", &now);
            text(&tree, "schema/kept/w-v1.schema", KEPT);
            let report = run(&mut tree, &base, &options()).expect("refused family");
            assert_eq!(report.outcomes, [Outcome::Refused { family: "w".into(), verdict: Version::Unmoved }]);
            assert_eq!(report.findings, findings);
            assert_eq!(report.patch, None);
            text(&tree, "schema/w-v1.schema", &now);
            text(&tree, "schema/kept/w-v1.schema", KEPT);
            assert_eq!(tree.read(Path::new("patches/0001.patch")), Ok(None));

            let moved = format!("family w 2\n{body}");
            tree.remove(Path::new("schema/w-v1.schema")).expect("move current filename");
            put(&mut tree, "schema/w-v2.schema", &moved);
            let report = run(&mut tree, &base, &options()).expect("wire patch");
            assert_eq!(report.outcomes, [Outcome::Patched { family: "w".into(), from: 1, to: 2 }]);
            assert_eq!(report.patch, Some(1));
            assert_eq!(report.asked, Vec::<String>::new());
            assert_eq!(report.findings, [], "{name}: the block plans without a site");
            text(&tree, "patches/0001.patch", &format!("patch 1\nowner sample 1\n\nfamily w 1 to 2\n{lines}"));
            text(&tree, "schema/kept/w-v1.schema", KEPT);
            text(&tree, "schema/kept/w-v2.schema", &moved);
            text(&tree, "schema/w-v2.schema", &moved);
            assert_eq!(tree.read(Path::new("patches/p0001.rs")), Ok(None));
            assert_eq!(golden(&mut tree, 1, unwritten, Mode::Check), Ok(()));
            for (file, bytes) in [
                ("enum_e_a_full.bin", &[0_u8][..]),
                ("enum_e_a_smallest.bin", &[0_u8][..]),
                ("enum_e_b_full.bin", &[1_u8][..]),
                ("enum_e_b_smallest.bin", &[1_u8][..]),
            ] {
                let path = Path::new("patches/golden/0001/w").join(file);
                assert_eq!(tree.read(&path).expect(file).as_deref(), Some(bytes), "{file}");
            }
            current_in(&tree).expect("generated module");
            let written = tree.landed().expect("branch files");
            run(&mut tree, &base, &options()).expect("derive again");
            assert_eq!(tree.landed().expect("unchanged files"), written);
        }
    }

    #[test]
    fn unlisted_layout_edits_still_refresh() {
        for (name, body, _) in EDITS {
            let now = format!("family w 1\n{body}");
            for wires in [None, Some("# no wire families\n"), Some("other 1\n")] {
                let (mut tree, base) = fixture(KEPT, &now, wires.unwrap_or_default());
                if wires.is_none() {
                    tree.remove(Path::new("schema/wires")).expect("absent wires");
                } else if wires == Some("other 1\n") {
                    put(&mut tree, "schema/other-v1.schema", "family other 1\nrecord Other {}\n");
                }
                let mut findings = Vec::new();
                if wires == Some("other 1\n") {
                    findings.push(Finding::Unkept { family: "other".into() });
                }
                findings.push(Finding::Lagging { family: "w".into(), version: 1 });
                assert_eq!(check(&tree, &base), Ok(findings), "{name}: {wires:?}");
                let report = run(&mut tree, &base, &options()).expect("nonwire refresh");
                assert!(report.outcomes.contains(&Outcome::Refreshed { family: "w".into(), version: 1 }));
                assert_eq!(report.patch, None);
                assert_eq!(report.findings, []);
                text(&tree, "schema/kept/w-v1.schema", &now);
                text(&tree, "schema/w-v1.schema", &now);
                assert_eq!(tree.read(Path::new("patches/0001.patch")), Ok(None));
            }
        }
    }

    #[test]
    fn wire_renames_wait_at_first_landed_text() {
        const RENAMED: &str = "family w 1\nenum State { a, b }\nrecord R { key: u8 }\n";
        let (mut tree, base) = fixture(KEPT, RENAMED, WIRES);
        let renames = vec![
            Rename::Declaration { old: "E".into(), new: "State".into() },
            Rename::Part { decl: "R".into(), old: "id".into(), new: "key".into() },
        ];
        let report = run(
            &mut tree,
            &base,
            &Options { renames: renames.iter().cloned().map(|rename| ("w".into(), rename)).collect(), ..options() },
        )
        .expect("confirmed renames wait");
        assert_eq!(report.outcomes, [Outcome::Waiting { family: "w".into(), renames }]);
        assert_eq!(report.findings, []);
        assert_eq!(report.patch, None);
        text(&tree, "schema/kept/w-v1.schema", KEPT);
        text(&tree, "schema/w-v1.schema", RENAMED);
        text(&tree, "schema/kept/w.renames", "rename E State\nrename R.id key\n");
        assert_eq!(check(&tree, &base), Ok(Vec::new()));
        let written = tree.landed().expect("waiting files");
        run(&mut tree, &base, &options()).expect("waiting renames read again");
        assert_eq!(tree.landed().expect("unchanged files"), written);
    }

    #[test]
    fn wire_nonlayout_edits_need_no_patch() {
        const OLD: &str = "family w 1\nrecord R { id: u8 t: text 8 }\nenum E { r: R, empty }\n";
        const NOW: &str = "# comment\nfamily\nw # header\n1\nrecord R was Before {\nid was key: u8 = 3\nt: text 8 = \"hi #é\"\n}\nenum E { r was record: boxed R, empty } # boxed is not layout\n";
        let (mut tree, base) = fixture(OLD, NOW, WIRES);
        assert_eq!(check(&tree, &base), Ok(Vec::new()));
        let report = run(&mut tree, &base, &options()).expect("no layout change");
        assert_eq!(report.outcomes, [Outcome::Current { family: "w".into() }]);
        assert_eq!(report.findings, []);
        assert_eq!(report.patch, None);
        text(&tree, "schema/kept/w-v1.schema", OLD);
        text(&tree, "schema/w-v1.schema", NOW);
        assert_eq!(tree.read(Path::new("patches/0001.patch")), Ok(None));
    }

    #[test]
    fn wire_versioned_fingerprints_set_aside_header_version() {
        const OLD: &str = "family w 1\nversioned record R { id: u8 }\nenum E { a, b }\n";
        const SAME: &str = "# header comment\nfamily\nw # family\n2\nversioned record R { id: u8 }\nenum E { a, b }\n";
        const NEXT: &str =
            "# header comment\nfamily\nw # family\n2\nversioned record R { id: u8 }\nenum E { a, b, c }\n";
        let (mut tree, base) = fixture(OLD, OLD, WIRES);
        tree.remove(Path::new("schema/w-v1.schema")).expect("move current filename");
        put(&mut tree, "schema/w-v2.schema", SAME);
        assert_eq!(
            check(&tree, &base),
            Ok(vec![Finding::Version { family: "w".into(), verdict: Version::Needless, kept: 1, current: 2 }])
        );
        let report = run(&mut tree, &base, &options()).expect("version alone is needless");
        assert_eq!(report.outcomes, [Outcome::Refused { family: "w".into(), verdict: Version::Needless }]);
        text(&tree, "schema/kept/w-v1.schema", OLD);
        text(&tree, "schema/w-v2.schema", SAME);
        put(&mut tree, "schema/w-v2.schema", NEXT);
        let report = run(&mut tree, &base, &options()).expect("layout changed");
        assert_eq!(report.outcomes, [Outcome::Patched { family: "w".into(), from: 1, to: 2 }]);
        assert_eq!(report.findings, []);
        text(&tree, "patches/0001.patch", "patch 1\nowner sample 1\n\nfamily w 1 to 2\nadd E.c\n");
        text(&tree, "schema/kept/w-v2.schema", NEXT);
        assert_eq!(golden(&mut tree, 1, unwritten, Mode::Check), Ok(()));
    }

    #[test]
    fn wire_branch_kept_texts_are_kept_afresh() {
        let mut tree = Tree::new(BTreeMap::new());
        put(&mut tree, "schema/wires", WIRES);
        put(&mut tree, "schema/w-v1.schema", KEPT);
        let report = run(&mut tree, &Landed::none(), &Options { baseline: true, ..options() }).expect("baseline");
        assert_eq!(report.outcomes, [Outcome::Kept { family: "w".into(), version: 1 }]);
        text(&tree, "schema/kept/w-v1.schema", KEPT);
        for version in [1, 2] {
            let now = format!("family w {version}\nenum E {{ a, b, c }}\n");
            put(&mut tree, &format!("schema/w-v{version}.schema"), &now);
            put(&mut tree, "schema/wires", &format!("w {version}\n"));
            let report = run(&mut tree, &Landed::none(), &options()).expect("not landed: keep afresh");
            assert_eq!(report.outcomes, [Outcome::Kept { family: "w".into(), version }]);
            assert_eq!(report.findings, []);
            assert_eq!(report.patch, None);
            text(&tree, &format!("schema/kept/w-v{version}.schema"), &now);
            assert_eq!(tree.read(Path::new("patches/0001.patch")), Ok(None));
        }
        assert_eq!(tree.read(Path::new("schema/kept/w-v1.schema")), Ok(None));
    }

    #[test]
    fn wire_unknown_family_is_an_error_without_writes() {
        let (mut tree, base) = fixture(KEPT, KEPT, "w 1\nunknown 1\n");
        let before = tree.landed().expect("snapshot includes schema/wires");
        let expected = Error::Wire { family: "unknown".into() };
        assert_eq!(expected.to_string(), "schema/wires: unknown: no family of the crate");
        assert_eq!(check(&tree, &base), Err(expected.clone()));
        assert_eq!(run(&mut tree, &base, &options()), Err(expected));
        text(&tree, "schema/wires", "w 1\nunknown 1\n");
        text(&tree, "schema/kept/w-v1.schema", KEPT);
        text(&tree, "schema/w-v1.schema", KEPT);
        assert_eq!(tree.landed().expect("no command files written"), before);
    }

    #[test]
    fn wire_landed_kept_edit_is_changed_and_restored() {
        let (mut tree, base) = fixture(KEPT, KEPT, WIRES);
        let commented = format!("# comment\n{KEPT}");
        put(&mut tree, "schema/kept/w-v1.schema", &commented);
        put(&mut tree, "schema/w-v1.schema", &commented);
        assert_eq!(check(&tree, &base), Ok(vec![Finding::Changed { path: PathBuf::from("schema/kept/w-v1.schema") }]));
        text(&tree, "schema/kept/w-v1.schema", &commented);
        let report = run(&mut tree, &base, &options()).expect("restore wire baseline");
        assert_eq!(report.restored, [PathBuf::from("schema/kept/w-v1.schema")]);
        assert_eq!(report.outcomes, [Outcome::Current { family: "w".into() }]);
        assert_eq!(report.findings, []);
        text(&tree, "schema/kept/w-v1.schema", KEPT);
        text(&tree, "schema/w-v1.schema", &commented);
    }
    #[test]
    fn wire_lines_require_a_numeric_floor_or_build() {
        for line in ["w", "w nope", "w 65536", "w -1", "w +1", "w 1 extra", "missing 1"] {
            let (tree, _) = fixture(KEPT, KEPT, line);
            let error = module(&tree).expect_err("invalid wire line");
            assert_eq!(error, Error::Wire { family: if line.starts_with("missing") { "missing" } else { "w" }.into() });
        }
        for line in ["w 1", "w build", " # comment\n w 1 # floor\n"] {
            let (tree, _) = fixture(KEPT, KEPT, line);
            module(&tree).expect("floor or build reads");
        }
    }
}
