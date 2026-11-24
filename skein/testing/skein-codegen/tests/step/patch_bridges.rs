//! Wire floors and down goldens (data/codec/bridges.md, 4 and 6).

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeMap;
    use skein_codec::patch::{Build, Halt, Old, RefusalIx, Site};
    use skein_codec::value::Node;
    use skein_codegen::patch::{self, Error, Finding, FloorWrong, Landed, Mode, Options, Tree};
    use std::{fs, path::Path};

    const OLD: &str = "family w 3\nrecord Task { title: u8 }\n";
    const NEW: &str = "family w 4\nrecord Task {}\n";
    const DROP: &str = "patch 1\nowner sample 1\n\nfamily w 3 to 4\ndrop Task.title\n";
    const DOWN: &str = "patch 1\nowner sample 1\n\nfamily w 3 to 4\ndrop Task.title\ndown Task.title 7\n";

    fn put(tree: &mut Tree, path: &str, text: &str) {
        tree.write(Path::new(path), text.as_bytes()).expect(path);
    }

    fn read_text(tree: &Tree, path: &str) -> String {
        String::from_utf8(tree.read(Path::new(path)).expect(path).expect(path)).expect(path)
    }

    fn text(tree: &Tree, path: &str, expected: &str) {
        assert_eq!(read_text(tree, path), expected, "{path}");
    }

    fn fixture(floor: &str, source: &str) -> Tree {
        Tree::new(
            [
                ("schema/w.schema", NEW),
                ("schema/kept/w-v3.schema", OLD),
                ("schema/kept/w-v4.schema", NEW),
                ("patches/0001.patch", source),
                ("schema/wires", floor),
            ]
            .into_iter()
            .map(|(path, source)| (path.into(), source.as_bytes().to_vec()))
            .collect(),
        )
    }

    fn floor_case(name: &str, floor: u16, source: &str, base_floor: Option<u16>, branch: bool, wrongs: &[FloorWrong]) {
        let wires = format!("w {floor}\n");
        let tree = fixture(&wires, source);
        let mut base = tree.landed().expect("landed");
        let mut files = base
            .paths()
            .map(|path| (path.to_path_buf(), base.get(path).expect("bytes").to_vec()))
            .collect::<BTreeMap<_, _>>();
        if branch {
            files.remove(Path::new("patches/0001.patch"));
            files.remove(Path::new("schema/kept/w-v4.schema"));
        }
        if let Some(floor) = base_floor {
            files.insert("schema/wires".into(), format!("w {floor}\n").into_bytes());
        } else {
            files.remove(Path::new("schema/wires"));
        }
        base = Landed::new(files);
        let expected = wrongs.iter().map(|wrong| Finding::Floor { family: "w".into(), floor, wrong: *wrong }).collect();
        assert_eq!(patch::check(&tree, &base), Ok(expected), "{name}");
        text(&tree, "schema/wires", &wires);
        text(&tree, "schema/w.schema", NEW);
        text(&tree, "schema/kept/w-v3.schema", OLD);
        text(&tree, "schema/kept/w-v4.schema", NEW);
        text(&tree, "patches/0001.patch", source);
    }

    #[test]
    fn floor_is_at_most_current_and_at_least_first_kept() {
        floor_case("above", 5, DOWN, None, false, &[FloorWrong::Above]);
        floor_case("current", 4, DOWN, None, false, &[]);
        floor_case("unkept", 2, DOWN, None, false, &[FloorWrong::Unkept]);
        floor_case("first", 3, DOWN, None, false, &[]);
    }

    #[test]
    fn landed_wire_names_without_floors_mean_the_landed_build() {
        let mut tree = fixture("w 4\n", DOWN);
        let mut files = BTreeMap::new();
        files.insert("schema/kept/w-v3.schema".into(), OLD.as_bytes().to_vec());
        files.insert("schema/kept/w-v4.schema".into(), NEW.as_bytes().to_vec());
        files.insert("patches/0001.patch".into(), DOWN.as_bytes().to_vec());
        files.insert("schema/wires".into(), b"w\n".to_vec());
        let base = Landed::new(files);
        assert_eq!(patch::check(&tree, &base), Ok(vec![]));
        put(&mut tree, "schema/wires", "w 3\n");
        assert_eq!(
            patch::check(&tree, &base),
            Ok(vec![Finding::Floor { family: "w".into(), floor: 3, wrong: FloorWrong::Lowered }])
        );
    }

    #[test]
    fn floor_never_lowers_the_landed_floor() {
        floor_case("lowered", 3, DOWN, Some(4), false, &[FloorWrong::Lowered]);
        floor_case("kept", 4, DOWN, Some(4), false, &[]);
        floor_case("raised-alone", 4, DOWN, Some(3), false, &[]);
    }

    #[test]
    fn moving_floor_stays_at_the_version_moved_from() {
        floor_case("passed", 4, DOWN, Some(3), true, &[FloorWrong::Passed]);
        floor_case("at-from", 3, DOWN, Some(3), true, &[]);
    }

    #[test]
    fn breaking_floor_is_the_second_version() {
        let source = format!("{DROP}breaking \"title is no longer meaningful\"\n");
        let tree = fixture("w 3\n", &source);
        let base = Landed::new([(Path::new("schema/kept/w-v3.schema").to_path_buf(), OLD.as_bytes().to_vec())].into());
        let findings = patch::check(&tree, &base).expect("check");
        assert_eq!(
            findings.first(),
            Some(&Finding::Floor { family: "w".into(), floor: 3, wrong: FloorWrong::Breaking })
        );
        assert!(findings.iter().any(
            |finding| matches!(finding, Finding::Unbridged { version: 4, error, .. } if error.ends_with(": Breaking"))
        ));
        floor_case("breaking-raised", 4, &source, Some(3), true, &[]);
        floor_case("breaking-past", 5, &source, Some(3), true, &[FloorWrong::Above, FloorWrong::Breaking]);
        text(&tree, "patches/0001.patch", &source);
        text(&tree, "schema/wires", "w 3\n");
    }

    #[test]
    fn every_reverse_above_floor_plans_but_build_wires_need_none() {
        let mut tree = fixture("w 3\n", DROP);
        let mut files = BTreeMap::new();
        files.insert("schema/kept/w-v3.schema".into(), OLD.as_bytes().to_vec());
        let base = Landed::new(files);
        assert_eq!(
            patch::check(&tree, &base),
            Ok(vec![Finding::Unbridged {
                family: "w".into(),
                version: 4,
                error: "line 5 at 45..49: Irreversible".into(),
            }])
        );
        text(&tree, "patches/0001.patch", DROP);
        put(&mut tree, "patches/0001.patch", DOWN);
        assert_eq!(patch::check(&tree, &base), Ok(vec![]));
        text(&tree, "patches/0001.patch", DOWN);
        put(&mut tree, "patches/0001.patch", DROP);
        put(&mut tree, "schema/wires", "w build\n");
        assert_eq!(patch::check(&tree, &base), Ok(vec![]));
        text(&tree, "schema/wires", "w build\n");
        let landed = tree.landed().expect("patch landed");
        put(&mut tree, "schema/wires", "w 4\n");
        assert_eq!(patch::check(&tree, &landed), Ok(vec![]));
        text(&tree, "schema/wires", "w 4\n");
        text(&tree, "patches/0001.patch", DROP);
        let latest = "family w 5\nrecord Task {}\n";
        put(&mut tree, "schema/w.schema", latest);
        put(&mut tree, "schema/kept/w-v5.schema", latest);
        put(&mut tree, "patches/0002.patch", "patch 1\nowner sample 2\n\nfamily w 4 to 5\n");
        put(&mut tree, "schema/wires", "w 5\n");
        let landed = tree.landed().expect("later floor and patch landed");
        assert_eq!(patch::check(&tree, &landed), Ok(vec![]));
        text(&tree, "patches/0001.patch", DROP);
        text(&tree, "schema/wires", "w 5\n");
    }

    fn no_site(site: Site, _old: &Old, _new: &mut Build, _at: Node, _cx: &mut ()) -> Result<(), Halt> {
        Err(Halt::NoSite(site))
    }

    #[test]
    fn run_asks_three_ways_and_keeps_the_authors_down_or_breaking() {
        let mut tree = fixture("w 3\n", DROP);
        let base = Landed::new([("schema/kept/w-v3.schema".into(), OLD.as_bytes().to_vec())].into());
        let options = Options { owner: Some("sample".into()), ..Options::default() };
        let report = patch::run(&mut tree, &base, &options).expect("asks reverse");
        assert_eq!(
            report.asked,
            [
                "patch 0001, w: drop Task.title: give `down Task.title VALUE`, write `down written Task`, or declare the block breaking; bridgeable: keep Task.title beside its replacement until the floor passes it"
            ]
        );
        text(&tree, "patches/0001.patch", DROP);
        assert_eq!(tree.read(Path::new("patches/p0001.rs")), Ok(None));
        for source in [DOWN.to_owned(), format!("{DROP}breaking \"no title\"\n")] {
            put(&mut tree, "patches/0001.patch", &source);
            patch::run(&mut tree, &base, &options).expect("derived again");
            text(&tree, "patches/0001.patch", &source);
        }
    }

    #[expect(
        clippy::disallowed_methods,
        reason = "SKEIN_PATCH_WRITE regenerates the generated down-site module fixtures"
    )]
    fn module_fixture(tree: &Tree, name: &str) {
        let path = skein_scratch::paths::package().join("tests/fixtures").join(format!("patch-{name}.rs"));
        let actual = read_text(tree, "patches/mod.rs");
        if std::env::var_os("SKEIN_PATCH_WRITE").is_some_and(|value| value == "1") {
            fs::write(&path, &actual).expect("regenerate module fixture");
        }
        assert_eq!(actual, fs::read_to_string(path).expect("module fixture"));
    }

    #[test]
    fn down_sites_have_target_names_bodies_and_shared_dispatch_numbers() {
        let mut tree = fixture("w 3\n", DOWN);
        let base = Landed::new([("schema/kept/w-v3.schema".into(), OLD.as_bytes().to_vec())].into());
        let now = "family w 4\nrecord Task { title: text 1 }\n";
        put(&mut tree, "schema/w.schema", now);
        for target in ["Task", "Task.title"] {
            let source = format!(
                "patch 1\nowner sample 1\n\nfamily w 3 to 4\nretype Task.title u8 text 1\nwritten Task\ndown written {target}\nrefuses no_title \"no title\"\n"
            );
            put(&mut tree, "patches/0001.patch", &source);
            tree.remove(Path::new("patches/p0001.rs")).expect("fresh skeleton");
            let report = patch::run(&mut tree, &base, &Options::default()).expect("down skeleton");
            assert_eq!(
                report.asked,
                ["patch 0001, w: written Task".to_owned(), format!("patch 0001, w: down written {target}")]
            );
            let name = target.to_lowercase().replace('.', "_") + "_down";
            text(
                &tree,
                "patches/p0001.rs",
                &format!(
                    "//! Patch 0001: the command wrote the skeleton; its functions are the author's (data/codec/patches.md, 5).\n\nuse super::Context;\nuse skein_codec::patch::{{Build, Halt, Old, Site}};\nuse skein_codec::value::Node;\n\n/// Site 0, `written Task`, w 3 to 4.\n///\n/// - `retype Task.title u8 text 1`\npub fn task(_old: &Old, _new: &mut Build, _at: Node, _cx: &mut Context) -> Result<(), Halt> {{\n    Err(Halt::NoSite(Site::new(1, 0)))\n}}\n\n/// Down site 1, `down written {target}`, w 3 to 4.\n///\n/// - `retype Task.title u8 text 1`\n/// - `refuses no_title \"no title\"`\npub fn {name}(_old: &Old, _new: &mut Build, _at: Node, _cx: &mut Context) -> Result<(), Halt> {{\n    Err(Halt::NoSite(Site::new(1, 1)))\n}}\n"
                ),
            );
            let module = patch::module(&tree).expect("module");
            module_fixture(&tree, &name);
            patch::run(&mut tree, &base, &Options::default()).expect("derive again");
            text(&tree, "patches/0001.patch", &source);
            text(&tree, "patches/mod.rs", &module);
        }
    }

    fn fails_naming(tree: &mut Tree, path: &str, message: &str) {
        let Err(Error::Golden(failures)) = patch::golden(tree, 1, no_site, Mode::Check) else {
            unreachable!("goldens must fail")
        };
        let prefix = format!("{path}: {message}");
        assert!(failures.iter().any(|failure| failure.starts_with(&prefix)), "{failures:?}");
    }

    #[test]
    fn down_goldens_are_written_checked_and_name_missing_different_or_stale_files() {
        let mut tree = fixture("w 3\n", DOWN);
        let path = "patches/golden/0001/w/down/record_task_full.bin";
        fails_naming(&mut tree, path, "");
        patch::golden(&mut tree, 1, no_site, Mode::Write).expect("write both directions");
        assert_eq!(tree.read(Path::new(path)).expect("down bytes").expect("file present"), [7]);
        assert_eq!(patch::golden(&mut tree, 1, no_site, Mode::Check), Ok(()));
        tree.write(Path::new(path), &[8]).expect("stale bytes");
        fails_naming(&mut tree, path, "");
        patch::golden(&mut tree, 1, no_site, Mode::Write).expect("rewrite");
        put(&mut tree, "patches/golden/0001/w/down/stale.bin", "stale");
        fails_naming(&mut tree, "patches/golden/0001/w/down/stale.bin", "stale");
        patch::golden(&mut tree, 1, no_site, Mode::Write).expect("remove stale");
        assert_eq!(tree.read(Path::new("patches/golden/0001/w/down/stale.bin")), Ok(None));
        tree.remove(Path::new(path)).expect("remove");
        fails_naming(&mut tree, path, "");
        patch::golden(&mut tree, 1, no_site, Mode::Write).expect("restore");
        let before = tree.read(Path::new(path)).expect("held down golden").expect("file present");
        put(&mut tree, "schema/wires", "w 4\n");
        patch::golden(&mut tree, 1, no_site, Mode::Write).expect("raised floor leaves down alone");
        assert_eq!(tree.read(Path::new(path)).expect("still held").expect("file present"), before);
    }

    fn refuse(_site: Site, _old: &Old, new: &mut Build, _at: Node, _cx: &mut ()) -> Result<(), Halt> {
        Err(new.refuse(RefusalIx::new(0)))
    }

    fn lose(_site: Site, _old: &Old, new: &mut Build, _at: Node, _cx: &mut ()) -> Result<(), Halt> {
        new.put_uint(1)
    }

    #[test]
    fn down_goldens_hold_unheld_and_declared_refusals_and_reject_altered_round_trips() {
        let mut tree = fixture("w 3\n", DOWN);
        let now = "family w 4\nrecord Task { title: u8, score: u8 }\n";
        put(&mut tree, "schema/w.schema", now);
        put(&mut tree, "schema/kept/w-v4.schema", now);
        put(&mut tree, "patches/0001.patch", "patch 1\nowner sample 1\n\nfamily w 3 to 4\nadd Task.score u8 0\n");
        patch::golden(&mut tree, 1, no_site, Mode::Write).expect("unheld written");
        text(
            &tree,
            "patches/golden/0001/w/down/record_task_full.bin.refused",
            "unheld Path { decl: DeclIx(0), part: Field(FieldIx(1)) }\n",
        );
        assert_eq!(
            tree.read(Path::new("patches/golden/0001/w/down/record_task_smallest.bin"))
                .expect("held")
                .expect("file present"),
            [0]
        );
        assert_eq!(patch::golden(&mut tree, 1, no_site, Mode::Check), Ok(()));
        put(
            &mut tree,
            "patches/0001.patch",
            "patch 1\nowner sample 1\n\nfamily w 3 to 4\nadd Task.score u8 0\ndown written Task.title\nrefuses no_title \"no title\"\n",
        );
        patch::golden(&mut tree, 1, refuse, Mode::Write).expect("declared refusal written");
        text(&tree, "patches/golden/0001/w/down/record_task_smallest.bin.refused", "refuses no_title\n");
        assert_eq!(patch::golden(&mut tree, 1, refuse, Mode::Check), Ok(()));
        let Err(Error::Golden(failures)) = patch::golden(&mut tree, 1, lose, Mode::Write) else {
            unreachable!("round trip must fail")
        };
        assert!(
            failures
                .iter()
                .any(|failure| failure.starts_with("patches/golden/0001/w/down/record_task_smallest.bin: Altered")),
            "{failures:?}"
        );
    }
}
