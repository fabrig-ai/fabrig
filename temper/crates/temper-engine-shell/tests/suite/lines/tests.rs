use temper_engine_shell::lines::read_handover;

#[test]
fn the_playground_start_makes_a_private_random_key_before_reading_it() {
    use std::os::unix::{ffi::OsStrExt as _, fs::MetadataExt as _};
    let scratch = skein_scratch::Scratch::new("playground-key");
    let path = scratch.path().join("demo/llm.key");
    let read = temper_engine_service::HandoverRead {
        forge: 0,
        run: skein_lib::Token::new(1),
        path: path.as_os_str().as_bytes().into(),
        max: 4097,
    };
    let first = read_handover(&read).expect("make then read key").expect("key present");
    assert_eq!(first.expose().len(), 64);
    assert!(first.expose().iter().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(b)));
    let file = std::fs::read(&path).expect("generated fixture");
    assert_eq!(file.len(), 65);
    assert_eq!(file.last(), Some(&b'\n'));
    let same_key = file.strip_suffix(b"\n") == Some(first.expose());
    assert!(same_key, "the account gets the generated key without its final newline");
    assert_eq!(std::fs::metadata(path.parent().expect("demo")).expect("directory").mode() & 0o777, 0o700);
    assert_eq!(std::fs::metadata(&path).expect("key").mode() & 0o777, 0o600);
    // Replacing a linked file must neither reuse its mode nor change its target.
    std::fs::remove_file(&path).expect("remove fixture");
    let target = scratch.path().join("kept");
    std::fs::write(&target, b"kept").expect("fixture");
    std::os::unix::fs::symlink(&target, &path).expect("linked fixture");
    let next = read_handover(&read).expect("replace then read key").expect("key present");
    assert!(next.expose() != first.expose(), "a fresh kernel seed, not a reused key");
    assert_eq!(std::fs::read(target).expect("target unchanged"), b"kept");
    assert!(!std::fs::symlink_metadata(&path).expect("key").is_symlink());
    assert_eq!(std::fs::metadata(&path).expect("key").mode() & 0o777, 0o600);
}
