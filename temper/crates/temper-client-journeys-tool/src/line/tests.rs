use super::*;

#[test]
fn forge_lines_round_trip_and_are_kept() {
    for source in [
        r#"forge commit forgejo playground/widget "msg" "CONTRIBUTING.md" ".fake-forge/ci""#,
        "forge checks forgejo playground/widget",
    ] {
        let line = Line::parse(source.as_bytes()).expect("forge line");
        assert_eq!(line.print(), source.as_bytes());
        assert!(line.kept());
    }
}

#[test]
fn forge_fields_enforce_file_and_repository_bounds() {
    let prefix = r#"forge commit forgejo playground/widget "msg""#;
    let mut bad = vec![
        (prefix.to_owned(), "1 to 16"),
        (format!("{prefix} {}", (0..17).map(|n| format!("\"{n}\"")).collect::<Vec<_>>().join(" ")), "1 to 16"),
        (format!("{prefix} \"a\" \"a\""), "distinct"),
        (format!("{prefix} \"{}\"", "a".repeat(257)), "256"),
        ("forge checks forgejo widget".into(), "OWNER/NAME"),
        ("forge checks forgejo a/b/c".into(), "OWNER/NAME"),
        ("forge checks forgejo /b".into(), "OWNER/NAME"),
        ("forge checks forgejo a/".into(), "OWNER/NAME"),
        ("forge commit 1 playground/widget \"msg\" \"a\"".into(), "forgejo"),
        ("forge commit github playground/widget \"msg\" \"a\"".into(), "forgejo"),
        ("forge dance".into(), "expected commit or checks"),
    ];
    for path in ["", "/a", "a//b", "a/", ".", "..", "a/./b", "a/../b"] {
        bad.push((format!("{prefix} \"{path}\""), "relative PATH"));
    }
    for (source, expected) in bad {
        let why = Line::parse(source.as_bytes()).expect_err("invalid forge fields");
        assert!(why.contains(expected), "{why}; expected {expected}");
    }
    let source = format!("{prefix} \"{}\"", "a".repeat(256));
    assert!(Line::parse(source.as_bytes()).is_ok());
    let source = format!("{prefix} {}", (0..16).map(|n| format!("\"{n}\"")).collect::<Vec<_>>().join(" "));
    assert!(Line::parse(source.as_bytes()).is_ok());
}

#[test]
fn forms_round_trip_and_bad_fields_are_named() {
    for source in [
        "",
        "end",
        r#"web press button "Set up temper""#,
        r#"web press button "Start" 0"#,
        r#"web submit textbox "Ask" 2"#,
        r#"web toggle checkbox "Strict""#,
        r#"web input textbox "Name" "Hello\\\"\x0a\xc3\xa9" 3"#,
        r#"web see "Hello""#,
        r#"web reload "/projects/1?tab=home""#,
        "web look",
        "web look 320",
        "web look 3840",
        "tui look",
        "tui stop",
        r#"tui see "Inbox""#,
        "tui key enter",
        r#"tui key char "\xc3\xa9""#,
        r#"tui key control "c""#,
        r#"tui key alt "x""#,
        r#"tui key paste "Hello\x0a""#,
        "ctl task show 3 --until ended",
        r#"ctl chat start "Hello there" --key "one""#,
    ] {
        let line = Line::parse(source.as_bytes()).unwrap_or_else(|why| panic!("{source}: {why}"));
        assert_eq!(line.print(), source.as_bytes());
        assert_eq!(Line::parse(line.print()).expect("print"), line);
    }
    for key in [
        "tab",
        "backtab",
        "backspace",
        "delete",
        "up",
        "down",
        "left",
        "right",
        "home",
        "end",
        "pageup",
        "pagedown",
        "escape",
    ] {
        let source = format!("tui key {key}");
        assert_eq!(Line::parse(source.as_bytes()).expect("key").print(), source.as_bytes());
    }
    for (source, expected) in [
        ("other look", "unknown client"),
        ("web dance", "unknown web record"),
        ("tui dance", "unknown tui record"),
        ("web", "web record"),
        ("web press", "role"),
        ("web press unknown", "unknown role"),
        ("web press button", "name"),
        (r#"web input textbox "Name""#, "value"),
        ("web see", "text"),
        ("web reload", "path"),
        ("ctl", "argument"),
        (r#"web press button "A" 1 2"#, "extra field"),
        ("web look 319", "320 to 3840"),
        ("web look 3841", "320 to 3840"),
        ("web look 500 600", "extra field"),
        ("tui stop extra", "extra field"),
        ("end extra", "extra field"),
        ("tui key", "key"),
        ("tui key unknown", "key"),
        ("tui key char", "key"),
        ("tui key enter extra", "extra field"),
        (r#"web see "a" "b""#, "extra field"),
        (r#"web reload "//elsewhere/path""#, "box's origin"),
        (r#"web see "\q""#, "bad escape"),
        ("web  look", "invalid word"),
    ] {
        let why = Line::parse(source.as_bytes()).expect_err("bad line");
        assert!(why.contains(expected), "{source}: {why}; expected {expected}");
    }
}

#[test]
fn ctl_box_values_are_bare_or_quoted_and_unknown_names_are_refused() {
    for source in [
        "ctl forge connect forgejo-admin {forgejo.admin} --api {forgejo.api} --git {forgejo.git} --git-program {git}",
        r#"ctl forge connect forgejo-admin "{forgejo.admin}" --api "{forgejo.api}" --git "{forgejo.git}" --git-program "{git}""#,
    ] {
        let line = Line::parse(source.as_bytes()).expect("box values");
        assert_eq!(line.print(), source.as_bytes());
        assert!(line.kept());
    }
    for source in ["ctl {forgejo.nope}", r#"ctl "{forgejo.nope}""#] {
        let why = Line::parse(source.as_bytes()).expect_err("unknown box value");
        assert!(why.contains("{forgejo.nope}"), "{why}");
    }
}
