use std::{fs, os::unix::fs::PermissionsExt};

#[test]
fn provider_key_files_refuse_group_permissions_and_symbolic_links() {
    use std::os::unix::{ffi::OsStrExt, fs::symlink};
    let directory = tempfile::tempdir().expect("private key scratch");
    let canonical = fs::canonicalize(directory.path()).expect("canonical private key scratch");
    let key = canonical.join("provider.pem");
    fs::write(&key, b"key contents are never included in a diagnostic").expect("key file");
    let user = skein_shell::effective_user();
    fs::set_permissions(&key, fs::Permissions::from_mode(0o640)).expect("group bits");
    let error = super::read_startup_file(key.as_os_str().as_bytes(), 8192, user).expect_err("group bits refused");
    assert!(error.contains("Unsafe(Mode("), "{error}");
    assert!(!error.contains("key contents"), "{error}");
    fs::set_permissions(&key, fs::Permissions::from_mode(0o600)).expect("private key");
    let link = canonical.join("linked.pem");
    symlink(&key, &link).expect("key link");
    let error = super::read_startup_file(link.as_os_str().as_bytes(), 8192, user).expect_err("link refused");
    assert!(error.contains("Unsafe(Link)"), "{error}");
    assert!(!error.contains("key contents"), "{error}");

    let real = canonical.join("real");
    fs::create_dir(&real).expect("key directory");
    let key = real.join("provider.pem");
    fs::write(&key, b"key contents are never included in a diagnostic").expect("key in directory");
    fs::set_permissions(&key, fs::Permissions::from_mode(0o600)).expect("private key in directory");
    let linked = canonical.join("linked");
    symlink(&real, &linked).expect("directory link");
    let key = linked.join("provider.pem");
    let error = super::read_startup_file(key.as_os_str().as_bytes(), 8192, user).expect_err("directory link refused");
    assert!(error.contains("Unsafe(Link)"), "{error}");
    assert!(!error.contains("key contents"), "{error}");
}
