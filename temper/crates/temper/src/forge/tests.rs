use super::{group_entry, reconcile_settings, remove, start};
use crate::tests::configuration;
use std::{
    fs,
    os::unix::{
        ffi::OsStrExt,
        fs::{DirBuilderExt, PermissionsExt},
    },
    path::{Path, PathBuf},
};

fn group_fixture(name: &str) -> (PathBuf, crate::Directories) {
    let root = std::env::temp_dir().join(format!("temper-{name}-{}", std::process::id()));
    let mut directories = configuration::directories(&root);
    directories.data = root.join("data").as_os_str().as_bytes().into();
    directories.cache = root.join("cache").as_os_str().as_bytes().into();
    directories.config = root.join("config").as_os_str().as_bytes().into();
    fs::DirBuilder::new().recursive(true).mode(0o700).create(root.join("data/box/groups")).expect("private groups");
    (root, directories)
}

fn group_file(directories: &crate::Directories, number: u16, directory: &Path) -> PathBuf {
    let setup = serde_json::json!({
        "directory": directory,
        "bot": {"login":"temper", "email":"temper@example.test"},
        "mode": {"fake": {
            "listen":{"socket":"127.0.0.1:9"}, "git":"demo/bin/git", "owner":"group-owner",
            "repositories":[{"name":"widget", "landing":"main"}]
        }}
    });
    write_group(directories, number, &setup)
}

fn write_group(directories: &crate::Directories, number: u16, setup: &serde_json::Value) -> PathBuf {
    let bytes = temper_forge_setup::group::compose(&serde_json::to_vec(setup).expect("setup JSON"), number, false)
        .expect("composed group");
    let path = crate::path(&directories.data).join(crate::path(&temper_forge_setup::group::path(number)));
    fs::write(&path, bytes).expect("group file");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).expect("private group file");
    path
}

fn checked_document(root: &Path, directories: &crate::Directories) -> temper_forge_setup::check::Checked {
    let value = configuration::check_document(root);
    crate::setup::check(&serde_json::to_vec(&value).expect("document JSON"), &directories.jig())
        .expect("checked document")
        .checked
}

#[test]
fn group_entry_reads_the_group_file_before_the_document() {
    let (root, directories) = group_fixture("group-first");
    let checked = checked_document(&root, &directories);
    group_file(&directories, 1, Path::new("group-forge"));
    let entry = group_entry(&directories, 1, Some(&checked)).expect("read group").expect("entry");
    assert_eq!(crate::path(&entry.directory), root.join("data/group-forge"));
    let temper_forge_setup::setup::Mode::Fake(fake) = entry.mode else { panic!("fake entry") };
    assert_eq!(&*fake.owner, b"group-owner");
    fs::remove_dir_all(root).expect("remove fixture");
}

#[test]
fn group_entry_falls_back_to_the_document_without_a_file() {
    let (root, directories) = group_fixture("group-fallback");
    let checked = checked_document(&root, &directories);
    assert_eq!(group_entry(&directories, 1, Some(&checked)).expect("fallback"), checked.setup.entry(1).cloned());
    assert_eq!(group_entry(&directories, 3, Some(&checked)).expect("no entry"), None);
    assert_eq!(group_entry(&directories, 1, None).expect("no document"), None);
    fs::remove_dir_all(root).expect("remove fixture");
}

#[test]
fn group_entry_refuses_a_group_file_others_can_read() {
    let (root, directories) = group_fixture("group-public");
    let checked = checked_document(&root, &directories);
    let path = group_file(&directories, 1, Path::new("group-forge"));
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).expect("public group file");
    assert_eq!(
        group_entry(&directories, 1, Some(&checked)),
        Err(format!("{}: readable by others or reached through a link", path.display()))
    );
    fs::remove_dir_all(root).expect("remove fixture");
}

#[test]
fn group_entry_refuses_an_invalid_group_file() {
    let (root, directories) = group_fixture("group-invalid");
    let checked = checked_document(&root, &directories);
    let path = group_file(&directories, 1, Path::new("group-forge"));
    fs::write(path, b"{}").expect("invalid group");
    assert_eq!(group_entry(&directories, 1, Some(&checked)), Err("box/groups/1: setup: missing".into()));
    fs::remove_dir_all(root).expect("remove fixture");
}

