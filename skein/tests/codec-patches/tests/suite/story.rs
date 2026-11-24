//! Each landing supplies the next run's base (data/codec/command.md, 9).

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeMap;
    use skein_codec::patch::{Build, Halt, Old, Site};
    use skein_codec::value::Node;
    use skein_codec_patches::patches::{self, Context};
    use skein_codegen::diff::Rename;
    use skein_codegen::patch::{self, Error, Finding, Landed, Mode, Options, Outcome, Report, Tree};
    use std::fs;
    use std::path::{Path, PathBuf};

    const BASELINE: &str = "01-baseline";
    const FIELD: &str = "02-field";
    const RENAMES: &str = "03-renames";
    const WRITTEN: &str = "04-written";
    const RENUMBERED: &str = "05-renumbered";
    const WIRE: &str = "06-wire";
    // Two records and three enum variants, each smallest and full.
    const SHELF_GOLDENS: [&str; 10] = [
        "enum_format_audio_full.bin",
        "enum_format_audio_smallest.bin",
        "enum_format_ebook_full.bin",
        "enum_format_ebook_smallest.bin",
        "enum_format_paper_full.bin",
        "enum_format_paper_smallest.bin",
        "record_book_full.bin",
        "record_book_smallest.bin",
        "record_shelf_full.bin",
        "record_shelf_smallest.bin",
    ];
    // One Label record, smallest and full.
    const LABEL_GOLDENS: [&str; 2] = ["record_label_full.bin", "record_label_smallest.bin"];

    fn owner() -> PathBuf {
        skein_scratch::paths::package()
    }
    fn input(step: &str, name: &str) -> PathBuf {
        owner().join("story").join(step).join(name)
    }
    fn author(tree: &mut Tree, path: &str, step: &str, name: &str) {
        let bytes = fs::read(input(step, name)).expect("author's input");
        tree.write(Path::new(path), &bytes).expect("story file");
    }
    #[expect(
        clippy::disallowed_methods,
        reason = "reads SKEIN_PATCH_WRITE to regenerate the command's module fixtures"
    )]
    fn text(tree: &Tree, path: &str, step: &str, name: &str) {
        let bytes = tree.read(Path::new(path)).expect("read story file").expect("command's text");
        let actual = core::str::from_utf8(&bytes).expect("command's UTF-8 text");
        let write = std::env::var_os("SKEIN_PATCH_WRITE").is_some_and(|value| value == "1");
        if path == "patches/mod.rs" && write {
            fs::write(input(step, name), actual).expect("regenerate the command's module fixture");
        }
        assert_eq!(actual, fs::read_to_string(input(step, name)).expect("expected text"), "{step}: {path}");
    }
    fn files(tree: &Tree, directory: &str) -> BTreeMap<PathBuf, Vec<u8>> {
        tree.files().expect("story files").into_iter().filter(|(path, _)| path.starts_with(directory)).collect()
    }
    fn restore(tree: &mut Tree, landed: &Landed) {
        for path in landed.paths() {
            tree.write(path, landed.get(path).expect("landed bytes")).expect("restore landed file");
        }
    }
    fn clean(tree: &Tree, landed: &Landed) {
        assert_eq!(patch::check(tree, landed), Ok(vec![]), "step has no findings");
        assert_eq!(patch::current_in(tree), Ok(()), "step has the generated module");
    }
    fn golden_names<const N: usize>(tree: &Tree, directory: &str, names: [&str; N]) {
        assert_eq!(
            files(tree, directory).into_keys().collect::<Vec<_>>(),
            names.map(|name| Path::new(directory).join(name)),
            "{directory}: every golden, no stale files"
        );
    }
    #[expect(clippy::disallowed_methods, reason = "SKEIN_PATCH_WRITE regenerates the story's down golden fixtures")]
    fn down_files(tree: &Tree, number: u32) {
        let directory = format!("patches/golden/{number:04}/label/down");
        let actual = files(tree, &directory);
        let fixture = input(WIRE, &format!("golden/{number:04}"));
        if std::env::var_os("SKEIN_PATCH_WRITE").is_some_and(|value| value == "1") {
            for (path, bytes) in &actual {
                let name = path.file_name().expect("golden name");
                fs::create_dir_all(&fixture).expect("golden fixtures");
                fs::write(fixture.join(name), bytes).expect("regenerate down golden");
            }
        }
        let mut expected = fs::read_dir(&fixture)
            .expect("down golden fixtures")
            .map(|entry| entry.expect("down golden fixture").path())
            .collect::<Vec<_>>();
        expected.sort_unstable();
        assert!(!expected.is_empty(), "down goldens required");
        assert_eq!(actual.len(), expected.len(), "every down golden");
        for path in expected {
            let name = path.file_name().expect("golden name");
            let bytes = fs::read(&path).expect("expected down golden");
            assert_eq!(actual.get(&Path::new(&directory).join(name)), Some(&bytes), "{}", path.display());
        }
    }

    fn wire_steps(tree: &mut Tree) {
        // 6. A wire moves above its floor with a derived reverse and goldens down.
        let before = tree.landed().expect("written patch lands");
        author(tree, "schema/wires", WIRE, "wires");
        let marked = patch::run(tree, &before, &Options::default()).expect("mark label a wire");
        assert_eq!(marked, Report { outcomes: vec![current("label"), current("shelf")], ..Report::default() });
        text(tree, "schema/wires", WIRE, "wires");
        text(tree, "patches/mod.rs", WIRE, "marked-mod.rs");
        let marked = tree.landed().expect("wire floor lands");
        author(tree, "schema/label-v3.schema", WIRE, "label-v3.schema");
        let report = patch::run(tree, &marked, &Options::default()).expect("wire widens");
        assert_eq!(
            report,
            Report {
                outcomes: vec![Outcome::Patched { family: "label".into(), from: 2, to: 3 }, current("shelf")],
                patch: Some(4),
                ..Report::default()
            }
        );
        text(tree, "patches/0004.patch", WIRE, "0004.patch");
        text(tree, "schema/kept/label-v3.schema", WIRE, "label-v3.schema");
        text(tree, "patches/mod.rs", WIRE, "mod.rs");
        down_files(tree, 4);
        assert_eq!(patch::golden(tree, 4, patches::written, Mode::Check), Ok(()));
        clean(tree, &marked);
        let widened = tree.landed().expect("wire move lands");

        // 7. Dropping color needs the author's down value, never the command's.
        author(tree, "schema/label-v4.schema", WIRE, "label-v4.schema");
        let report = patch::run(tree, &widened, &Options::default()).expect("asks down value");
        assert_eq!(report.outcomes, [Outcome::Patched { family: "label".into(), from: 3, to: 4 }, current("shelf")]);
        assert_eq!(report.patch, Some(5));
        assert_eq!(
            report.asked,
            [
                "patch 0005, label: drop Label.color: give `down Label.color VALUE`, write `down written Label`, or declare the block breaking; bridgeable: keep Label.color beside its replacement until the floor passes it"
            ]
        );
        assert_eq!(
            report.findings,
            [Finding::Unbridged { family: "label".into(), version: 4, error: "line 5 at 48..53: Irreversible".into() }]
        );
        text(tree, "patches/0005.patch", WIRE, "asked.patch");
        text(tree, "schema/kept/label-v4.schema", WIRE, "label-v4.schema");
        text(tree, "patches/mod.rs", WIRE, "asked-mod.rs");
        assert!(files(tree, "patches/golden/0005").is_empty());
        author(tree, "patches/0005.patch", WIRE, "0005.patch");
        let report = patch::run(tree, &widened, &Options::default()).expect("down value kept");
        assert_eq!(report.asked, Vec::<String>::new());
        assert_eq!(report.findings, []);
        text(tree, "patches/0005.patch", WIRE, "0005.patch");
        text(tree, "patches/mod.rs", WIRE, "down-mod.rs");
        down_files(tree, 5);
        assert_eq!(patch::golden(tree, 5, patches::written, Mode::Check), Ok(()));
        clean(tree, &widened);
    }

    fn current(family: &str) -> Outcome {
        Outcome::Current { family: family.into() }
    }
    fn shelf_patch(from: u16, to: u16) -> Outcome {
        Outcome::Patched { family: "shelf".into(), from, to }
    }
    fn rename(family: &str, decl: &str, old: &str, new: &str) -> Options {
        Options {
            renames: vec![(family.into(), Rename::Part { decl: decl.into(), old: old.into(), new: new.into() })],
            ..Options::default()
        }
    }
    fn skeleton(site: Site, _old: &Old, _new: &mut Build, _at: Node, _cx: &mut Context) -> Result<(), Halt> {
        Err(Halt::NoSite(site))
    }
    // Step four's patch is still 2; the compiled author's module is its final 3.
    fn written(site: Site, old: &Old, new: &mut Build, at: Node, cx: &mut Context) -> Result<(), Halt> {
        patches::written(Site::new(3, site.index()), old, new, at, cx)
    }

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "data/codec/command.md 9 asks for one readable history, each run checked beside its inputs"
    )]
    fn owner_history_replays_from_baseline_through_rebased_written_patch() {
        let mut tree = Tree::new(BTreeMap::new());
        let options = Options::default();

        // 1. Before release the command requires an explicit baseline.
        author(&mut tree, "schema/shelf-v1.schema", BASELINE, "shelf-v1.schema");
        author(&mut tree, "schema/label-v1.schema", BASELINE, "label-v1.schema");
        let before = tree.clone();
        assert_eq!(patch::run(&mut tree, &Landed::none(), &options), Err(Error::Baseline));
        assert_eq!(tree, before, "refused run writes nothing");
        assert_eq!(
            patch::run(&mut tree, &Landed::none(), &Options { baseline: true, ..Options::default() }),
            Ok(Report {
                outcomes: vec![
                    Outcome::Kept { family: "label".into(), version: 1 },
                    Outcome::Kept { family: "shelf".into(), version: 1 }
                ],
                ..Report::default()
            })
        );
        text(&tree, "schema/kept/shelf-v1.schema", BASELINE, "shelf-v1.schema");
        text(&tree, "schema/kept/label-v1.schema", BASELINE, "label-v1.schema");
        text(&tree, "patches/mod.rs", BASELINE, "mod.rs");
        text(&tree, "patches/context.rs", BASELINE, "context.rs");
        clean(&tree, &Landed::none());
        let baseline = tree.landed().expect("baseline lands");

        // 2. A new field first asks its author, then plans with zero pages.
        author(&mut tree, "schema/shelf-v2.schema", FIELD, "shelf-v2.schema");
        let unplanned =
            Finding::Unplanned { patch: 1, family: "shelf".into(), error: "line 5 at 62..63: Asked".into() };
        assert_eq!(
            patch::run(&mut tree, &baseline, &Options { owner: Some("skein".into()), ..Options::default() }),
            Ok(Report {
                outcomes: vec![current("label"), shelf_patch(1, 2)],
                patch: Some(1),
                asked: vec!["patch 0001, shelf: add Book.pages u16 ?".into()],
                findings: vec![unplanned.clone()],
                ..Report::default()
            })
        );
        text(&tree, "patches/0001.patch", FIELD, "asked.patch");
        text(&tree, "schema/kept/shelf-v2.schema", FIELD, "shelf-v2.schema");
        text(&tree, "patches/mod.rs", FIELD, "asked-mod.rs");
        assert_eq!(patch::check(&tree, &baseline), Ok(vec![unplanned]));
        assert!(files(&tree, "patches/golden/0001").is_empty(), "an unplanned patch writes no goldens");
        author(&mut tree, "patches/0001.patch", FIELD, "0001.patch");
        assert_eq!(
            patch::run(&mut tree, &baseline, &options),
            Ok(Report { outcomes: vec![current("label"), shelf_patch(1, 2)], patch: Some(1), ..Report::default() })
        );
        text(&tree, "patches/0001.patch", FIELD, "0001.patch");
        text(&tree, "patches/mod.rs", FIELD, "mod.rs");
        golden_names(&tree, "patches/golden/0001/shelf", SHELF_GOLDENS);
        assert_eq!(patch::golden(&mut tree, 1, patches::written, Mode::Check), Ok(()));
        clean(&tree, &baseline);
        let field = tree.landed().expect("added pages lands");

        // 3. Label can refresh; shelf's frozen v2 keeps its rename waiting.
        author(&mut tree, "schema/label-v1.schema", RENAMES, "label-v1.schema");
        assert_eq!(
            patch::run(&mut tree, &field, &rename("label", "Label", "text", "caption")),
            Ok(Report {
                outcomes: vec![Outcome::Refreshed { family: "label".into(), version: 1 }, current("shelf")],
                ..Report::default()
            })
        );
        text(&tree, "schema/kept/label-v1.schema", RENAMES, "label-v1.schema");
        text(&tree, "patches/0001.patch", FIELD, "0001.patch");
        text(&tree, "patches/mod.rs", RENAMES, "mod.rs");
        clean(&tree, &field);
        author(&mut tree, "schema/shelf-v2.schema", RENAMES, "shelf-v2.schema");
        let waiting = Outcome::Waiting {
            family: "shelf".into(),
            renames: vec![Rename::Part { decl: "Book".into(), old: "title".into(), new: "name".into() }],
        };
        assert_eq!(
            patch::run(&mut tree, &field, &rename("shelf", "Book", "title", "name")),
            Ok(Report { outcomes: vec![current("label"), waiting.clone()], ..Report::default() })
        );
        text(&tree, "schema/kept/shelf.renames", RENAMES, "shelf.renames");
        text(&tree, "schema/kept/shelf-v2.schema", FIELD, "shelf-v2.schema");
        text(&tree, "patches/mod.rs", RENAMES, "mod.rs");
        clean(&tree, &field);
        let renames = tree.landed().expect("renames land");

        // 4. Audio is dropped. The skeleton reports its site; the author then
        // replaces it and reviews the results that will live under 0003.
        author(&mut tree, "schema/shelf-v3.schema", WRITTEN, "shelf-v3.schema");
        assert_eq!(
            patch::run(&mut tree, &renames, &options),
            Ok(Report {
                outcomes: vec![current("label"), shelf_patch(2, 3)],
                patch: Some(2),
                asked: vec!["patch 0002, shelf: written Format".into()],
                ..Report::default()
            })
        );
        text(&tree, "patches/0002.patch", WRITTEN, "0002.patch");
        text(&tree, "patches/p0002.rs", WRITTEN, "skeleton.rs");
        text(&tree, "patches/mod.rs", WRITTEN, "mod.rs");
        text(&tree, "schema/kept/shelf-v3.schema", WRITTEN, "shelf-v3.schema");
        assert_eq!(tree.read(Path::new("schema/kept/shelf.renames")), Ok(None), "patch consumes waiting rename");
        clean(&tree, &renames);
        let Err(Error::Golden(failures)) = patch::golden(&mut tree, 2, skeleton, Mode::Check) else {
            unreachable!("skeleton must fail its goldens")
        };
        // Successful Book conversions also report their two not-yet-reviewed files.
        // Keep the path and omit the platform's filesystem error wording.
        let failures = failures
            .into_iter()
            .map(|failure| {
                if failure.starts_with("patches/golden/") {
                    failure.split_once(": ").expect("file failure has its reason").0.to_owned()
                } else {
                    failure
                }
            })
            .collect::<Vec<_>>();
        let mut expected = [
            "enum_format_paper_smallest.bin",
            "enum_format_paper_full.bin",
            "enum_format_audio_smallest.bin",
            "enum_format_audio_full.bin",
            "enum_format_ebook_smallest.bin",
            "enum_format_ebook_full.bin",
            "record_shelf_smallest.bin",
            "record_shelf_full.bin",
        ]
        .map(|name| format!("patch 2, shelf/{name}: patch 2 site 0 (written Format)"))
        .to_vec();
        expected.extend([
            "patches/golden/0002/shelf/record_book_full.bin".into(),
            "patches/golden/0002/shelf/record_book_smallest.bin".into(),
        ]);
        assert_eq!(failures, expected);
        author(&mut tree, "patches/p0002.rs", WRITTEN, "p0002.rs");
        let committed = Tree::disk(&owner()).files().expect("committed owner files");
        for (path, bytes) in committed.iter().filter(|(path, _)| path.starts_with("patches/golden/0003/shelf")) {
            let dest = Path::new("patches/golden/0002/shelf").join(path.file_name().expect("golden name"));
            tree.write(&dest, bytes).expect("reviewed shelf golden");
        }
        golden_names(&tree, "patches/golden/0002/shelf", SHELF_GOLDENS);
        assert_eq!(patch::golden(&mut tree, 2, written, Mode::Check), Ok(()));

        // 5. Main lands label's color while the branch's written change waits.
        // The replay tree keeps the branch's side of patches/ and schema/kept/;
        // the current schemas merge main's label v2 before the command runs.
        let mut main_tree = Tree::new(BTreeMap::new());
        restore(&mut main_tree, &renames);
        author(&mut main_tree, "schema/shelf-v2.schema", RENAMES, "shelf-v2.schema");
        author(&mut main_tree, "schema/label-v2.schema", RENUMBERED, "label-v2.schema");
        let label_asked = patch::run(&mut main_tree, &renames, &options).expect("main asks color");
        assert_eq!(label_asked.asked, ["patch 0002, label: add Label.color u8 ?"]);
        author(&mut main_tree, "patches/0002.patch", RENUMBERED, "0002.patch");
        assert_eq!(
            patch::run(&mut main_tree, &renames, &options),
            Ok(Report {
                outcomes: vec![Outcome::Patched { family: "label".into(), from: 1, to: 2 }, waiting],
                patch: Some(2),
                ..Report::default()
            })
        );
        text(&main_tree, "patches/0002.patch", RENUMBERED, "0002.patch");
        text(&main_tree, "schema/kept/label-v2.schema", RENUMBERED, "label-v2.schema");
        text(&main_tree, "patches/mod.rs", RENUMBERED, "mod.rs");
        golden_names(&main_tree, "patches/golden/0002/label", LABEL_GOLDENS);
        clean(&main_tree, &renames);
        let main = main_tree.landed().expect("main's color lands first");
        author(&mut tree, "schema/label-v2.schema", RENUMBERED, "label-v2.schema");
        assert_eq!(
            patch::run(&mut tree, &main, &options),
            Ok(Report {
                outcomes: vec![current("label"), shelf_patch(2, 3)],
                patch: Some(3),
                moved: Some(2),
                asked: vec!["patch 0003, shelf: written Format".into()],
                restored: vec![
                    "patches/0002.patch".into(),
                    "patches/golden/0002/label/record_label_full.bin".into(),
                    "patches/golden/0002/label/record_label_smallest.bin".into(),
                    "schema/kept/label-v2.schema".into()
                ],
                ..Report::default()
            })
        );
        text(&tree, "patches/0002.patch", RENUMBERED, "0002.patch");
        assert_eq!(tree.read(Path::new("patches/p0002.rs")), Ok(None), "written function moved");
        assert!(files(&tree, "patches/golden/0002/shelf").is_empty(), "reviewed shelf goldens moved");
        golden_names(&tree, "patches/golden/0003/shelf", SHELF_GOLDENS);
        golden_names(&tree, "patches/golden/0002/label", LABEL_GOLDENS);
        clean(&tree, &main);
        let replayed = tree.files().expect("replayed owner files");
        assert_eq!(
            replayed.keys().collect::<Vec<_>>(),
            committed.keys().collect::<Vec<_>>(),
            "final schema/ and patches/ paths"
        );
        for (path, bytes) in &committed {
            let result = replayed.get(path).expect("same paths");
            if let (Ok(result), Ok(bytes)) = (core::str::from_utf8(result), core::str::from_utf8(bytes)) {
                assert_eq!(result, bytes, "final {}", path.display());
            } else {
                assert_eq!(result, bytes, "final {}", path.display());
            }
        }
        wire_steps(&mut tree);
    }
}
