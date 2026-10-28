use super::make_missing;
use std::{
    fs,
    os::unix::fs::{MetadataExt, PermissionsExt},
};

#[test]
fn credentials_are_private_whole_and_left_in_place_on_a_second_run() {
    let directory = std::env::temp_dir().join(format!("temper-forge-sign-in-{}", std::process::id()));
    fs::create_dir(&directory).expect("fresh private scratch");
    fs::set_permissions(&directory, fs::Permissions::from_mode(0o700)).expect("private scratch");
    for (name, key) in [("key.pem", true), ("client.secret", false)] {
        let path = directory.join(name);
        make_missing(&path, key).expect("make credential");
        let first = fs::metadata(&path).expect("credential metadata");
        assert_eq!(first.permissions().mode() & 0o777, 0o600);
        assert_eq!(first.uid(), skein_shell::effective_user());
        if key {
            assert!(first.len() > 2000);
        } else {
            assert_eq!(first.len(), 43);
        }
        make_missing(&path, key).expect("leave existing credential alone");
        let second = fs::metadata(&path).expect("credential metadata");
        assert_eq!(first.ino(), second.ino());
        assert_eq!(first.mtime_nsec(), second.mtime_nsec());
    }
    assert_eq!(fs::read_dir(&directory).expect("scratch entries").count(), 2, "no partial files");
    fs::remove_dir_all(directory).expect("scratch removed");
}
