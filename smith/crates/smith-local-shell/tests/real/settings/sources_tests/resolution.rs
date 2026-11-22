use skein_scratch::Scratch as Files;
use smith_local_shell::testing::resolve;
use std::os::unix::fs::PermissionsExt;

#[test]
fn a_missing_or_non_executable_program_is_refused() {
    let files = Files::new("smith-credentials");
    assert!(resolve::programs::locate(None, Some(files.path().as_os_str())).is_err(), "absent executable");
    std::fs::write(files.path().join("claude"), b"script").expect("scratch non-executable");
    std::fs::set_permissions(files.path().join("claude"), std::fs::Permissions::from_mode(0o600)).expect("file mode");
    assert!(resolve::programs::locate(None, Some(files.path().as_os_str())).is_err(), "non-executable");
    assert!(resolve::programs::locate(None, None).is_err(), "absent PATH");
}
