use super::{
    disk::{Action, Call, Disk},
    fixtures,
};
use skein_fake_machine::How;
use std::{io::ErrorKind, path::Path};
use temper_forge_setup_shell::files;

fn call(action: Action, path: &Path) -> Call {
    Call { action, path: path.into(), to: None, result: Ok(()) }
}

#[test]
fn a_file_is_written_whole_private_and_renamed_over() {
    let mut disk = Disk::new(&[]);
    let (path, partial, parent) = (Path::new("/value"), Path::new("/value.partial"), Path::new("/"));
    files::write_private_with(&mut disk, path, b"before").expect("first write");
    let first = disk.open_at(path, How::Read).expect("first node");
    disk.calls.clear();
    files::write_private_with(&mut disk, path, b"after").expect("replacement");
    assert_eq!(disk.read(path), b"after");
    assert_eq!(first.bytes(16), b"before");
    assert_eq!(first.stat().links, 0, "the original node was replaced");
    assert_eq!(disk.entries(parent), 1);
    assert_eq!(
        disk.calls,
        [
            call(Action::CreateNew(0o600), partial),
            call(Action::Write, partial),
            call(Action::Sync, partial),
            Call { to: Some(path.into()), ..call(Action::Rename, partial) },
            call(Action::Open, parent),
            call(Action::Sync, parent),
        ]
    );
}

#[test]
fn a_sync_failure_is_reported_on_either_side_of_the_rename() {
    let (path, partial, parent) = (Path::new("/value"), Path::new("/value.partial"), Path::new("/"));
    for (fault, current, links) in [(partial, &b"before"[..], 1), (parent, &b"after"[..], 0)] {
        let mut disk = Disk::new(&[]);
        files::write_private_with(&mut disk, path, b"before").expect("first write");
        let first = disk.open_at(path, How::Read).expect("first node");
        disk.fail = Some((Action::Sync, fault.into()));
        let error = files::write_private_with(&mut disk, path, b"after").expect_err("sync refused");
        assert_eq!(error.kind(), ErrorKind::PermissionDenied);
        assert_eq!(
            disk.calls.last(),
            Some(&Call { result: Err(ErrorKind::PermissionDenied), ..call(Action::Sync, fault) })
        );
        assert_eq!(disk.read(path), current, "sync failed at {}", fault.display());
        assert_eq!(first.bytes(16), b"before");
        assert_eq!(first.stat().links, links);
    }
}

#[test]
fn clear_removes_a_first_preparations_files_and_never_backups() {
    use skein_fake_machine::Item;
    let place = fixtures::place();
    let root = Path::new(fixtures::DIRECTORY);
    let mut disk = Disk::new(&[Item::directory(b"fixture"), Item::directory(b"fixture/forge-1")]);
    assert!(!files::present_with(&mut disk, &place).expect("empty forge"));
    disk.calls.clear();
    files::layout_with(&mut disk, &place).expect("layout");
    let layout = ["home", "forgejo", "forgejo/repositories", "runner", "backups"];
    let requested: Vec<_> = layout
        .iter()
        .flat_map(|name| {
            let path = root.join(name);
            [call(Action::CreateDirectory, &path), call(Action::SetMode(0o700), &path)]
        })
        .collect();
    assert_eq!(disk.calls[..requested.len()], requested);
    assert_eq!(disk.read(&root.join("home/gitconfig")), b"");
    for name in [
        "state",
        "backups/dump.zip",
        "forgejo/leftover",
        "runner/secret",
        "admin_token",
        "person_password",
        "person_read.header",
        "person_write.header",
    ] {
        files::write_private_with(&mut disk, &root.join(name), b"kept").expect("file");
    }
    assert!(files::present_with(&mut disk, &place).expect("first preparation left files"));
    disk.calls.clear();
    files::clear_with(&mut disk, &place).expect("cleared");
    let removed = [
        ("forgejo", Action::RemoveTree),
        ("runner", Action::RemoveTree),
        ("admin_token", Action::RemoveFile),
        ("person_password", Action::RemoveFile),
        ("person_read.header", Action::RemoveFile),
        ("person_write.header", Action::RemoveFile),
    ];
    let requested: Vec<_> = removed
        .iter()
        .flat_map(|(name, action)| {
            let path = root.join(name);
            [call(Action::NoFollow, &path), call(*action, &path)]
        })
        .collect();
    assert_eq!(disk.calls, requested);
    for (name, _) in removed {
        assert!(!files::exists_with(&mut disk, &root.join(name)).expect("inspection"), "{name} removed");
    }
    for name in ["state", "backups/dump.zip"] {
        assert_eq!(disk.read(&root.join(name)), b"kept", "{name}");
    }
    assert!(!files::present_with(&mut disk, &place).expect("only retained files"));
}

#[test]
fn a_renamed_file_exists_under_its_new_name_only() {
    let mut disk = Disk::new(&[]);
    let (from, to, parent) = (Path::new("/dump.partial.zip"), Path::new("/backup.zip"), Path::new("/"));
    files::write_private_with(&mut disk, from, b"PKcontents").expect("partial");
    assert!(!files::exists_with(&mut disk, to).expect("absent"));
    disk.calls.clear();
    files::rename_with(&mut disk, from, to).expect("rename");
    assert_eq!(
        disk.calls,
        [
            call(Action::Open, from),
            call(Action::Sync, from),
            Call { to: Some(to.into()), ..call(Action::Rename, from) },
            call(Action::Open, parent),
            call(Action::Sync, parent),
        ]
    );
    assert!(!files::exists_with(&mut disk, from).expect("old name"));
    assert!(files::exists_with(&mut disk, to).expect("backup"));
    assert_eq!(disk.read(to), b"PKcontents");
}
