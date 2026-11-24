//! The in-memory tree used by the patch command (skein/docs/design/data/codec/command.md, 8).

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeMap;
    use skein_codegen::patch::{Finding, Landed, Options, Tree, check, current_in, run};
    use std::path::{Path, PathBuf};

    const CURRENT: &str = "schema/shelf-v1.schema";
    const KEPT: &str = "schema/kept/shelf-v1.schema";
    const SCHEMA: &[u8] = b"family shelf 1\nrecord Book { id: u8 }\n";

    #[test]
    fn written_golden_is_listed_until_it_is_removed() {
        const GOLDEN: &str = "patches/golden/0001/shelf/a.bin";
        const BYTES: &[u8] = b"\x01";
        let mut tree = Tree::new(BTreeMap::new());
        tree.write(Path::new(GOLDEN), BYTES).expect("write golden into new directories");
        assert_eq!(tree.files(), Ok(BTreeMap::from([(PathBuf::from(GOLDEN), BYTES.to_vec())])));
        tree.remove(Path::new(GOLDEN)).expect("remove last golden");
        assert_eq!(tree.read(Path::new(GOLDEN)), Ok(None));
        assert_eq!(tree.files(), Ok(BTreeMap::new()));
    }

    #[test]
    fn landed_holds_history_and_wire_manifest_not_current_schema() {
        const WIRES: &str = "schema/wires";
        const PATCH: &str = "patches/0001.patch";
        let history = BTreeMap::from([
            (PathBuf::from(KEPT), SCHEMA.to_vec()),
            (PathBuf::from(WIRES), b"shelf build\n".to_vec()),
            (PathBuf::from(PATCH), b"patch 1\nowner shop 1\n".to_vec()),
        ]);
        let mut files = history.clone();
        files.insert(PathBuf::from(CURRENT), SCHEMA.to_vec());
        assert_eq!(Tree::new(files).landed(), Ok(Landed::new(history)));
    }

    fn checked_without_changes(tree: &Tree) -> Vec<Finding> {
        let before = tree.clone();
        let findings = check(tree, &Landed::none()).expect("check readable tree");
        assert_eq!(*tree, before, "check must not change the tree");
        findings
    }

    #[test]
    fn baseline_keeps_schema_and_generates_module_without_check_writes() {
        let mut tree = Tree::new(BTreeMap::from([(PathBuf::from(CURRENT), SCHEMA.to_vec())]));
        assert_eq!(checked_without_changes(&tree), [Finding::Unkept { family: "shelf".into() }]);

        let report =
            run(&mut tree, &Landed::none(), &Options { baseline: true, ..Options::default() }).expect("baseline");
        assert_eq!(report.to_string(), "shelf: kept version 1, its first\n");
        assert_eq!(tree.read(Path::new(KEPT)), Ok(Some(SCHEMA.to_vec())));
        assert_eq!(current_in(&tree), Ok(()));
        assert_eq!(checked_without_changes(&tree), []);
    }
}
