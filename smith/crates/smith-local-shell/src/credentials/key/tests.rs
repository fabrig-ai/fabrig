use super::*;
use crate::credentials::tests::Files;
use crate::testing::key_file;

#[test]
fn a_key_file_reads_without_its_trailing_newline() {
    let bytes = value(b"key-for-work\n").expect("private key");
    assert!(bytes.as_ref() == b"key-for-work", "one newline removed");
}

#[test]
fn an_absent_key_is_not_signed_in() {
    let files = Files::new("smith-credentials");
    assert!(matches!(inspect(&files.path().join("absent")), Credential::NotSignedIn), "missing key");
    assert!(matches!(inspect(Path::new("relative")), Credential::NotSignedIn), "no directory");
}

#[test]
fn an_empty_key_is_unreadable() {
    for bytes in [b"".as_slice(), b"\n"] {
        assert!(matches!(finding(value(bytes)), Credential::KeyUnreadable { .. }), "key is empty");
    }
}

#[test]
fn a_key_with_a_space_is_unreadable() {
    for bytes in [b"key for-work".as_slice(), b"key-for-work\n\n", b"key-for-work\r\n", b"key\x7f"] {
        assert!(matches!(finding(value(bytes)), Credential::KeyUnreadable { .. }), "printable one-line key required");
    }
}

#[test]
fn a_key_at_the_token_bound_is_usable() {
    let bound = usize::try_from(crate::credentials::token_limits().token_bytes).expect("token bound");
    let bytes = vec![b'x'; bound];
    assert!(finding(value(&bytes)).usable(), "key at bound");
}

#[test]
fn a_key_past_the_token_bound_is_unreadable() {
    let bound = usize::try_from(crate::credentials::token_limits().token_bytes).expect("token bound");
    let bytes = vec![b'x'; bound + 1];
    assert!(matches!(finding(value(&bytes)), Credential::KeyUnreadable { .. }), "key past bound");
}

#[test]
fn a_usable_key_report_holds_no_value() {
    let credential = finding(read_value(Ok(skein_lib::Secret::new(b"key-for-work".as_slice().into()))));
    assert!(matches!(credential, Credential::Usable { until: None }), "usable key required");
    for text in
        [credential.line("work"), format!("{credential:?}"), serde_json::to_string(&credential).expect("state JSON")]
    {
        assert!(!text.contains("key-for-work"), "report hides the key");
    }
}

#[test]
fn unusable_key_reports_name_the_problem_without_private_data() {
    use skein_shell::Unread;

    let path = Path::new("/fixture/private/key");
    let bound = usize::try_from(crate::credentials::token_limits().token_bytes).expect("token bound");
    let oversized = vec![b'x'; bound + 1];
    for (supplied, expected, words) in [
        (Err(Unread::Unsafe(skein_io::file::Unsafe::Mode(0o044))), KeyProblem::Mode, "mode"),
        (Ok(b"".as_slice()), KeyProblem::Size, "size"),
        (Ok(oversized.as_slice()), KeyProblem::Size, "size"),
        (Ok(b"key for-work".as_slice()), KeyProblem::Unreadable, "unreadable"),
    ] {
        let credential = finding(read_value(supplied.map(|bytes| skein_lib::Secret::new(bytes.into()))));
        assert!(
            matches!(credential, Credential::KeyUnreadable { problem } if problem == expected),
            "key failure class"
        );
        let line = credential.line("work");
        assert!(line.contains("work") && line.contains(words), "account and problem named");
        for text in [line, format!("{credential:?}"), serde_json::to_string(&credential).expect("state JSON")] {
            assert!(!text.contains("key-for-work") && !text.contains("key for-work"), "no key bytes");
            assert!(!text.contains(path.to_str().expect("scratch UTF-8")), "no private path");
        }
    }
}

#[test]
fn key_descriptors_hide_their_paths_in_debug() {
    let files = Files::new("smith-credentials");
    let path = key_file(files.path(), b"key-for-work", 0o600);
    let directory = crate::credentials::Directory::from_base(files.path().to_path_buf()).expect("absolute base");
    let source = crate::credentials::Source::Key { file: path };
    for text in [format!("{directory:?}"), format!("{source:?}")] {
        assert!(!text.contains(files.path().to_str().expect("scratch UTF-8")), "no private path");
        assert!(!text.contains("key-for-work"), "no key bytes");
    }
}
