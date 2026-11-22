use skein_scratch::Scratch as Files;
use smith_local_shell::testing::{keep_key as keep, logout, raw_key as key};
use std::os::unix::fs::PermissionsExt;

#[test]
fn piped_keys_replace_privately() {
    let files = Files::new("smith-credentials");
    let path = files.path().join("keys/work");
    keep(&path, &mut b"key-for-work\n".as_slice(), false).unwrap();
    assert!(key::read(&path).unwrap().as_ref() == b"key-for-work", "trailing newline removed");
    assert_eq!(std::fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
    keep(&path, &mut b"key-for-next".as_slice(), false).unwrap();
    assert!(key::read(&path).unwrap().as_ref() == b"key-for-next", "key replaced");
    logout::remove_key(&path).unwrap();
    assert!(!path.exists());
}
