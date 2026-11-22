use super::{agent_document, prepare, scratch};
use std::{fs, os::unix::fs::PermissionsExt};

#[test]
fn a_secret_file_readable_by_others_refuses_the_start() {
    let directory = scratch("private-secret");
    let secret = directory.join("worker.secret");
    fs::write(&secret, b"private-worker-value").expect("secret file");
    fs::set_permissions(&secret, fs::Permissions::from_mode(0o644)).expect("public permissions");
    let agent = agent_document(&directory, false);
    let bytes = format!(
        r#"{{"workers":[{{
        "engine":{{"socket":"127.0.0.1:7200"}},"name":"worker-a","secret":"{}","slots":1,
        "programs":{{"check_interval_ms":60000}},
        "graces":{{"grace_ms":10000,"agent_stop_ms":5000,"delivery_ms":1000,"save_ms":1000}},
        "agent":{{"program":"jig-test","configuration":"{}","arguments":["agent"]}},
        "limits":{{"profile":"standard"}},"memory_bytes":1152921504606846976
    }}],"application":{{"workers":[{{"root":"agent-root"}}]}}}}"#,
        secret.display(),
        agent.display()
    );
    let mut owned = Vec::new();
    let error = prepare(bytes.as_bytes(), &mut owned).err().expect("start refused");
    assert!(error.contains("Unsafe"), "private read refusal: {error}");
    assert!(!error.contains("private-worker-value"), "diagnostic contains no secret");
    assert!(owned.is_empty(), "refusal happens before opening runtime descriptors");
    fs::remove_dir_all(directory).expect("scratch cleanup");
}

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
