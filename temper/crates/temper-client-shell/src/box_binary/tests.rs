use super::*;
use std::os::unix::ffi::OsStringExt;

fn words(values: &[&str]) -> Box<[Box<[u8]>]> {
    values.iter().map(|value| value.as_bytes().into()).collect()
}
fn environment(values: &[(&str, &str)]) -> Box<[EnvironmentEntry]> {
    values.iter().map(|(key, value)| (key.as_bytes().into(), value.as_bytes().into())).collect()
}
fn captured(environment: Box<[EnvironmentEntry]>) -> Capture {
    Capture {
        run_as: b"/bin/temper".as_slice().into(),
        arguments: Box::new([]),
        environment,
        working: Ok("/working".into()),
        executable: Ok("/own/bin/temper".into()),
    }
}

#[test]
fn box_directories_resolve_flags_then_twins_then_xdg_then_home() {
    for (arguments, variables, cache) in [
        (
            vec!["serve", "--cache", "given", "--other", "untouched"],
            vec![("HOME", "/home/ada"), ("TEMPER_CACHE", "/twin"), ("XDG_CACHE_HOME", "/xdg")],
            "/working/given",
        ),
        (vec!["serve"], vec![("HOME", "/home/ada"), ("TEMPER_CACHE", "/twin"), ("XDG_CACHE_HOME", "/xdg")], "/twin"),
        (vec!["serve"], vec![("HOME", "/home/ada"), ("TEMPER_CACHE", ""), ("XDG_CACHE_HOME", "/xdg")], "/xdg/temper"),
        (vec!["serve"], vec![("HOME", "/home/ada")], "/home/ada/.cache/temper"),
        (vec!["--", "--cache", "/not-a-flag"], vec![("HOME", "/home/ada")], "/home/ada/.cache/temper"),
    ] {
        let arguments = words(&arguments);
        let resolved = resolve(&arguments, &environment(&variables), b"/working").expect("directories");
        assert_eq!(resolved.directories.cache.as_ref(), cache.as_bytes());
        assert_eq!(resolved.directories.data.as_ref(), b"/home/ada/.local/share/temper");
        assert!(!resolved.echo_steps);
    }
    let resolved = resolve(
        &words(&["--data", "box-data", "--config", "/config", "--echo", "steps"]),
        &environment(&[("HOME", "/home/ada")]),
        b"/working",
    )
    .expect("box paths");
    assert_eq!(resolved.directories.data.as_ref(), b"/working/box-data");
    assert_eq!(resolved.directories.config.as_ref(), b"/config");
    assert!(resolved.echo_steps);
}

#[test]
fn the_site_is_https_loopback_with_an_explicit_port_or_the_default() {
    for (value, host, port) in [
        (None, "fabrig.ai", 443),
        (Some(""), "fabrig.ai", 443),
        (Some("https://127.0.0.1:8443"), "127.0.0.1", 8443),
        (Some("https://[::1]:8443"), "[::1]", 8443),
        (Some("https://localhost:8443"), "localhost", 8443),
    ] {
        let variables = value.map_or_else(Vec::new, |value| vec![("TEMPER_INTERNAL_SITE", value)]);
        assert_eq!(site(&environment(&variables)).expect("site"), Origin { host: host.as_bytes().into(), port });
    }
    for value in [
        "https://fabrig.ai:443",
        "http://127.0.0.1:8443",
        "https://127.0.0.1:8443/x",
        "https://localhost",
        "https://localhost:0",
        "https://localhost:65536",
        "https://localhost:+443",
        "https://localhost:443/",
        "https://localhost:443?x",
        "https://localhost:443#x",
        "https://user@localhost:443",
    ] {
        let (code, line) = site(&environment(&[("TEMPER_INTERNAL_SITE", value)])).expect_err("refused site");
        assert_eq!(code, 1, "{value}");
        assert!(line.starts_with("temper: TEMPER_INTERNAL_SITE:"), "{line}");
    }
}

