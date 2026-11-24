use super::*;
use std::os::unix::ffi::OsStringExt;

use skein_scratch::Scratch;

#[test]
fn preparing_refuses_a_bad_name() {
    let scratch = Scratch::new("program-name");
    let programs = scratch.path().join("programs");
    for name in ["", &"a".repeat(33), "A", "..", "a/b"] {
        assert!(matches!(prepare(&programs, name), Err(Refusal::Name)), "bad source {name:?}");
        assert!(!programs.exists(), "bad name makes nothing");
    }
    let longest = "a".repeat(32);
    assert_eq!(prepare(&programs, &longest).expect("maximum source name"), programs.join(longest));
}

#[test]
fn preparing_refuses_a_relative_directory() {
    assert!(
        matches!(prepare(Path::new("relative"), "source"), Err(Refusal::Relative(_))),
        "relative directory refused"
    );
    assert!(matches!(prepare(Path::new("relative"), ".."), Err(Refusal::Name)), "name checked first");
}

#[test]
fn the_sign_in_runs_auth_login_in_the_source_s_environment() {
    let program = Path::new("/usr/bin/claude");
    let configuration = Path::new("/private/programs/source");
    let base = [
        ("HOME", "/private/home"),
        ("XDG_CONFIG_HOME", "/private/home/.config"),
        ("ANTHROPIC_API_KEY", "key"),
        ("ANTHROPIC_CONFIG_DIR", "/home/u/.config/anthropic"),
        ("ANTHROPIC_AUTH_TOKEN", "token"),
        ("CLAUDE_CODE_OAUTH_TOKEN", "token"),
        ("CLAUDE_CODE_USE_BEDROCK", "1"),
        ("AWS_BEARER_TOKEN_BEDROCK", "token"),
        ("CLAUDE_CONFIG_DIR", "/person/config"),
    ];
    let command = sign_in(program, configuration, base.map(|(name, value)| (name.into(), value.into())));
    assert_eq!(command.get_program(), program);
    assert_eq!(command.get_args().collect::<Vec<_>>(), ["auth", "login"]);
    assert_eq!(
        command.get_envs().collect::<Vec<_>>(),
        [
            (OsStr::new("ANTHROPIC_CONFIG_DIR"), Some(configuration.join("anthropic").as_os_str())),
            (OsStr::new("CLAUDE_CONFIG_DIR"), Some(configuration.as_os_str())),
            (OsStr::new("HOME"), Some(OsStr::new("/private/home"))),
            (OsStr::new("XDG_CONFIG_HOME"), Some(OsStr::new("/private/home/.config"))),
        ]
    );
}

#[test]
fn the_sign_in_s_environment_drops_a_named_profile() {
    let base = [("ANTHROPIC_PROFILE", "work"), ("HOME", "/h")];
    let command = sign_in(
        Path::new("/usr/bin/claude"),
        Path::new("/programs/a"),
        base.map(|(name, value)| (name.into(), value.into())),
    );
    assert_eq!(
        command.get_envs().collect::<Vec<_>>(),
        [
            (OsStr::new("ANTHROPIC_CONFIG_DIR"), Some(OsStr::new("/programs/a/anthropic"))),
            (OsStr::new("CLAUDE_CONFIG_DIR"), Some(OsStr::new("/programs/a"))),
            (OsStr::new("HOME"), Some(OsStr::new("/h"))),
        ]
    );
}

#[test]
fn the_sign_in_s_environment_drops_federation_variables() {
    let rule = ("ANTHROPIC_FEDERATION_RULE_ID", "rule");
    let organization = ("ANTHROPIC_ORGANIZATION_ID", "organization");
    for credentials in [std::slice::from_ref(&rule), std::slice::from_ref(&organization), &[rule, organization]] {
        let base = credentials.iter().copied().chain([("HOME", "/h")]);
        let command = sign_in(
            Path::new("/usr/bin/claude"),
            Path::new("/programs/a"),
            base.map(|(name, value)| (name.into(), value.into())),
        );
        assert_eq!(
            command.get_envs().collect::<Vec<_>>(),
            [
                (OsStr::new("ANTHROPIC_CONFIG_DIR"), Some(OsStr::new("/programs/a/anthropic"))),
                (OsStr::new("CLAUDE_CONFIG_DIR"), Some(OsStr::new("/programs/a"))),
                (OsStr::new("HOME"), Some(OsStr::new("/h"))),
            ]
        );
    }
}

#[test]
fn the_sign_in_s_environment_drops_an_endpoint() {
    let base = [("ANTHROPIC_BASE_URL", "https://example.com"), ("HOME", "/h")];
    let command = sign_in(
        Path::new("/usr/bin/claude"),
        Path::new("/programs/a"),
        base.map(|(name, value)| (name.into(), value.into())),
    );
    assert_eq!(
        command.get_envs().collect::<Vec<_>>(),
        [
            (OsStr::new("ANTHROPIC_CONFIG_DIR"), Some(OsStr::new("/programs/a/anthropic"))),
            (OsStr::new("CLAUDE_CONFIG_DIR"), Some(OsStr::new("/programs/a"))),
            (OsStr::new("HOME"), Some(OsStr::new("/h"))),
        ]
    );
}

#[test]
fn the_terminal_environment_has_no_hosted_spawn_bound() {
    let mut base: Vec<_> = (0..100).map(|index| (format!("VARIABLE_{index}").into(), "x".repeat(100).into())).collect();
    base.push((OsString::from_vec(vec![b'N', 0xff]), OsString::from_vec(vec![0xfe])));
    let command = sign_in(Path::new("/claude"), Path::new("/programs/source"), base);
    assert_eq!(command.get_envs().count(), 103);
}

#[test]
fn private_program_directories_are_decided_from_metadata_at_the_name() {
    let path = Path::new("/fixture/programs");
    let user = 1000;
    let private = Stat { kind: Kind::Directory, mode: 0o700, owner: user, size: 0, links: 1 };
    assert!(!private_directory_with(path, user, Err(std::io::ErrorKind::NotFound.into())).expect("absent"));
    assert!(private_directory_with(path, user, Ok(private)).expect("private"));
    for (metadata, reason) in [
        (Stat { kind: Kind::Symlink, ..private }, "symbolic link"),
        (Stat { mode: 0o750, ..private }, "nonprivate mode bits 0o50"),
        (Stat { kind: Kind::File, ..private }, "not a directory"),
        (Stat { owner: 1001, ..private }, "owned by user 1001"),
    ] {
        let why = private_directory_with(path, user, Ok(metadata)).expect_err("unsafe");
        assert_eq!(why.to_string(), format!("{}: {reason}", path.display()));
    }
}
