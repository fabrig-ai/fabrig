use super::groups::{self, Context};
use std::{
    fs,
    os::unix::{ffi::OsStrExt, fs::PermissionsExt},
    path::PathBuf,
};
use temper_box::host::Loaded;
use temper_forge_setup::{check::Checked, setup::group_settings};

const FAKE: &[u8] = br#"{"bot":{"login":"temper","email":"temper@example.invalid"},"mode":{"fake":{"listen":{"socket":"127.0.0.1:8369"},"git":"demo/bin/git","owner":"demo","repositories":[{"name":"widget","landing":"main"}]}}}"#;
const OWNED: &[u8] = br#"{"bot":{"login":"temper","email":"temper@example.invalid"},"mode":{"owned":{"forgejo":"/opt/forgejo/forgejo","runner":null,"listen":{"socket":"127.0.0.1:3000"},"person":{"login":"ada","email":"ada@example.invalid"}}}}"#;

fn context(name: &str) -> (PathBuf, Context) {
    let root = std::env::temp_dir().join(format!("temper-box-groups-{}-{name}", std::process::id()));
    fs::create_dir(&root).expect("scratch");
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).expect("private scratch");
    let path: Box<[u8]> = root.as_os_str().as_bytes().into();
    let directories = crate::Directories {
        config: path.clone(),
        data: path.clone(),
        cache: path,
        origins: [skein_shell::Origin::Flag; 3],
    };
    let context = Context {
        directories,
        limits: Checked {
            fetch_bytes: temper_forge_setup::limits::FETCH_BYTES,
            limits: temper_forge_setup::limits::small(),
        },
        llm: Some("127.0.0.1:8370".parse().expect("LLM address")),
        forgejo_listens: true,
    };
    (root, context)
}
fn write(root: &std::path::Path, name: &str, bytes: &[u8]) {
    let path = root.join(name);
    fs::create_dir_all(path.parent().expect("parent")).expect("directories");
    fs::write(&path, bytes).expect("file");
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).expect("private file");
}
fn settings(root: &std::path::Path, body: &[u8], forge: u16) {
    write(root, &format!("box/groups/{forge}"), &group_settings(body, forge).expect("settings"));
}
fn refused(context: &Context, words: &str) {
    let Loaded::Refused(why) = groups::load(context, 7) else { panic!("refused") };
    assert!(String::from_utf8_lossy(&why).contains(words), "{}", String::from_utf8_lossy(&why));
}

