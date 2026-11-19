use super::*;

#[test]
fn patch_replaces_whole_top_level_keys_and_keeps_omitted_values() {
    let shape = Shape::WorkerAlone;
    let file = parse(
        b"engine = '127.0.0.1:7200'\nname = 'kept'\n[environment]\nZ = 'first'\nA = 'second'\n[programs]\ngit = '/tools/git'\nclaude = '/tools/claude'\n",
        shape,
        &facts(),
    ).expect("worker").file;
    let patched = patch(&file, b"[programs]\ngit = '/new/git'\n", shape, &facts()).expect("partial patch").file;
    assert_eq!(patched.engine, file.engine);
    assert_eq!(patched.name, file.name);
    assert_eq!(patched.environment, file.environment);
    assert_eq!(patched.programs.git, Some("/new/git".into()));
    assert_eq!(patched.programs.claude, None);
    let replaced =
        patch(&patched, b"[environment]\nY = 'one'\nB = 'two'\n", shape, &facts()).expect("environment patch").file;
    assert_eq!(replaced.environment, [("Y".into(), "one".into()), ("B".into(), "two".into())]);
    assert_eq!(parse(&render(&replaced, shape), shape, &facts()).expect("rendered patch").file, replaced);
    assert_eq!(patch(&replaced, b"", shape, &facts()).expect("empty patch").file, replaced);
}

#[test]
fn patch_refuses_syntax_other_shapes_and_invalid_result_with_file_words() {
    let shape = Shape::EngineAlone;
    let file = parse(b"workers = '127.0.0.1:7200'\n", shape, &facts()).expect("engine").file;
    for (text, words) in [
        ("not TOML", "expected"),
        ("engine = '127.0.0.1:7200'", "a key of temper-worker's file"),
        ("unknown = 1", "unknown; temper-engine help config lists the keys"),
        ("listen = 7", "a socket address on loopback"),
        ("listen = '127.0.0.1:65535'", "listen uses port 65535; give webhooks explicitly"),
        ("[sign_in.google]\nclient = 'google'\nsecret = 'google.secret'", "required with [sign_in.google]"),
    ] {
        let why = patch(&file, text.as_bytes(), shape, &facts()).expect_err("refused").to_string();
        assert!(why.contains(FILE) && why.contains(words), "{why}");
    }
    assert_eq!(
        patch(&file, b"public = 'https://example.com'", shape, &facts())
            .expect("required workers from base")
            .file
            .workers,
        file.workers
    );
    let invalid = File { owner: Some("not-an-address".into()), ..file };
    let why = patch(&invalid, b"", shape, &facts()).expect_err("inherited value checked too").to_string();
    assert!(why.contains("owner"), "{why}");
}

#[test]
fn substitutions_touch_only_string_values_and_escape_toml_content() {
    let source = b"# {place.path}\nengine = '127.0.0.1:7200'\n[environment]\n'{place.path}' = '{place.path}'\nZ = '''{place.path}'''\n";
    let value = "/scratch/quotes\"/back\\slash\nline";
    let rendered = substitute(source, |s| Ok(s.replace("{place.path}", value))).expect("substitution");
    let text = std::str::from_utf8(&rendered).expect("UTF-8");
    assert!(text.starts_with("# {place.path}\n"));
    assert!(text.contains("'{place.path}' = "));
    let file = parse(&rendered, Shape::WorkerAlone, &facts()).expect("escaped TOML").file;
    assert_eq!(file.environment, [("{place.path}".into(), value.into()), ("Z".into(), value.into())]);
    assert!(substitute(b"broken = [", |s| Ok(s.into())).expect_err("bad TOML").contains(FILE));
}