#[test]
fn remove_takes_the_forge_directory_and_keeps_the_cache() {
    let (root, directories) = group_fixture("remove-forge");
    let forge = root.join("data/forge-3");
    fs::create_dir_all(forge.join("nested/again")).expect("nested forge");
    fs::write(forge.join("nested/again/file"), b"forge data").expect("forge data");
    let binary = root.join("cache/forgejo").join(skein_forge::forgejo::pin::VERSION).join("forgejo");
    fs::create_dir_all(binary.parent().expect("cache parent")).expect("cache directory");
    fs::write(&binary, b"cached binary").expect("cache binary");
    let group = group_file(&directories, 3, Path::new("forge-3"));
    fs::create_dir(root.join("data/box/started")).expect("started directory");
    let started = root.join("data/box/started/3");
    fs::write(&started, b"").expect("started record");
    // Dispatch succeeds with no deployment document, as the box's once step does.
    assert_eq!(start(&directories, &["remove".into(), "3".into()], None).expect("remove"), 0);
    assert!(!forge.exists());
    assert_eq!(fs::read(binary).expect("kept cache"), b"cached binary");
    assert!(group.is_file());
    assert!(started.is_file());
    fs::remove_dir_all(root).expect("remove fixture");
}

#[test]
fn remove_ends_at_once_without_a_file_or_a_directory() {
    let (root, directories) = group_fixture("remove-absent");
    assert_eq!(remove(&directories, 3), Ok(0));
    group_file(&directories, 3, Path::new("forge-3"));
    assert_eq!(remove(&directories, 3), Ok(0));
    assert_eq!(remove(&directories, 3), Ok(0));
    fs::remove_dir_all(root).expect("remove fixture");
}

#[test]
fn remove_ends_at_once_for_a_group_without_a_directory() {
    let (root, directories) = group_fixture("remove-no-directory");
    let group = write_group(
        &directories,
        3,
        &serde_json::json!({
            "bot":{"login":"temper", "email":"temper@example.test"}, "mode":"managed"
        }),
    );
    assert_eq!(remove(&directories, 3), Ok(0));
    assert!(group.is_file());
    fs::remove_dir_all(root).expect("remove fixture");
}

#[test]
fn remove_refuses_a_directory_not_strictly_beneath_data() {
    let (root, mut directories) = group_fixture("remove-boundary");
    directories.cache = root.join("data/cached/binaries").as_os_str().as_bytes().into();
    directories.config = root.join("data/configured/settings").as_os_str().as_bytes().into();
    for directory in [
        root.join("data"),
        root.join("outside"),
        root.join("data-sibling"),
        root.join("data/cached"),
        root.join("data/cached/binaries"),
        root.join("data/cached/binaries/child"),
        root.join("data/configured"),
        root.join("data/configured/settings"),
        root.join("data/configured/settings/child"),
        root.join("data/box"),
        root.join("data/box/child"),
        root.join("data/../outside"),
    ] {
        fs::create_dir_all(&directory).expect("refused directory");
        let kept = directory.join("kept");
        fs::write(&kept, b"keep").expect("kept data");
        group_file(&directories, 3, &directory);
        let why = start(&directories, &["remove".into(), "3".into()], None).expect_err("unsafe removal refused");
        assert!(why.line().starts_with("temper forge remove: forge 3: "));
        assert_eq!(fs::read(kept).expect("nothing removed"), b"keep", "{}", directory.display());
    }
    fs::remove_dir_all(root).expect("remove fixture");
}

#[test]
fn remove_refuses_a_link() {
    let (root, directories) = group_fixture("remove-link");
    let target = root.join("target");
    fs::create_dir(&target).expect("link target");
    fs::write(target.join("kept"), b"keep").expect("target data");
    let link = root.join("data/linked");
    std::os::unix::fs::symlink(&target, &link).expect("directory link");
    for directory in [link.clone(), link.join("child"), link.join(".")] {
        group_file(&directories, 3, &directory);
        assert_eq!(remove(&directories, 3), Err(format!("{}: reached through a symbolic link", link.display())));
        assert!(fs::symlink_metadata(&link).expect("link kept").file_type().is_symlink());
        assert_eq!(fs::read(target.join("kept")).expect("target kept"), b"keep");
    }
    fs::remove_dir_all(root).expect("remove fixture");
}

