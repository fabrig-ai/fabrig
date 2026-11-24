use super::disk::{Disk, Handle};
use std::path::Path;
use temper_forge_setup_shell::{files, process};

#[test]
fn a_second_lock_is_refused_while_the_first_is_held() {
    let disk = Disk::new(&[]);
    let path = Path::new("/lock");
    let first = disk.lock(path).expect("first lock");
    assert!(matches!(disk.lock(path), Err(process::Refused::Held)));
    drop(first);
    disk.lock(path).expect("released lock");
}

#[test]
fn an_opened_file_lock_follows_its_inode_and_releases_with_its_handle() {
    let mut disk = Disk::new(&[]);
    let (path, renamed) = (Path::new("/lock"), Path::new("/renamed"));
    let file = disk.open_lock(path, 0o600).expect("opened file");
    files::rename_with(&mut disk, path, renamed).expect("renamed before locking");
    let held = process::lock_file_with(file, Handle::lock).expect("opened node locked without a path");
    assert!(matches!(disk.lock(renamed), Err(process::Refused::Held)));
    let replacement = disk.lock(path).expect("replacement path names a different node");
    drop(held);
    let released = disk.lock(renamed).expect("opened node released");
    drop(released);
    drop(replacement);
}
