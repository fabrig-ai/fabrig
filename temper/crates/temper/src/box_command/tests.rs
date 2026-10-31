use super::*;
use crate::tests::configuration;
use std::{
    fs,
    os::unix::fs::{MetadataExt, PermissionsExt},
};

fn fixture() -> (skein_scratch::Scratch, super::super::Directories, super::super::setup::Deployment) {
    let scratch = skein_scratch::Scratch::new("box-groups");
    let dirs = configuration::directories(scratch.path());
    let value = configuration::demo_document(scratch.path(), "127.0.0.1:8370");
    let bytes = serde_json::to_vec(&value).expect("fixture JSON");
    fs::write(dirs.document(), &bytes).expect("fixture document");
    let deployment = crate::setup::check(&bytes, &dirs.jig()).expect("checked fixture deployment");
    (scratch, dirs, deployment)
}
fn machine(dirs: &crate::Directories, deployment: &crate::setup::Deployment) -> group::Machine {
    group::Machine {
        flags: dirs.flags(),
        directories: dirs.jig(),
        limits: deployment.checked.limits.clone(),
        llm: deployment.demo.as_ref().map(|demo| demo.llm().listen()),
        forgejo_listens: cfg!(target_os = "linux"),
    }
}
fn owned_setup() -> serde_json::Value {
    serde_json::json!({"forge":2,"directory":"forge-2","bot":{"login":"temper","email":"temper@localhost"},"mode":{"owned":{"forgejo":"/forgejo","runner":null,"listen":{"socket":"127.0.0.1:3000"},"person":{"login":"ada","email":"ada@localhost"}}}})
}
#[test]
fn the_bridge_writes_each_owned_and_fake_entry_once_and_records_the_unasked() {
    let (scratch, dirs, mut deployment) = fixture();
    let fake = deployment.checked.setup.entries[0].clone();
    let (_, bytes) = configuration::documents(scratch.path(), "/fixture/temper");
    let mut value: serde_json::Value = serde_json::from_slice(&bytes).expect("fixture JSON");
    let mut owned = owned_setup();
    owned["forge"] = 2.into();
    owned["start"] = "asked".into();
    let mut unasked = owned.clone();
    unasked["forge"] = 3.into();
    unasked.as_object_mut().expect("setup object").remove("start");
    let demo_bytes: serde_json::Value =
        serde_json::from_slice(&fs::read(dirs.document()).expect("fixture document")).expect("box group fixture");
    let demo_entry = demo_bytes["application"]["setup"][0].clone();
    let mut fake_other = demo_entry.clone();
    fake_other["forge"] = 4.into();
    let mut managed = owned.clone();
    managed["forge"] = 5.into();
    managed["mode"] = serde_json::json!("managed");
    managed.as_object_mut().expect("setup object").remove("directory");
    managed.as_object_mut().expect("setup object").remove("start");
    value["application"]["setup"] = serde_json::json!([demo_entry, owned, unasked, fake_other, managed]);
    let bytes = serde_json::to_vec_pretty(&value).expect("fixture JSON");
    fs::write(dirs.document(), &bytes).expect("fixture document");
    let document = jig_protocol_config::document(
        &bytes,
        &jig_protocol_config::DocumentLimits { bytes: crate::CONFIG_BYTES, tokens: crate::CONFIG_BYTES, depth: 32 },
    )
    .expect("box group fixture");
    let application = crate::member(&document, 0, b"application");
    let setup = crate::member(&document, application, b"setup");
    let mut problems = temper_forge_setup::problem::Problems::empty();
    deployment.checked.setup =
        temper_forge_setup::Setup::read(&document, setup, &deployment.checked.limits, &dirs.jig(), &mut problems)
            .unwrap_or_else(|| panic!("{}", String::from_utf8_lossy(&temper_forge_setup::problem::lines(&problems))));
    assert_eq!(deployment.checked.setup.entries[0], fake);
    bridge_groups(&dirs, &deployment).expect("bridge writes private group files");
    for forge in [1, 2, 3, 4] {
        let path = scratch.path().join(format!("box/groups/{forge}"));
        assert_eq!(fs::metadata(&path).expect("group metadata").mode() & 0o777, 0o600);
        let bytes = fs::read(&path).expect("group file");
        assert!(bytes.starts_with(b"{\"setup\":{\n        \"bot\": {"));
        let file = group::read(&bytes, forge, &machine(&dirs, &deployment)).expect("bridge group");
        assert_eq!(file.playground, forge == 1);
        assert_eq!(scratch.path().join(format!("box/started/{forge}")).exists(), forge == 3 || forge == 4);
        let metadata = fs::metadata(&path).expect("group metadata");
        bridge_groups(&dirs, &deployment).expect("bridge writes private group files");
        assert_eq!(fs::metadata(&path).expect("group metadata").ino(), metadata.ino(), "existing file kept");
        assert_eq!(fs::read(&path).expect("group file"), bytes, "existing bytes kept");
    }
    let record = scratch.path().join("box/started/3");
    fs::remove_file(&record).expect("bridge interrupted before its record");
    bridge_groups(&dirs, &deployment).expect("bridge resumes with an existing group file");
    assert!(record.exists(), "the unasked entry remains in the startup chain");
    assert!(!scratch.path().join("box/groups/5").exists());
    for dir in ["box", "box/groups", "box/started"] {
        assert_eq!(fs::metadata(scratch.path().join(dir)).expect("private directory metadata").mode() & 0o777, 0o700);
    }
}
#[test]
fn startup_reads_each_group_file_and_skips_an_invalid_one_with_a_line() {
    let (scratch, dirs, deployment) = fixture();
    bridge_groups(&dirs, &deployment).expect("bridge writes private group files");
    let directory = scratch.path().join("box/groups");
    fs::write(directory.join("2"), b"invalid").expect("box group fixture");
    fs::set_permissions(directory.join("2"), fs::Permissions::from_mode(0o600)).expect("box group fixture");
    fs::write(directory.join(".temporary"), b"invalid").expect("box group fixture");
    fs::write(directory.join("01"), b"invalid").expect("box group fixture");
    let machine = machine(&dirs, &deployment);
    assert!(startup_group(&directory.join("2"), 2, &machine).expect_err("invalid group file").contains("invalid form"));
    let groups = startup_groups(&dirs, &machine);
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].group.forge, 1);
}
#[test]
fn the_machine_ports_are_the_entries_and_the_demo_llm_listens() {
    let (_scratch, dirs, mut deployment) = fixture();
    let bytes =
        serde_json::to_vec(&serde_json::json!({"setup":owned_setup(),"playground":false})).expect("box group fixture");
    let owned = group::read(&bytes, 2, &machine(&dirs, &deployment)).expect("owned group").entry;
    let mut managed = deployment.checked.setup.entries[0].clone();
    managed.mode = temper_forge_setup::setup::Mode::Managed { admin_token: None };
    managed.forge = 3;
    deployment.checked.setup.entries =
        vec![deployment.checked.setup.entries[0].clone(), owned, managed].into_boxed_slice();
    let mut expected = vec!["127.0.0.1:9".parse().expect("fixture socket address")];
    if cfg!(target_os = "linux") {
        expected.push("127.0.0.1:3000".parse().expect("fixture socket address"));
    }
    expected.push("127.0.0.1:8370".parse().expect("fixture socket address"));
    assert_eq!(machine_ports(&deployment), expected);
}
