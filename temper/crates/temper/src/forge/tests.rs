use crate::{
    directories::tests::{given, scratch},
    tests::configuration,
};
use std::{ffi::OsString, fs};
#[test]
fn commands_require_the_group_file_before_forge_effects() {
    let root = scratch("forge-no-settings");
    let directories = given(&root);
    fs::write(
        directories.document(),
        serde_json::to_vec(&configuration::check_document(&root)).expect("fixture value"),
    )
    .expect("fixture value");
    for action in ["fetch", "prepare", "reconcile", "sign-in"] {
        let args: Vec<OsString> = [action, "7"].into_iter().map(OsString::from).collect();
        let why = super::start(&directories, &args, None).expect_err("no settings");
        assert_eq!(why.line(), format!("temper forge {action}: forge 7 has no settings: box/groups/7\n"));
        assert!(!root.join("forge-7").exists());
    }
    fs::remove_dir_all(root).expect("fixture value");
}
