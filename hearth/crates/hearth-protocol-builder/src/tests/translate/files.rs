use crate::render::parts::config;
#[test]
fn each_node_translates_to_the_records_and_back() {
    for (row, input) in crate::tests::fixtures::files::tree_nodes().into_iter().enumerate() {
        let translated = crate::translate::parts::config::config(config::Config {
            presets: Box::new([]),
            tmpfiles: Box::new([]),
            scripts: Box::new([]),
            groups: Box::new([]),
            users: Box::new([]),
            members: Box::new([]),
            tree: Box::new([config::TreeEntry { path: b"/entry".as_slice().into(), node: input.clone() }]),
        });
        let back = super::config::config_back(translated);
        assert_eq!(&crate::tests::fixtures::only(&back.tree).node, &input, "row {row}: node");
    }
}
