use skein_scratch::Scratch as Files;
use smith_local_shell::{credentials::Credential, testing::inspect_key as inspect};

fn file(files: &Files, bytes: &[u8], mode: u32) -> std::path::PathBuf {
    smith_local_shell::testing::key_file(files.path(), bytes, mode)
}

#[test]
fn a_key_open_to_others_is_unreadable() {
    let files = Files::new("smith-credentials");
    assert!(
        matches!(inspect(&file(&files, b"key-for-work", 0o644)), Credential::KeyUnreadable { .. }),
        "private mode required"
    );
}