#[test]
fn find_uses_the_captured_executable_identity_site_and_trust() {
    let captured =
        captured(environment(&[("TEMPER_INTERNAL_SITE", "https://localhost:8443"), ("SSL_CERT_FILE", "/cert.pem")]));
    let inputs = Inputs::new("/cache".into(), &captured).expect("inputs");
    let find = inputs.find();
    assert_eq!(find.beside, Path::new("/own/bin"));
    assert_eq!(find.cache, Path::new("/cache"));
    assert_eq!(find.site, &Origin { host: b"localhost".as_slice().into(), port: 8443 });
    match find.trust {
        skein_shell::Trust::Machine { ssl_cert_file } => {
            assert_eq!(ssl_cert_file.as_deref(), Some(Path::new("/cert.pem")));
        }
        skein_shell::Trust::Der(_) => panic!("the captured SSL_CERT_FILE is a PEM bundle"),
    }
    assert_eq!((find.version, find.commit), (temper_version::VERSION, temper_version::COMMIT));
}

#[test]
fn fetch_box_passes_progress_then_found_and_prefixes_a_refusal() {
    let mut stdout = Vec::new();
    stdout.extend_from_slice(b"fetched temper-box 0 of 3 bytes\nfetched temper-box 3 of 3 bytes\n");
    let path = PathBuf::from(std::ffi::OsString::from_vec(b"/cache/box-\xff".to_vec()));
    assert_eq!(found(Ok(path), &mut stdout), Ok(0));
    assert_eq!(stdout, b"fetched temper-box 0 of 3 bytes\nfetched temper-box 3 of 3 bytes\nfound /cache/box-\xff\n");
    stdout.clear();
    assert_eq!(
        found(Err("no temper-box of this build\nFetch it by hand: example".into()), &mut stdout),
        Err((1, "temper: no temper-box of this build\nFetch it by hand: example\n".into()))
    );
    assert!(stdout.is_empty());
    let (code, line) = found(Err("é".repeat(8192)), &mut stdout).expect_err("bounded refusal");
    assert_eq!(code, 1);
    assert!(line.len() <= 8192 && line.starts_with("temper: ") && line.ends_with('\n'));
    assert!(stdout.is_empty());
    // With an explicit cache the internal command needs neither HOME nor the other two directories.
    let captured = captured(environment(&[("TEMPER_INTERNAL_SITE", "http://127.0.0.1:8443")]));
    let (code, line) = fetch(&words(&["--cache", "/cache"]), &captured).expect_err("site refusal before find");
    assert_eq!(code, 1);
    assert!(line.starts_with("temper: TEMPER_INTERNAL_SITE:"), "{line}");
    assert_eq!(fetch(&words(&["--data", "/data"]), &captured).expect_err("cache only").0, 2);
}

#[test]
fn a_fetch_is_announced_once_before_its_first_progress_line() {
    let mut stderr = Vec::new();
    {
        let mut out = Fetching { out: &mut stderr, started: false };
        out.write_all(b"").expect("empty write");
        out.flush().expect("flush");
    }
    assert!(stderr.is_empty(), "a found file says nothing");
    let mut out = Fetching { out: &mut stderr, started: false };
    out.write_all(b"fetched temper-box 0 of 3 bytes\n").expect("start");
    out.write_all(b"fetched temper-box 3 of 3 bytes\n").expect("end");
    assert_eq!(stderr, b"fetching temper-box\nfetched temper-box 0 of 3 bytes\nfetched temper-box 3 of 3 bytes\n");
}

#[test]
fn exec_keeps_every_argument_and_the_captured_environment_including_echo_steps() {
    let mut arguments = words(&["--cache", "/cache", "serve", "--echo", "steps", "--unknown", "a b", ""]).into_vec();
    arguments.push(b"non-utf8-\xff".as_slice().into());
    let captured = captured(environment(&[("HOME", "/home/ada"), ("BOX_TEST", "kept")]));
    assert!(resolve(&arguments, &captured.environment, b"/working").expect("echo").echo_steps);
    // There is no terminal branch yet: echo steps and every other form use this exec command.
    let command = exec_command(Path::new("/found/temper-box"), &arguments, &captured.environment);
    assert_eq!(command.get_program().as_bytes(), b"/found/temper-box");
    assert_eq!(
        command.get_args().map(OsStrExt::as_bytes).collect::<Vec<_>>(),
        arguments.iter().map(AsRef::as_ref).collect::<Vec<_>>()
    );
    assert_eq!(
        command.get_envs().map(|(k, v)| (k.as_bytes(), v.expect("set").as_bytes())).collect::<Vec<_>>(),
        vec![(b"BOX_TEST".as_slice(), b"kept".as_slice()), (b"HOME".as_slice(), b"/home/ada".as_slice())]
    );
}
