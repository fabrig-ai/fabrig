use super::*;
use crate::credentials::{logout::remove_key_with, tests::Files};

#[test]
fn piped_keys_replace_privately() {
    let path = Path::new("/fixture/installation/keys/work");
    let mut stored = None;
    for (mut input, key) in
        [(b"key-for-work\n".as_slice(), b"key-for-work".as_slice()), (b"key-for-next", b"key-for-next")]
    {
        keep_with(path, &mut input, false, |file, bytes, expected| {
            assert_eq!(file, path);
            assert_eq!(expected, Expect::Any, "login may replace a saved key");
            stored = Some(bytes);
            Ok(())
        })
        .expect("keep piped key");
        assert_eq!(stored.as_deref(), Some(key), "publish stripped bytes, replacing any previous key");
    }
    for present in [true, false] {
        remove_key_with(path, |file| {
            assert_eq!(file, path);
            assert_eq!(stored.is_some(), present);
            if stored.take().is_some() { Ok(()) } else { Err(std::io::ErrorKind::NotFound.into()) }
        })
        .expect("logout succeeds even if the key is already absent");
        assert!(stored.is_none(), "logout unlinks the key");
    }
}

#[test]
fn terminal_input_is_refused() {
    let files = Files::new("smith-credentials");
    assert!(keep(&files.path().join("work"), &mut b"key-for-work".as_slice(), true).is_err());
    assert!(!files.path().join("work").exists());
}

#[test]
fn empty_input_is_refused() {
    let files = Files::new("smith-credentials");
    for input in [b"".as_slice(), b"\n", b"key with spaces", b"key\n\n"] {
        let result = keep(&files.path().join("work"), &mut &*input, false);
        assert!(result.is_err());
    }
    assert!(!files.path().join("work").exists());
}

#[test]
fn login_debug_holds_no_key_text() {
    let environment = crate::credentials::Environment { variables: vec![("WORK_KEY".into(), "key-for-work".into())] };
    let report = format!("{environment:?}");
    assert!(!report.contains("key-for-work"), "environment debug is redacted");
    let words = crate::credentials::Credential::Usable { until: None }.line("work");
    assert!(!words.contains("key-for-work"), "credential words hold no value");
}
