#[test]
fn a_push_key_refuses_nonprivate_files_naming_its_member() {
    use std::os::unix::{ffi::OsStrExt, fs::PermissionsExt};
    let scratch = skein_scratch::Scratch::new("push-private-file");
    let path = scratch.path().join("push.pem");
    let name = || jig_protocol_config::File {
        at: jig_protocol_config::Path::root().member(b"application").member(b"push").member(b"key"),
        path: path.as_os_str().as_bytes().into(),
        max: 1024,
        private: true,
    };
    std::fs::write(&path, b"fixture").expect("key fixture file");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).expect("nonprivate fixture");
    let why = super::read_files(Box::new([name()]), skein_shell::effective_user()).expect_err("nonprivate key refused");
    assert!(why.starts_with("application.push.key:"));
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).expect("private fixture");
    assert!(super::read_files(Box::new([name()]), skein_shell::effective_user()).is_ok());
}
