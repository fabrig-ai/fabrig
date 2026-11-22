//! The shared engine/TUI path and private link (client/tui/README.md, 2).
use crate::link_file;

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
