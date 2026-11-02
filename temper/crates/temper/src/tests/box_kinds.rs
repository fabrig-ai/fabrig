//! Slot removal's data boundary (temper/docs/design/protocol/setup.md, 7.1).
use crate::forge::remove_directory;
use std::fs;

#[test]
fn forge_remove_refuses_a_path_outside_data() {
    let scratch = skein_scratch::Scratch::new("forge-remove-boundary");
    let data = scratch.path().join("data");
    let cache = scratch.path().join("cache");
    fs::create_dir(&data).expect("data");
    fs::create_dir(&cache).expect("cache");
    fs::write(cache.join("kept"), b"binary").expect("cache binary");
    assert!(remove_directory(&data, &cache, &cache).is_err());
    assert!(remove_directory(&data, &scratch.path().join("absent"), &cache).is_err());
    let link = data.join("forge-7");
    std::os::unix::fs::symlink(&cache, &link).expect("external symlink");
    assert!(remove_directory(&data, &link, &cache).is_err());
    assert!(remove_directory(&data, &data, &cache).is_err());
    assert_eq!(fs::read(cache.join("kept")).expect("kept binary"), b"binary");
}

#[test]
fn forge_remove_of_an_absent_directory_succeeds() {
    let scratch = skein_scratch::Scratch::new("forge-remove-absent");
    let directory = scratch.path().join("forge-7");
    remove_directory(scratch.path(), &directory, &scratch.path().join("cache")).expect("absent succeeds");
    fs::create_dir(&directory).expect("forge directory");
    fs::write(directory.join("state"), b"state").expect("forge state");
    remove_directory(scratch.path(), &directory, &scratch.path().join("cache")).expect("remove forge");
    remove_directory(scratch.path(), &directory, &scratch.path().join("cache")).expect("repeat succeeds");
    assert!(!directory.exists());
}

#[test]
fn forge_commands_compose_the_recorded_kind_with_the_machine() {
    use temper_forge_setup::setup::Mode;
    let scratch = skein_scratch::Scratch::new("forge-kind-entry");
    let directories = super::configuration::directories(scratch.path());
    let bytes = serde_json::to_vec(&serde_json::json!({"application":{"machine":{"owned":{
        "listen":{"socket":"127.0.0.1:8372"},"remote":"http://127.0.0.1:8372",
        "person":{"login":"ada","email":"ada@example.invalid"},"forgejo":"fetched","runner":null
    }, "fake":{"listen":{"socket":"127.0.0.1:8369"},"remote":"http://127.0.0.1:8369","git":"demo/bin/git"}}}}))
    .expect("machine JSON");
    let groups = scratch.path().join("box/groups");
    fs::create_dir_all(&groups).expect("groups");
    for (kind, directory) in [("owned", "forge-17"), ("fake", "playground-17")] {
        fs::write(groups.join("17"), format!("{kind}\n")).expect("group kind");
        let entry = crate::forge::entry(&directories, &bytes, 17).expect("composed entry");
        assert_eq!(entry.forge, 17);
        assert_eq!(entry.directory.as_ref(), scratch.path().join(directory).as_os_str().as_encoded_bytes());
        match entry.mode {
            Mode::Owned(owned) => {
                assert_eq!(kind, "owned");
                assert_eq!(owned.listen.port(), 8372);
                assert!(owned.fetch.forgejo);
            }
            Mode::Fake(fake) => {
                assert_eq!(kind, "fake");
                assert_eq!(fake.listen.port(), 8369);
            }
            Mode::Managed { .. } => panic!("a kind slot is owned or fake"),
        }
    }
    fs::write(groups.join("17"), b"unknown\n").expect("invalid group");
    assert!(crate::forge::entry(&directories, &bytes, 17).is_err());
}
