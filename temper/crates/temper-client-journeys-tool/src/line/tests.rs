use super::*;

#[test]
fn forge_lines_round_trip_and_are_kept() {
    for source in [r#"forge commit 1 playground/widget "msg" "a.txt" "x\x0a""#, "forge checks 1 playground/widget"] {
        let line = Line::parse(source.as_bytes()).expect("forge line");
        assert_eq!(line.print(), source.as_bytes());
        assert!(line.kept());
    }
}

#[test]
fn forge_fields_enforce_file_and_repository_bounds() {
    let prefix = r#"forge commit 1 playground/widget "msg""#;
    let mut bad = vec![
        (prefix.to_owned(), "1 to 16"),
        (format!("{prefix} \"a.txt\""), "CONTENT"),
        (format!("{prefix} {}", (0..17).map(|n| format!("\"{n}\" \"x\"")).collect::<Vec<_>>().join(" ")), "1 to 16"),
        (format!("{prefix} \"a\" \"x\" \"a\" \"y\""), "distinct"),
        (format!("{prefix} \"{}\" \"x\"", "a".repeat(257)), "256"),
        (format!("{prefix} \"a\" \"{}\"", "x".repeat(65537)), "64 KiB"),
        ("forge checks 1 widget".into(), "OWNER/NAME"),
        ("forge checks 1 a/b/c".into(), "OWNER/NAME"),
        ("forge checks 1 /b".into(), "OWNER/NAME"),
        ("forge checks 1 a/".into(), "OWNER/NAME"),
        ("forge dance".into(), "expected commit or checks"),
    ];
    for path in ["", "/a", "a//b", "a/", ".", "..", "a/./b", "a/../b"] {
        bad.push((format!("{prefix} \"{path}\" \"x\""), "relative PATH"));
    }
    for (source, expected) in bad {
        let why = Line::parse(source.as_bytes()).expect_err("invalid forge fields");
        assert!(why.contains(expected), "{why}; expected {expected}");
    }
    let source = format!("{prefix} \"{}\" \"{}\"", "a".repeat(256), "x".repeat(65536));
    assert!(Line::parse(source.as_bytes()).is_ok());
    let source = format!("{prefix} {}", (0..16).map(|n| format!("\"{n}\" \"\"")).collect::<Vec<_>>().join(" "));
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
