//! Registered stores' families are passed over (data/codec/command.md, 2 and 7).

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeMap;
    use skein_codegen::diff::Rename;
    use skein_codegen::patch::{self, Error, Finding, Landed, Options, Outcome};
    use skein_scratch::Scratch;
    use std::fs;
    use std::path::{Path, PathBuf};

    const V1: &str = "family shelf 1\nrecord Book { id: u8 }\nenum State { ready }\n";
    const V2: &str = "family shelf 2\nrecord Book { id: u8, note: option text 8 }\nenum State { ready }\n";
    const PATCH: &str = "patch 1\nowner shop 1\n\nfamily shelf 1 to 2\nadd Book.note option text 8 none\n";
    const REGISTERED: &str = "# stored by layout\n\n  shelf  # the map family\n\n";

    fn put(root: &Path, path: &str, text: &str) {
        let path = root.join(path);
        fs::create_dir_all(path.parent().expect("file parent")).expect("directories");
        fs::write(path, text).expect("fixture file");
    }

    fn options() -> Options {
        Options { owner: Some("shop".into()), ..Options::default() }
    }

    fn fixture(name: &str) -> (Scratch, Landed) {
        let scratch = Scratch::new(&format!("patch-registered-{name}"));
        let root = scratch.path();
        for (path, source) in [
            ("schema/shelf-v2.schema", V2),
            ("schema/kept/shelf-v1.schema", V1),
            ("schema/kept/shelf-v2.schema", V2),
            ("patches/0001.patch", PATCH),
        ] {
            put(root, path, source);
        }
        let landed = Landed::read(root).expect("landed history");
        patch::run(root, &landed, &options()).expect("generate module and context");
        put(root, "schema/registered", REGISTERED);
        let landed = Landed::read(root).expect("landed generated files");
        (scratch, landed)
    }

    fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
        let mut files = BTreeMap::new();
        let mut directories = vec![PathBuf::new()];
        while let Some(directory) = directories.pop() {
            for entry in fs::read_dir(root.join(&directory)).expect("directory") {
                let entry = entry.expect("entry");
                let path = directory.join(entry.file_name());
                if entry.path().is_dir() {
                    directories.push(path);
                } else {
                    files.insert(path, fs::read(entry.path()).expect("file"));
                }
            }
        }
        files
    }

    fn registered_finding(root: &Path, landed: &Landed) {
        let before = snapshot(root);
        assert_eq!(patch::check(root, landed), Ok(vec![Finding::Registered { family: "shelf".into() }]));
        assert_eq!(snapshot(root), before, "the check writes nothing");
    }

    #[test]
    fn registered_layout_changes_are_passed_over_without_writes() {
        for (name, body) in [
            ("option", "record Book { id: u8, note: option text 8, score: option u32 }\nenum State { ready }\n"),
            ("variant", "record Book { id: u8, note: option text 8 }\nenum State { pending, ready }\n"),
            ("rename", "record Volume { id: u8, note: option text 8 }\nenum State { ready }\n"),
        ] {
            let (scratch, landed) = fixture(name);
            let root = scratch.path();
            put(root, "schema/shelf-v2.schema", &format!("family shelf 2\n{body}"));
            let before = snapshot(root);
            let report = patch::run(
                root,
                &landed,
                &Options {
                    renames: if name == "rename" {
                        vec![("shelf".into(), Rename::Declaration { old: "Book".into(), new: "Volume".into() })]
                    } else {
                        vec![]
                    },
                    ..options()
                },
            )
            .expect("registered family");
            assert_eq!(report.outcomes, [Outcome::Registered { family: "shelf".into() }], "{name}");
            assert_eq!(report.patch, None);
            assert_eq!(report.proposed, []);
            assert_eq!(report.asked, Vec::<String>::new());
            assert_eq!(report.restored, Vec::<PathBuf>::new());
            assert_eq!(report.findings, []);
            assert_eq!(patch::check(root, &landed), Ok(vec![]));
            assert_eq!(snapshot(root), before, "{name}: no file written or removed");
        }
    }

    #[test]
    fn registered_current_version_must_stay_at_latest_landed() {
        let (scratch, landed) = fixture("version");
        let root = scratch.path();
        fs::remove_file(root.join("schema/shelf-v2.schema")).expect("move version");
        put(root, "schema/shelf-v3.schema", &V2.replace("family shelf 2", "family shelf 3"));
        registered_finding(root, &landed);
    }

    #[test]
    fn registered_branch_patch_block_is_found_and_not_regenerated() {
        let (scratch, landed) = fixture("block");
        let root = scratch.path();
        put(root, "patches/0002.patch", "patch 1\nowner shop 2\n\nfamily shelf 2 to 3\n");
        registered_finding(root, &landed);
        let report = patch::run(root, &landed, &options()).expect("pass over registered block");
        assert_eq!(report.outcomes, [Outcome::Registered { family: "shelf".into() }]);
        assert_eq!(report.patch, None);
        assert!(!root.join("patches/0002.patch").exists());
        assert_eq!(report.findings, []);
    }

    #[test]
    fn registered_branch_kept_text_is_found() {
        let (scratch, landed) = fixture("kept");
        put(scratch.path(), "schema/kept/shelf-v3.schema", &V2.replace("family shelf 2", "family shelf 3"));
        registered_finding(scratch.path(), &landed);
    }

    #[test]
    fn registered_branch_renames_are_found_but_landed_renames_stay() {
        let (scratch, landed) = fixture("renames");
        let root = scratch.path();
        put(root, "schema/kept/shelf.renames", "rename Book Volume\n");
        registered_finding(root, &landed);
        let landed = Landed::read(root).expect("landed waiting renames");
        let before = snapshot(root);
        assert_eq!(patch::check(root, &landed), Ok(vec![]));
        let report = patch::run(root, &landed, &options()).expect("leave landed renames alone");
        assert_eq!(report.findings, []);
        assert_eq!(snapshot(root), before);
        put(root, "schema/kept/shelf.renames", "rename Book Tome\n");
        registered_finding(root, &landed);
    }

    #[test]
    fn registered_manifest_requires_a_family_of_the_crate() {
        let (scratch, landed) = fixture("unknown");
        let root = scratch.path();
        put(root, "schema/registered", "# map families\n\nshelf\nunknown # not here\n");
        let before = snapshot(root);
        let expected = Error::Registered { family: "unknown".into() };
        assert_eq!(patch::run(root, &landed, &options()), Err(expected.clone()));
        assert_eq!(patch::check(root, &landed), Err(expected.clone()));
        assert_eq!(expected.to_string(), "schema/registered: unknown: no family of the crate");
        assert_eq!(snapshot(root), before);
    }

    #[test]
    fn branch_patch_contains_only_unregistered_families() {
        let (scratch, landed) = fixture("mixed");
        let root = scratch.path();
        let old = "family label 1\nrecord Label { id: u8 }\n";
        let new = "family label 2\nrecord Label { id: u8, note: option u8 }\n";
        put(root, "schema/kept/label-v1.schema", old);
        let mut files = landed
            .paths()
            .map(|path| (path.to_path_buf(), landed.get(path).expect("bytes").to_vec()))
            .collect::<BTreeMap<_, _>>();
        files.insert("schema/kept/label-v1.schema".into(), old.as_bytes().to_vec());
        let landed = Landed::new(files);
        put(root, "schema/label-v2.schema", new);
        put(root, "schema/shelf-v2.schema", &V2.replace("ready", "pending, ready"));
        let report = patch::run(root, &landed, &options()).expect("patch unregistered family only");
        assert_eq!(
            report.outcomes,
            [
                Outcome::Patched { family: "label".into(), from: 1, to: 2 },
                Outcome::Registered { family: "shelf".into() },
            ]
        );
        assert_eq!(report.patch, Some(2));
        assert_eq!(report.findings, []);
        assert_eq!(
            fs::read_to_string(root.join("patches/0002.patch")).expect("patch"),
            "patch 1\nowner shop 2\n\nfamily label 1 to 2\nadd Label.note option u8 none\n"
        );
        assert_eq!(fs::read_to_string(root.join("schema/kept/shelf-v2.schema")).expect("kept"), V2);
    }

    #[test]
    fn store_crates_register_their_map_families_and_check_cleanly() {
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        for (path, names) in [
            (
                "jig/crates/jig-protocol-store",
                "jig-store-core\njig-store-tasks\njig-store-people\njig-store-notes\njig-store-accounts\njig-store-settings\n",
            ),
            ("temper/crates/temper-protocol-store", "temper-store\n"),
        ] {
            let root = workspace.join(path);
            assert_eq!(fs::read_to_string(root.join("schema/registered")).expect("registered map families"), names);
            let landed = Landed::read(&root).expect("landed store crate");
            assert_eq!(patch::check(&root, &landed), Ok(vec![]), "{path}");
        }
    }
}
