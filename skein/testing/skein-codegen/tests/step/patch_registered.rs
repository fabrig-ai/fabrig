//! Registered stores' families are passed over (data/codec/command.md, 2 and 7).

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeMap;
    use skein_codegen::diff::Rename;
    use skein_codegen::patch::{self, Error, Finding, Landed, Options, Outcome, Tree};
    use std::path::{Path, PathBuf};

    const V1: &str = "family shelf 1\nrecord Book { id: u8 }\nenum State { ready }\n";
    const V2: &str = "family shelf 2\nrecord Book { id: u8, note: option text 8 }\nenum State { ready }\n";
    const PATCH: &str = "patch 1\nowner shop 1\n\nfamily shelf 1 to 2\nadd Book.note option text 8 none\n";
    const REGISTERED: &str = "# stored by layout\n\n  shelf  # the map family\n\n";

    fn put(tree: &mut Tree, path: &str, text: &str) {
        tree.write(Path::new(path), text.as_bytes()).expect("fixture file");
    }

    fn options() -> Options {
        Options { owner: Some("shop".into()), ..Options::default() }
    }

    fn fixture() -> (Tree, Landed) {
        let files = [
            ("schema/shelf-v2.schema", V2),
            ("schema/kept/shelf-v1.schema", V1),
            ("schema/kept/shelf-v2.schema", V2),
            ("patches/0001.patch", PATCH),
        ];
        let mut tree =
            Tree::new(files.into_iter().map(|(path, text)| (PathBuf::from(path), text.as_bytes().to_vec())).collect());
        let landed = tree.landed().expect("landed history");
        patch::run(&mut tree, &landed, &options()).expect("generate module and context");
        put(&mut tree, "schema/registered", REGISTERED);
        let landed = tree.landed().expect("landed generated files");
        (tree, landed)
    }

    fn registered_finding(tree: &Tree, landed: &Landed) {
        let before = tree.files().expect("tree files");
        assert_eq!(patch::check(tree, landed), Ok(vec![Finding::Registered { family: "shelf".into() }]));
        assert_eq!(tree.files().expect("tree files"), before, "the check writes nothing");
    }

    #[test]
    fn registered_layout_changes_are_passed_over_without_writes() {
        for (name, body) in [
            ("option", "record Book { id: u8, note: option text 8, score: option u32 }\nenum State { ready }\n"),
            ("variant", "record Book { id: u8, note: option text 8 }\nenum State { pending, ready }\n"),
            ("rename", "record Volume { id: u8, note: option text 8 }\nenum State { ready }\n"),
        ] {
            let (mut tree, landed) = fixture();
            put(&mut tree, "schema/shelf-v2.schema", &format!("family shelf 2\n{body}"));
            let before = tree.clone();
            let report = patch::run(
                &mut tree,
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
            assert_eq!(patch::check(&tree, &landed), Ok(vec![]));
            assert_eq!(tree, before, "{name}: no file written or removed");
        }
    }

    #[test]
    fn registered_current_version_must_stay_at_latest_landed() {
        let (mut tree, landed) = fixture();
        tree.remove(Path::new("schema/shelf-v2.schema")).expect("move version");
        put(&mut tree, "schema/shelf-v3.schema", &V2.replace("family shelf 2", "family shelf 3"));
        registered_finding(&tree, &landed);
    }

    #[test]
    fn registered_branch_patch_block_is_found_and_not_regenerated() {
        let (mut tree, landed) = fixture();
        put(&mut tree, "patches/0002.patch", "patch 1\nowner shop 2\n\nfamily shelf 2 to 3\n");
        registered_finding(&tree, &landed);
        let report = patch::run(&mut tree, &landed, &options()).expect("pass over registered block");
        assert_eq!(report.outcomes, [Outcome::Registered { family: "shelf".into() }]);
        assert_eq!(report.patch, None);
        assert_eq!(tree.read(Path::new("patches/0002.patch")).expect("branch patch"), None);
        assert_eq!(report.findings, []);
    }

    #[test]
    fn registered_branch_kept_text_is_found() {
        let (mut tree, landed) = fixture();
        put(&mut tree, "schema/kept/shelf-v3.schema", &V2.replace("family shelf 2", "family shelf 3"));
        registered_finding(&tree, &landed);
    }

    #[test]
    fn registered_branch_renames_are_found_but_landed_renames_stay() {
        let (mut tree, landed) = fixture();
        put(&mut tree, "schema/kept/shelf.renames", "rename Book Volume\n");
        registered_finding(&tree, &landed);
        let landed = tree.landed().expect("landed waiting renames");
        let before = tree.files().expect("tree files");
        assert_eq!(patch::check(&tree, &landed), Ok(vec![]));
        let report = patch::run(&mut tree, &landed, &options()).expect("leave landed renames alone");
        assert_eq!(report.findings, []);
        assert_eq!(tree.files().expect("tree files"), before);
        put(&mut tree, "schema/kept/shelf.renames", "rename Book Tome\n");
        registered_finding(&tree, &landed);
    }

    #[test]
    fn registered_manifest_requires_a_family_of_the_crate() {
        let (mut tree, landed) = fixture();
        put(&mut tree, "schema/registered", "# map families\n\nshelf\nunknown # not here\n");
        let before = tree.files().expect("tree files");
        let expected = Error::Registered { family: "unknown".into() };
        assert_eq!(patch::run(&mut tree, &landed, &options()), Err(expected.clone()));
        assert_eq!(patch::check(&tree, &landed), Err(expected.clone()));
        assert_eq!(expected.to_string(), "schema/registered: unknown: no family of the crate");
        assert_eq!(tree.files().expect("tree files"), before);
    }

    #[test]
    fn branch_patch_contains_only_unregistered_families() {
        let (mut tree, landed) = fixture();
        let old = "family label 1\nrecord Label { id: u8 }\n";
        let new = "family label 2\nrecord Label { id: u8, note: option u8 }\n";
        put(&mut tree, "schema/kept/label-v1.schema", old);
        let mut files = landed
            .paths()
            .map(|path| (path.to_path_buf(), landed.get(path).expect("bytes").to_vec()))
            .collect::<BTreeMap<_, _>>();
        files.insert("schema/kept/label-v1.schema".into(), old.as_bytes().to_vec());
        let landed = Landed::new(files);
        put(&mut tree, "schema/label-v2.schema", new);
        put(&mut tree, "schema/shelf-v2.schema", &V2.replace("ready", "pending, ready"));
        let report = patch::run(&mut tree, &landed, &options()).expect("patch unregistered family only");
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
            tree.read(Path::new("patches/0002.patch")).expect("patch").as_deref(),
            Some(b"patch 1\nowner shop 2\n\nfamily label 1 to 2\nadd Label.note option u8 none\n".as_slice())
        );
        assert_eq!(tree.read(Path::new("schema/kept/shelf-v2.schema")).expect("kept").as_deref(), Some(V2.as_bytes()));
    }

    #[test]
    fn registered_fixture_checks_cleanly() {
        let (tree, landed) = fixture();
        assert_eq!(patch::check(&tree, &landed), Ok(vec![]));
    }
}
