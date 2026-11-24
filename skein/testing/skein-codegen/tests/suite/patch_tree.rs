//! The patch command leaves the same files on disk and in memory.

#[cfg(test)]
mod tests {
    use skein_codegen::patch::{Landed, Options, Report, Tree, run};
    use skein_scratch::Scratch;
    use std::path::{Path, PathBuf};

    const OLD: &[u8] = b"family shelf 1\nrecord Book { id: u8 }\n";
    const NEW: &[u8] = b"family shelf 2\nrecord Book { id: u8, note: option text 8 }\n";
    const CURRENT: &str = "schema/shelf-v2.schema";
    const KEPT: &str = "schema/kept/shelf-v1.schema";
    const GOLDENS: &str = "patches/golden/0001";

    fn withdraw_patch(tree: &mut Tree, landed: &Landed) -> Report {
        tree.write(Path::new("schema/shelf-v1.schema"), OLD).expect("author restores previous schema");
        tree.remove(Path::new(CURRENT)).expect("author removes new schema");
        run(tree, landed, &Options::default()).expect("remove patch with no block left")
    }

    #[test]
    fn removing_branch_patch_matches_memory_and_prunes_disk_goldens() {
        const FILES: &[(&str, &[u8])] = &[
            (CURRENT, NEW),
            (KEPT, OLD),
            ("schema/kept/shelf-v2.schema", NEW),
            ("patches/0001.patch", b"patch 1\nowner shop 1\n\nfamily shelf 1 to 2\nadd Book.note option text 8 none\n"),
            ("patches/golden/0001/shelf/record_book_smallest.bin", b"\0\0"),
        ];
        let scratch = Scratch::new("patch-tree-disk");
        let mut disk = Tree::disk(scratch.path());
        for &(path, bytes) in FILES {
            disk.write(Path::new(path), bytes).expect("seed disk tree");
        }
        let mut memory = Tree::new(FILES.iter().map(|&(path, bytes)| (PathBuf::from(path), bytes.to_vec())).collect());
        let landed = Landed::new([(PathBuf::from(KEPT), OLD.to_vec())].into_iter().collect());

        assert_eq!(withdraw_patch(&mut disk, &landed), withdraw_patch(&mut memory, &landed));
        assert_eq!(disk.files().expect("disk files"), memory.files().expect("memory files"));
        assert!(!scratch.path().join(GOLDENS).exists(), "empty patch golden directory pruned");
    }
}
