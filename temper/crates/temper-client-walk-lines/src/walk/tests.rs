use super::*;
use skein_scratch::Scratch;

fn roots(scratch: &Scratch) -> Roots {
    Roots { web: scratch.path().join("web"), tui: scratch.path().join("tui") }
}
fn put(root: &Path, name: &str, bytes: &[u8]) -> PathBuf {
    let directory = root.join(name);
    fs::create_dir_all(&directory).expect("journey directory");
    fs::write(directory.join("walk"), bytes).expect("walk");
    directory
}

#[test]
fn lines_keep_their_file_numbers_skipping_empty_lines() {
    let scratch = Scratch::new("walk-lines");
    let roots = roots(&scratch);
    put(&roots.web, "a", b"walk 2\nweb see \"Hi\"\n\nweb mark\n");
    assert_eq!(
        read(&roots, "a").expect("walk"),
        Walk {
            chain: vec![Journey {
                name: "a".into(),
                lines: vec![
                    Numbered { number: 2, line: Line::parse(b"web see \"Hi\"").expect("see") },
                    Numbered { number: 4, line: Line::parse(b"web mark").expect("mark") },
                ],
                scripts: None,
            }],
        }
    );
    put(&roots.web, "b", b"walk 2\nweb press button \"Start\"\nweb reload \"/home\"\n");
    let walk = read(&roots, "b").expect("browser acts and reloads");
    assert_eq!(walk.chain[0].lines[0].line.print(), b"web press button \"Start\"");
    assert_eq!(walk.chain[0].lines[1].line.print(), b"web reload \"/home\"");
}

#[test]
fn session_and_tui_lines_are_refused_by_name_at_their_file_number() {
    let scratch = Scratch::new("walk-forbidden");
    let roots = roots(&scratch);
    for (source, expected) in [
        ("ctl task list", "ctl"),
        ("forge checks forgejo work/hello", "forge"),
        ("deployment restart", "deployment"),
        ("end", "end"),
        ("web look", "web look"),
        ("web look 320", "web look"),
        ("web requests", "web requests"),
        ("tui key enter", "tui"),
        ("tui see \"Ready\"", "tui"),
        ("tui stop", "tui"),
        ("tui look", "tui"),
    ] {
        put(&roots.web, "a", format!("walk 2\n\n{source}\n").as_bytes());
        let error = read(&roots, "a").expect_err("forbidden line");
        assert_eq!((error.journey.as_str(), error.line), ("a", 3));
        assert!(error.why.contains(expected), "{source}: {error:?}");
    }
}

#[test]
fn the_header_is_exactly_walk_two() {
    let scratch = Scratch::new("walk-header");
    let roots = roots(&scratch);
    for header in ["walk 1", "walk 2 ", "walk 2\r", "", "recipe 1"] {
        put(&roots.web, "a", format!("{header}\nweb mark\n").as_bytes());
        assert_eq!(
            read(&roots, "a").expect_err("header"),
            Unreadable { journey: "a".into(), line: 1, why: "expected walk 2".into() }
        );
    }
    put(&roots.web, "a", b"walk 2");
    assert_eq!(read(&roots, "a").expect("header without newline").chain[0].lines, []);
}

#[test]
fn after_joins_both_roots_root_first_and_carries_scripts_unvalidated() {
    let scratch = Scratch::new("walk-after");
    let roots = roots(&scratch);
    let a = put(&roots.web, "a", b"walk 2\nweb see \"Hi\"\n\nweb mark\n");
    fs::write(a.join("scripts.json"), b"not JSON\x00\xff").expect("scripts bytes");
    put(&roots.tui, "b", b"walk 2\nafter a\nweb mark\n");
    put(&roots.web, "c", b"walk 2\nafter b\nweb mark\n");
    let walk = read(&roots, "c").expect("cross-root chain");
    assert_eq!(walk.chain.iter().map(|j| j.name.as_str()).collect::<Vec<_>>(), ["a", "b", "c"]);
    assert_eq!(walk.chain[0].scripts.as_deref(), Some(b"not JSON\x00\xff".as_slice()));
    assert!(walk.chain[1].scripts.is_none());
    assert_eq!(walk.chain[1].lines[0].number, 3);
    assert_eq!(walk.chain[1].lines[0].line.print(), b"web mark");
    assert_eq!(walk.chain[0].lines[1].line.print(), b"web mark", "ancestors retain their marks");
    put(&roots.web, "b", b"walk 2\nweb see \"Web wins\"\n");
    assert_eq!(read(&roots, "b").expect("web first").chain[0].lines[0].line.print(), b"web see \"Web wins\"");
}