#[test]
fn old_form_reconcile_adopts_no_repositories() {
    let directory = std::env::temp_dir().join(format!("temper-reconcile-settings-{}", std::process::id()));
    fs::create_dir(&directory).expect("settings scratch");
    let directories = jig_protocol_config::Directories::given(
        directory.as_os_str().as_bytes(),
        directory.as_os_str().as_bytes(),
        directory.as_os_str().as_bytes(),
    )
    .expect("fixture directories");
    for fake in [true, false] {
        let mut value = configuration::old_form_check_document(&directory);
        value["application"]["webhooks"] = serde_json::Value::Null;
        let entry = &mut value["application"]["setup"][0];
        if fake {
            entry["mode"]["fake"]["repositories"] = serde_json::json!([
                {"name":"z-last", "landing":"main"}, {"name":"a-first", "landing":"trunk"},
            ]);
        } else {
            entry["mode"] = serde_json::json!({"managed":{"admin_token":"admin_token"}});
            entry["orgs"] = serde_json::json!(["example"]);
        }
        let deployment = crate::setup::check(&serde_json::to_vec(&value).expect("fixture JSON"), &directories)
            .expect("checked deployment");
        let settings = reconcile_settings(&deployment.named, &deployment.checked, 1).expect("settings");
        assert!(settings.given.adopted.is_empty());
        assert_eq!(settings.given.listener, None, "the box makes no hook");
    }
    fs::remove_dir_all(directory).expect("settings scratch removed");
}

#[test]
fn reconcile_refuses_records_only_owned_and_fake_without_writing_files() {
    let directory = std::env::temp_dir().join(format!("temper-records-reconcile-{}", std::process::id()));
    fs::create_dir(&directory).expect("settings scratch");
    let directories = jig_protocol_config::Directories::given(
        directory.as_os_str().as_bytes(),
        directory.as_os_str().as_bytes(),
        directory.as_os_str().as_bytes(),
    )
    .expect("fixture directories");
    for owned in [false, true] {
        let mut value = configuration::check_document(&directory);
        value["application"]["webhooks"] = serde_json::Value::Null;
        let forge = value["application"]["forges"][0].as_object_mut().expect("forge");
        for name in ["api_token", "user", "webhook_secret", "identity"] {
            forge.remove(name);
        }
        let identity = &mut value["application"]["identities"][0];
        identity.as_object_mut().expect("identity").remove("token");
        identity["forge"] = 1.into();
        if owned {
            value["application"]["setup"][0]["mode"] = serde_json::json!({"owned":{
                "forgejo":"/fixture/forgejo", "runner":null, "listen":{"socket":"127.0.0.1:9"},
                "person":{"login":"ada","email":"ada@example.test"}
            }});
        }
        let deployment = crate::setup::check(&serde_json::to_vec(&value).expect("fixture JSON"), &directories)
            .expect("checked deployment");
        assert_eq!(
            reconcile_settings(&deployment.named, &deployment.checked, 1).err(),
            Some("forge 1 starts from its records: `temper serve` takes its handover".into())
        );
        assert!(!directory.join("forge-1").exists(), "refusal creates no directory or credential file");
    }
    fs::remove_dir_all(directory).expect("settings scratch removed");
}

#[test]
fn reconcile_directs_string_managed_to_the_client_before_files() {
    let directory = std::path::Path::new("/fixture/client-managed");
    let directories = jig_protocol_config::Directories::given(
        b"/fixture/client-managed",
        b"/fixture/client-managed",
        b"/fixture/cache",
    )
    .expect("fixture directories");
    for transport in
        [serde_json::json!("plaintext"), serde_json::json!({"tls":{"server_name":"forge.example","trust":null}})]
    {
        let mut value = configuration::check_document(directory);
        value["application"]["webhooks"] = serde_json::Value::Null;
        value["application"]["forges"][0]["transport"] = transport;
        value["application"]["setup"][0] = serde_json::json!({
            "forge":1,"mode":"managed","bot":{"login":"temper","email":"temper@example.test"}
        });
        let deployment = crate::setup::check(&serde_json::to_vec(&value).expect("fixture JSON"), &directories)
            .expect("checked deployment");
        assert_eq!(
            reconcile_settings(&deployment.named, &deployment.checked, 1).err(),
            Some("forge 1's admin’s token is given from the client: connect it as its admin there".into())
        );
    }
}