#[test]
fn a_fake_group_file_loads_as_the_playgrounds_steps() {
    let (root, context) = context("fake");
    settings(&root, FAKE, 7);
    let Loaded::Group(group) = groups::load(&context, 7) else { panic!("loaded") };
    assert_eq!(group.forge, 7);
    assert_eq!(&*group.label, b"Playground");
    assert!(group.recorded);
    assert_eq!(
        group.steps.iter().map(|(_, step)| step.name()).collect::<Vec<_>>(),
        [b"fake llm".as_slice(), b"prepare 7", b"fake forge 7"]
    );
    assert_eq!(group.steps[0].1.listeners(), &[context.llm.expect("LLM")]);
    assert_eq!(group.steps[2].1.listeners(), &["127.0.0.1:8369".parse().expect("forge address")]);
    fs::remove_dir_all(root).expect("scratch removed");
}
#[test]
fn an_owned_group_file_loads_with_its_started_notice() {
    let (root, context) = context("owned");
    settings(&root, OWNED, 7);
    let Loaded::Group(group) = groups::load(&context, 7) else { panic!("loaded") };
    assert!(group.recorded);
    assert_eq!(&*group.label, b"Forge 7 (Forgejo)");
    let forgejo = group.steps.iter().find(|(_, step)| step.ready().is_some()).expect("Forgejo step");
    assert_eq!(
        String::from_utf8_lossy(forgejo.1.started_notice()),
        format!(
            "Forge 7 (Forgejo): started\nForge 7 (Forgejo): api http://127.0.0.1:3000/api/v1\nForge 7 (Forgejo): person ada, read header {}/forge-7/person_read.header, write header {}/forge-7/person_write.header\n",
            root.display(),
            root.display()
        )
    );
    fs::remove_dir_all(root).expect("scratch removed");
}
#[test]
fn a_missing_group_file_is_absent() {
    let (root, context) = context("absent");
    assert!(matches!(groups::load(&context, 7), Loaded::Absent));
    fs::remove_dir_all(root).expect("scratch removed");
}
#[test]
fn a_group_file_open_to_others_is_refused() {
    let (root, context) = context("open");
    settings(&root, FAKE, 7);
    fs::set_permissions(root.join("box/groups/7"), fs::Permissions::from_mode(0o644)).expect("open file");
    refused(&context, &format!("{}/box/groups/7: readable by others", root.display()));
    fs::remove_dir_all(root).expect("scratch removed");
}
#[test]
fn a_group_file_for_another_forge_is_refused() {
    let (root, context) = context("other");
    write(&root, "box/groups/7", &group_settings(FAKE, 8).expect("settings"));
    refused(&context, "box/groups/7: forge 8's settings");
    fs::remove_dir_all(root).expect("scratch removed");
}
#[test]
fn a_group_file_with_invalid_settings_names_its_first_problem() {
    let (root, context) = context("invalid");
    write(&root, "box/groups/7", b"{}");
    let why = groups::entry(&context.directories, &context.limits, 7).expect_err("invalid");
    assert!(why.starts_with("box/groups/7: "), "{why}");
    assert_eq!(why.lines().count(), 1);
    fs::remove_dir_all(root).expect("scratch removed");
}
#[test]
fn managed_and_fake_without_a_playground_llm_are_refused() {
    let (root, mut context) = context("mode");
    settings(&root, br#"{"bot":{"login":"temper","email":"temper@example.invalid"},"mode":"managed"}"#, 7);
    refused(&context, "a managed forge runs no group");
    settings(&root, FAKE, 7);
    context.llm = None;
    refused(&context, "no playground LLM");
    fs::remove_dir_all(root).expect("scratch removed");
}
#[test]
fn removal_takes_the_record_the_directory_then_the_settings() {
    let (root, _) = context("remove");
    write(&root, "box/started/7", b"");
    settings(&root, FAKE, 7);
    write(&root, "forge-7/deep/tree/file", b"data");
    write(&root, "cache/forgejo/binary", b"cached");
    assert_eq!(groups::remove(&root, 7), Ok(()));
    for name in ["box/started/7", "forge-7", "box/groups/7"] {
        assert!(!root.join(name).exists());
    }
    assert!(root.join("cache/forgejo/binary").is_file());
    fs::remove_dir_all(root).expect("scratch removed");
}
#[test]
fn a_removal_cut_after_the_directory_ends_on_its_next_ask() {
    let (root, _) = context("cut");
    settings(&root, FAKE, 7);
    assert_eq!(groups::remove(&root, 7), Ok(()));
    assert!(!root.join("box/groups/7").exists());
    fs::remove_dir_all(root).expect("scratch removed");
}
#[test]
fn a_removal_of_nothing_is_done() {
    let (root, _) = context("nothing");
    assert_eq!(groups::remove(&root, 7), Ok(()));
    fs::remove_dir_all(root).expect("scratch removed");
}
#[test]
fn removal_stops_at_its_first_failed_path() {
    let (root, _) = context("remove-failure");
    fs::create_dir_all(root.join("box/started/7")).expect("record is a directory");
    write(&root, "forge-7/file", b"data");
    settings(&root, FAKE, 7);
    let why = groups::remove(&root, 7).expect_err("record removal refused");
    assert!(String::from_utf8_lossy(&why).starts_with(&format!("{}/box/started/7: ", root.display())));
    assert!(root.join("forge-7/file").exists());
    assert!(root.join("box/groups/7").exists());
    fs::remove_dir_all(root).expect("scratch removed");
}
#[test]
fn the_start_loads_recorded_groups_and_drops_records_without_settings() {
    let (root, context) = context("recorded");
    for name in ["3", "7", "9", "not-a-number", "+8", "65536"] {
        write(&root, &format!("box/started/{name}"), b"");
    }
    settings(&root, FAKE, 7);
    let (groups, lines) = groups::recorded(&context, 4).expect("recorded");
    assert_eq!(groups.iter().map(|group| group.forge).collect::<Vec<_>>(), [7]);
    assert!(groups[0].recorded);
    assert_eq!(groups[0].before, 4);
    assert_eq!(
        lines,
        [
            "box: group 3: no settings; its started record is removed",
            "box: group 9: no settings; its started record is removed"
        ]
    );
    assert!(!root.join("box/started/9").exists());
    for name in ["7", "not-a-number", "+8", "65536"] {
        assert!(root.join(format!("box/started/{name}")).exists());
    }
    fs::remove_dir_all(root).expect("scratch removed");
}
#[test]
fn the_start_leaves_a_refused_group_dormant_and_names_it() {
    let (root, context) = context("refused-record");
    write(&root, "box/started/7", b"");
    write(&root, "box/groups/7", b"{}");
    let (groups, lines) = groups::recorded(&context, 0).expect("recorded");
    assert!(groups.is_empty());
    assert_eq!(lines.len(), 1);
    assert!(lines[0].starts_with("box: group 7: box/groups/7: "));
    assert!(root.join("box/started/7").exists());
    fs::remove_dir_all(root).expect("scratch removed");
}
#[test]
fn a_directory_removal_failure_keeps_settings_after_dropping_the_record() {
    let (root, _) = context("directory-failure");
    write(&root, "box/started/7", b"");
    write(&root, "forge-7", b"not a directory");
    settings(&root, FAKE, 7);
    let why = groups::remove(&root, 7).expect_err("directory removal refused");
    assert!(String::from_utf8_lossy(&why).starts_with(&format!("{}/forge-7: ", root.display())));
    assert!(!root.join("box/started/7").exists());
    assert!(root.join("box/groups/7").exists());
    fs::remove_dir_all(root).expect("scratch removed");
}
