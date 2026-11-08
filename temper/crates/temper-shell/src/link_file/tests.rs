//! The shared engine/TUI path and private link (client/README.md, 6.12).
use crate::link_file::{self, MOST};
use std::{
    fs,
    os::unix::fs::{MetadataExt, PermissionsExt},
};

#[test]
fn paths_key_the_deployment_and_refuse_a_missing_absolute_runtime() {
    let mut failures = Vec::new();
    for (data, runtime, linux, expected) in [
        (
            b"/data".as_slice(),
            Some(b"/run/user/test".as_slice()),
            true,
            Some("/run/user/test/temper/printed-link-bd47413b5c03dfc6"),
        ),
        (b"/data", Some(b"relative".as_slice()), true, None),
        (b"/data", None, true, None),
        (b"/data", None, false, Some("/data/printed-link")),
    ] {
        if link_file::path(data, runtime, linux) != expected.map(Into::into) {
            failures.push((runtime, linux));
        }
    }
    assert!(failures.is_empty(), "path rows: {failures:?}");
    assert_ne!(link_file::path(b"/data", Some(b"/runtime"), true), link_file::path(b"/other", Some(b"/runtime"), true));
}

#[test]
fn deployments_resolve_as_the_box_without_flags() {
    let mut failures = Vec::new();
    for (row, entries) in [
        ("absolute TEMPER_DATA", vec![("HOME", "/home/test"), ("TEMPER_DATA", "/deployment")]),
        ("relative TEMPER_DATA", vec![("HOME", "/home/test"), ("TEMPER_DATA", "relative")]),
        ("XDG_DATA_HOME", vec![("HOME", "/home/test"), ("XDG_DATA_HOME", "/xdg")]),
        ("HOME", vec![("HOME", "/home/test")]),
        ("no environment", vec![]),
    ] {
        let environment: Vec<_> =
            entries.iter().map(|(k, v)| (Box::from(k.as_bytes()), Box::from(v.as_bytes()))).collect();
        let expected = crate::directories::resolve(&[], &environment, b"/working").ok().map(|(d, _)| d.data);
        if link_file::deployment(&environment, b"/working") != expected {
            failures.push(row);
        }
    }
    assert!(failures.is_empty(), "deployment rows: {failures:?}");
}

const FIRST: &[u8] =
    b"http://127.0.0.1:7000/jig/v1/sign-in/link?secret=AwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwM".as_slice();

#[test]
fn keeping_replaces_the_whole_private_file_and_forgetting_is_quiet() {
    let scratch = skein_scratch::Scratch::new("link-file-keep");
    let path = scratch.path().join("missing/parent/printed-link");
    let user = skein_shell::effective_user();
    link_file::keep(&path, FIRST, user).expect("keep");
    assert!(fs::read(&path).expect("link") == [FIRST, b"\n"].concat(), "whole URL and LF");
    assert_eq!(fs::metadata(&path).expect("file").mode() & 0o777, 0o600);
    for directory in [path.parent().unwrap(), scratch.path().join("missing").as_path()] {
        assert_eq!(fs::metadata(directory).expect("parent").mode() & 0o777, 0o700);
    }
    link_file::keep(&path, b"short", user).expect("replace");
    assert!(fs::read(&path).expect("replaced") == b"short\n", "no suffix from the previous link");
    assert_eq!(fs::read_dir(path.parent().unwrap()).unwrap().count(), 1, "no temporary left");
    link_file::forget(&path);
    assert!(!path.exists());
    link_file::forget(&path);
}

#[test]
fn keeping_refuses_an_insecure_or_foreign_parent_without_disclosing_the_url() {
    let mut failures = Vec::new();
    for (mode, user) in [(0o770, skein_shell::effective_user()), (0o700, skein_shell::effective_user().wrapping_add(1))]
    {
        let scratch = skein_scratch::Scratch::new("link-file-refusal");
        let parent = scratch.path().join("parent");
        fs::create_dir(&parent).unwrap();
        fs::set_permissions(&parent, fs::Permissions::from_mode(mode)).unwrap();
        let path = parent.join("printed-link");
        let why = match link_file::keep(&path, FIRST, user) {
            Ok(()) => {
                failures.push((mode, user));
                continue;
            }
            Err(why) => why,
        };
        assert!(why.starts_with(&format!("{}: ", path.display())), "error names the target path");
        assert!(!why.as_bytes().windows(FIRST.len()).any(|part| part == FIRST), "error holds no URL");
        assert!(!path.exists(), "refusal writes nothing");
    }
    assert!(failures.is_empty(), "private parent refusals, (mode, user): {failures:?}");
}

#[test]
fn reading_refuses_nonprivate_foreign_and_oversize_files_and_strips_one_lf() {
    let scratch = skein_scratch::Scratch::new("link-file-read");
    let path = scratch.path().join("printed-link");
    let user = skein_shell::effective_user();
    assert!(link_file::read(&path, user).is_none(), "absent");
    let mut failures = Vec::new();
    for (row, bytes, mode, owner, expected) in [
        ("final LF", [FIRST, b"\n"].concat(), 0o600, user, Some(FIRST.to_vec())),
        ("one LF only", b"two\n\n".to_vec(), 0o600, user, Some(b"two\n".to_vec())),
        ("no LF", b"bare".to_vec(), 0o600, user, Some(b"bare".to_vec())),
        ("group-readable", FIRST.to_vec(), 0o640, user, None),
        ("foreign owner", FIRST.to_vec(), 0o600, user.wrapping_add(1), None),
        ("past MOST", vec![b'x'; usize::try_from(MOST).unwrap() + 1], 0o600, user, None),
        (
            "at MOST",
            vec![b'x'; usize::try_from(MOST).unwrap()],
            0o600,
            user,
            Some(vec![b'x'; usize::try_from(MOST).unwrap()]),
        ),
    ] {
        fs::write(&path, bytes).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(mode)).unwrap();
        let found = link_file::read(&path, owner);
        if found.as_ref().map(skein_lib::Secret::expose) != expected.as_deref() {
            failures.push(row);
        }
    }
    assert!(failures.is_empty(), "private bounded read rows: {failures:?}");
}
