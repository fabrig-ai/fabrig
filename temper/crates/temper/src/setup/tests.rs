//! Machine templates are checked beside the seed's setup entries (contracts, 6).
use super::*;
use serde_json::Value;
use std::{fs, os::unix::fs::PermissionsExt};

fn document() -> Value {
    use core::num::NonZeroU16;
    use temper_init::{Content, Directories, Facts, Ports};
    let answer = temper_init::files(
        &Facts {
            binary: b"/bin/temper",
            git: Some(b"/usr/bin/git"),
            claude: None,
            login: Some(b"person"),
            ports: Ports {
                clients: NonZeroU16::new(7100).expect("nonzero clients port"),
                webhooks: NonZeroU16::new(7200).expect("nonzero webhooks port"),
                forge: NonZeroU16::new(8369).expect("nonzero fake forge port"),
                owned: NonZeroU16::new(3000).expect("nonzero owned forge port"),
                llm: NonZeroU16::new(8370).expect("nonzero LLM port"),
            },
        },
        &Directories { config: b"/config", data: b"/data" },
        &[1; 32],
        &[2; 32],
    )
    .expect("valid fixture value");
    let file = answer.files.iter().find(|file| file.name == b"temper.json").expect("valid fixture value");
    let Content::Bytes { bytes, .. } = &file.content else { panic!("document") };
    serde_json::from_slice(bytes).expect("valid fixture value")
}
fn template(entry: &Value) -> Value {
    let mut template = entry.clone();
    template.as_object_mut().expect("valid fixture value").remove("forge");
    template.as_object_mut().expect("valid fixture value").remove("start");
    template
}
fn machine_document() -> Value {
    let mut d = document();
    d["application"]["owned"] = template(&d["application"]["setup"][1]);
    d["application"]["demo"]["fake"] = template(&d["application"]["setup"][0]);
    d
}
fn read(d: &Value) -> Result<Deployment, String> {
    check(
        &serde_json::to_vec(d).expect("valid fixture value"),
        &config::Directories::given(b"/config", b"/data", b"/cache").expect("valid fixture value"),
    )
}
#[test]
fn the_setup_check_reads_the_owned_member() {
    let deployment = read(&machine_document()).expect("valid fixture value");
    let mut expected = deployment.checked.setup.entry(3).expect("valid fixture value").clone();
    expected.forge = 7;
    assert_eq!(deployment.owned.expect("valid fixture value").entry(7), expected);
    let demo = deployment.demo.expect("valid fixture value");
    let mut expected = deployment.checked.setup.entry(1).expect("valid fixture value").clone();
    expected.forge = 9;
    assert_eq!(demo.fake().expect("valid fixture value").entry(9), expected);
}
#[test]
fn the_init_template_reads_no_machine_member() {
    for null in [false, true] {
        let mut d = document();
        if null {
            d["application"]["owned"] = Value::Null;
            d["application"]["demo"]["fake"] = Value::Null;
        }
        let deployment = read(&d).expect("valid fixture value");
        assert!(deployment.owned.is_none());
        assert!(deployment.demo.expect("valid fixture value").fake().is_none());
    }
}
#[test]
fn the_machine_listeners_are_unique_among_other_kinds() {
    let source = machine_document();
    for pointer in [
        "/application/clients/listener/socket",
        "/application/webhooks/listener/socket",
        "/application/demo/llm/listen/socket",
        "/application/demo/fake/mode/fake/listen/socket",
    ] {
        let mut d = source.clone();
        d["application"]["owned"]["mode"]["owned"]["listen"]["socket"] =
            d.pointer(pointer).expect("valid fixture value").clone();
        assert_eq!(
            read(&d).expect_err("listener collision"),
            "application.owned.mode.owned.listen: must be unique\n",
            "{pointer}"
        );
    }
    let mut d = source;
    d["application"]["demo"]["fake"]["mode"]["fake"]["listen"]["socket"] =
        d["application"]["setup"][1]["mode"]["owned"]["listen"]["socket"].clone();
    assert_eq!(read(&d).expect_err("listener collision"), "application.demo.fake.mode.fake.listen: must be unique\n");
}

#[test]
fn an_entry_comes_from_the_document_before_its_file() {
    let root = crate::directories::tests::scratch("entry-document");
    let directories = crate::directories::tests::given(&root);
    let deployment = read(&document()).expect("checked document");
    let groups = crate::path(&directories.data).join("box/groups");
    fs::create_dir_all(&groups).expect("group directory");
    // A refused file must not prevent the document's entry from being used.
    fs::write(groups.join("3"), b"invalid JSON").expect("group file");
    assert_eq!(
        entry(&directories, &deployment.checked, 3).expect("document entry"),
        (deployment.checked.setup.entry(3).expect("owned entry").clone(), false)
    );
    fs::remove_dir_all(root).expect("scratch removed");
}

