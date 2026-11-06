use hearth_tree_world::{
    edits,
    fs::{Fs, Node},
    layers,
};
use skein_lib::Rng;

/// Recounts names from the traversal, independently of the filesystem's indexes.
fn recount(fs: &Fs, previous: &[(Vec<u8>, usize)], seed: u64) {
    let paths = fs.paths();
    for (path, _) in paths.iter().chain(previous) {
        let wanted = paths.iter().find(|(found, _)| found == path).map(|(_, at)| *at);
        assert_eq!(fs.lookup(path), wanted, "lookup of {path:?}, seed {seed}");
    }
    for at in 0..fs.count() {
        let inode = fs.node(at).inode;
        let links = paths.iter().filter(|(path, child)| !path.is_empty() && fs.node(*child).inode == inode).count();
        assert_eq!(fs.links(inode), u32::try_from(links).expect("small").max(1), "inode {inode}, seed {seed}");
        let names = paths.iter().filter(|(path, child)| !path.is_empty() && *child == at).count();
        assert_eq!(fs.names_of(at), names, "node {at}, seed {seed}");
    }
}

#[test]
fn edited_layers_indexes_agree_with_a_recount_of_the_paths() {
    for seed in 0..12 {
        let mut rng = Rng::new(0x7EE0_7300 + seed);
        let mut fs = layers::layer(&mut rng, 40 + seed * 4);
        // Distinct nodes may share an inode number; their name counts remain separate.
        let inode = fs.last_inode() + 1;
        fs.add(b"same-one", Node::file(inode, b"one"));
        fs.add(b"same-two", Node::file(inode, b"two"));
        recount(&fs, &[], seed);
        let previous = fs.paths();
        edits::edit(&mut rng, &mut fs, 12, 24);
        recount(&fs, &previous, seed);
    }
}
