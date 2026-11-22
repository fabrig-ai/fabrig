#![cfg(unix)]

#[cfg(unix)]
fn executable(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    // An invalid executable is enough: finding must never run it.
    std::fs::write(path, b"not an executable format\n").expect("rg stand-in");
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).expect("executable mode");
}

#[cfg(unix)]
#[test]
fn rg_finding_takes_the_first_executable_file_on_path() {
    use std::os::unix::{ffi::OsStrExt, fs::PermissionsExt};
    let scratch = skein_scratch::Scratch::new("smith-rg-path");
    for directory in ["A", "B", "C"] {
        std::fs::create_dir(scratch.path().join(directory)).expect("PATH directory");
        executable(&scratch.path().join(directory).join("rg"));
    }
    std::fs::set_permissions(scratch.path().join("A/rg"), std::fs::Permissions::from_mode(0o600))
        .expect("non-executable mode");
    let path = std::env::join_paths([
        std::path::PathBuf::from("rel"),
        scratch.path().join("missing"),
        scratch.path().join("A"),
        scratch.path().join("B"),
        scratch.path().join("C"),
    ])
    .expect("PATH");
    assert_eq!(
        crate::config::find_rg(Some(path.as_bytes())),
        Some(scratch.path().join("B/rg")),
        "first executable file"
    );
}