#[test]
fn an_entry_absent_from_both_names_the_file() {
    let root = crate::directories::tests::scratch("entry-absent");
    let directories = crate::directories::tests::given(&root);
    let deployment = read(&document()).expect("checked document");
    assert_eq!(
        entry(&directories, &deployment.checked, 7).expect_err("absent entry"),
        format!("the setup has no entry for it and {} is absent", root.join("data/box/groups/7").display())
    );
    assert!(!root.join("data").exists(), "lookup creates no files");
    fs::remove_dir_all(root).expect("scratch removed");
}

#[test]
fn a_refused_group_file_is_named() {
    let root = crate::directories::tests::scratch("entry-refused");
    let directories = crate::directories::tests::given(&root);
    let deployment = read(&document()).expect("checked document");
    let groups = root.join("data/box/groups");
    fs::create_dir_all(&groups).expect("group directory");
    let path = groups.join("7");
    let mut setup = document()["application"]["setup"][1].clone();
    setup["start"] = "asked".into();
    let bytes = serde_json::to_vec(&serde_json::json!({"setup": setup, "playground": false})).expect("group JSON");
    fs::write(&path, &bytes).expect("group file");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).expect("private group file");
    assert_eq!(
        entry(&directories, &deployment.checked, 7).expect_err("mismatched forge"),
        format!("{}: setup.forge: must match the group number\n", path.display())
    );
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).expect("public group file");
    let why = entry(&directories, &deployment.checked, 7).expect_err("unsafe file");
    assert!(why.starts_with(&format!("{}: ", path.display())), "{why}");
    assert!(why.contains("readable by others or reached through a link"), "{why}");
    fs::remove_dir_all(root).expect("scratch removed");
}

fn playground_file(root: &std::path::Path, number: u16, playground: bool) {
    let groups = root.join("data/box/groups");
    fs::create_dir_all(&groups).expect("groups directory");
    let path = groups.join(number.to_string());
    let mut setup = document()["application"]["setup"][0].clone();
    setup["forge"] = number.into();
    fs::write(
        &path,
        serde_json::to_vec(&serde_json::json!({"setup":setup,"playground":playground})).expect("group JSON"),
    )
    .expect("group file");
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).expect("private group file");
}

#[test]
fn the_playground_forge_is_its_group_files_number() {
    let root = crate::directories::tests::scratch("playground-forge");
    let directories = crate::directories::tests::given(&root);
    fs::write(directories.document(), serde_json::to_vec(&document()).expect("document JSON")).expect("document");
    playground_file(&root, 5, true);
    playground_file(&root, 12, false);
    assert_eq!(playground_forge(&directories).expect("playground forge"), Some(5));
    playground_file(&root, 9, true);
    assert_eq!(playground_forge(&directories).expect("highest playground forge"), Some(9));
    fs::remove_dir_all(root).expect("scratch removed");
}

#[test]
fn no_playground_file_is_none() {
    let root = crate::directories::tests::scratch("no-playground-forge");
    let directories = crate::directories::tests::given(&root);
    assert_eq!(playground_forge(&directories).expect("absent group directory"), None);
    assert!(!root.join("data").exists(), "lookup creates no files");
    fs::write(directories.document(), serde_json::to_vec(&document()).expect("document JSON")).expect("document");
    fs::create_dir_all(root.join("data/box/groups")).expect("empty group directory");
    assert_eq!(playground_forge(&directories).expect("empty group directory"), None);
    playground_file(&root, 3, false);
    assert_eq!(playground_forge(&directories).expect("no playground file"), None);
    fs::remove_dir_all(root).expect("scratch removed");
}

#[test]
fn a_refused_playground_group_file_is_named() {
    let root = crate::directories::tests::scratch("playground-group-refused");
    let directories = crate::directories::tests::given(&root);
    fs::write(directories.document(), serde_json::to_vec(&document()).expect("document JSON")).expect("document");
    playground_file(&root, 5, true);
    let path = root.join("data/box/groups/5");
    fs::write(&path, b"invalid JSON").expect("invalid group file");
    let why = playground_forge(&directories).expect_err("invalid playground file");
    assert!(why.starts_with(&format!("{}: ", path.display())), "{why}");
    fs::remove_dir_all(root).expect("scratch removed");
}