#[test]
fn cycle_missing_and_old_parent_name_the_journey_whose_after_names_them() {
    let scratch = Scratch::new("walk-bad-parent");
    let roots = roots(&scratch);
    put(&roots.web, "a", b"walk 2\nafter b\nweb mark\n");
    for (parent, expected) in
        [(None, "missing"), (Some(b"walk 1\n".as_slice()), "walk 2"), (Some(b"walk 2\nafter a\n"), "cycle")]
    {
        if let Some(parent) = parent {
            put(&roots.tui, "b", parent);
        }
        let error = read(&roots, "a").expect_err("bad parent");
        let (child, named) = if expected == "cycle" { ("b", "a") } else { ("a", "b") };
        assert_eq!((error.journey.as_str(), error.line), (child, 2));
        assert!(error.why.contains(expected) && error.why.contains(&format!("after {named:?}")), "{error:?}");
    }
}

#[test]
fn eight_ancestors_are_allowed_but_a_ninth_is_refused_at_its_child() {
    let scratch = Scratch::new("walk-depth");
    let roots = roots(&scratch);
    put(&roots.web, "a0", b"walk 2\nweb mark\n");
    for n in 1..=9 {
        put(&roots.web, &format!("a{n}"), format!("walk 2\nafter a{}\nweb mark\n", n - 1).as_bytes());
    }
    let walk = read(&roots, "a8").expect("eight ancestors");
    assert_eq!(walk.chain.len(), 9);
    assert_eq!(walk.chain[0].name, "a0");
    assert_eq!(walk.chain[8].name, "a8");
    let error = read(&roots, "a9").expect_err("ninth ancestor");
    assert_eq!((error.journey.as_str(), error.line), ("a1", 2));
    assert!(error.why.contains("after \"a0\"") && error.why.contains("ninth"), "{error:?}");
}

#[test]
fn names_and_after_fields_are_checked_before_opening_a_parent() {
    let scratch = Scratch::new("walk-names");
    let roots = roots(&scratch);
    for name in ["", "A", "1a", "a_b", "../a", "a/b"] {
        let error = read(&roots, name).expect_err("invalid name");
        assert_eq!(error.line, 0);
        assert_eq!(error.journey, name);
        assert!(error.why.contains("[a-z][a-z0-9-]*"), "{error:?}");
    }
    for after in ["after \"b\"", "after ../b", "after b extra", "after"] {
        put(&roots.web, "a", format!("walk 2\n{after}\nweb mark\n").as_bytes());
        let error = read(&roots, "a").expect_err("bad after");
        assert_eq!((error.journey.as_str(), error.line), ("a", 2));
    }
    put(&roots.web, "a", b"walk 2\n\nafter b\n");
    assert_eq!(read(&roots, "a").expect_err("after is only second").line, 3);
}

#[test]
fn file_errors_have_line_zero_and_the_file_bound_is_inclusive() {
    let scratch = Scratch::new("walk-file-bound");
    let roots = roots(&scratch);
    assert_eq!(read(&roots, "missing").expect_err("missing walk").line, 0);
    let mut bytes = b"walk 2\n".to_vec();
    bytes.resize(65536, b'\n');
    put(&roots.web, "a", &bytes);
    assert_eq!(read(&roots, "a").expect("64 KiB").chain[0].lines, []);
    bytes.push(b'\n');
    put(&roots.web, "a", &bytes);
    let error = read(&roots, "a").expect_err("too large");
    assert_eq!((error.journey.as_str(), error.line), ("a", 0));
    assert!(error.why.contains("64 KiB"), "{error:?}");
    fs::create_dir_all(roots.web.join("directory/walk")).expect("walk is a directory");
    assert_eq!(read(&roots, "directory").expect_err("unreadable file").line, 0);
}

#[test]
fn moved_checks_only_the_exact_first_line_and_preserves_io_errors() {
    let scratch = Scratch::new("walk-moved");
    assert!(!moved(scratch.path()).expect("no walk"));
    for (bytes, expected) in [
        (b"walk 1\n".as_slice(), false),
        (b"walk 2\ninvalid line\n", true),
        (b"walk 2", true),
        (b"walk 2 \n", false),
        (b"walk 2\r\n", false),
    ] {
        fs::write(scratch.path().join("walk"), bytes).expect("walk");
        assert_eq!(moved(scratch.path()).expect("moved"), expected);
    }
    fs::remove_file(scratch.path().join("walk")).expect("remove walk");
    fs::create_dir(scratch.path().join("walk")).expect("directory not a walk");
    assert!(moved(scratch.path()).is_err());
}
