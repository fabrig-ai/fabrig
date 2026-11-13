use alloc::{boxed::Box, format};
use skein_io::{self as io, kernel};
use skein_lib::{List, Queue, Time, Token, stream};

use crate::{
    Command, Ended, Error, Launch, Method, Programs, Status, configuration_directory, email, environment, keeps, status,
};

fn programs(directory: &[u8]) -> Programs {
    Programs { directory: directory.into(), claude: b"claude".as_slice().into() }
}

fn entry(name: &[u8], value: &[u8]) -> Box<[u8]> {
    let length = name.len().checked_add(value.len()).expect("entry length").checked_add(1).expect("separator");
    let mut result = List::with_capacity(u32::try_from(length).expect("small entry"));
    for byte in name {
        result.push(*byte).expect("entry room");
    }
    result.push(b'=').expect("separator room");
    for byte in value {
        result.push(*byte).expect("entry room");
    }
    result.into_boxed()
}

#[test]
fn a_configuration_directory_is_named_by_its_source() {
    for directory in [b"/programs".as_slice(), b"/programs/"] {
        assert_eq!(
            configuration_directory(&programs(directory), b"alice-2").expect("valid source").as_ref(),
            b"/programs/alice-2"
        );
    }
    assert_eq!(configuration_directory(&programs(b"/"), &[b'a'; 32]).expect("root source").len(), 33);
    for name in [b"".as_slice(), b"../x", b"Upper", b"a_b", b"\0", &[b'a'; 33]] {
        assert_eq!(configuration_directory(&programs(b"/programs"), name), Err(Error::Name));
    }
    for directory in [b"".as_slice(), b"relative", b"/nul\0"] {
        assert_eq!(configuration_directory(&programs(directory), b"a"), Err(Error::Directory));
    }
}

#[test]
fn the_source_s_environment_isolates_anthropic_profiles() {
    for caller in [None, Some(entry(b"ANTHROPIC_CONFIG_DIR", b"/home/u/.config/anthropic"))] {
        let mut base = List::with_capacity(3);
        base.push(entry(b"HOME", b"/h")).expect("base room");
        base.push(entry(b"XDG_CONFIG_HOME", b"/h/.config")).expect("base room");
        if let Some(caller) = caller {
            base.push(caller).expect("base room");
        }
        assert_eq!(
            environment(base.as_slice(), b"/programs/a").expect("bounded environment").as_ref(),
            &[
                entry(b"HOME", b"/h"),
                entry(b"XDG_CONFIG_HOME", b"/h/.config"),
                entry(b"CLAUDE_CONFIG_DIR", b"/programs/a"),
                entry(b"ANTHROPIC_CONFIG_DIR", b"/programs/a/anthropic"),
            ]
        );
    }
    assert!(!keeps(b"ANTHROPIC_CONFIG_DIR"));
}

#[test]
fn the_source_s_environment_drops_other_logins() {
    let names: &[&[u8]] = &[
        b"CLAUDE_CONFIG_DIR",
        b"ANTHROPIC_API_KEY",
        b"ANTHROPIC_AUTH_TOKEN",
        b"ANTHROPIC_OTHER",
        b"ANTHROPIC_PROFILE",
        b"ANTHROPIC_FEDERATION_RULE_ID",
        b"ANTHROPIC_ORGANIZATION_ID",
        b"CLAUDE_CODE_OAUTH_TOKEN",
        b"CLAUDE_CODE_USE_BEDROCK",
        b"CLAUDE_CODE_USE_VERTEX",
        b"CLAUDE_CODE_USE_FOUNDRY",
        b"AWS_BEARER_TOKEN_BEDROCK",
    ];
    let mut base = List::with_capacity(16);
    for name in names {
        base.push(entry(name, b"old")).expect("base room");
    }
    base.push(b"HOME=/scratch/home".as_slice().into()).expect("base room");
    base.push(b"TERM=xterm".as_slice().into()).expect("base room");
    let result = environment(base.as_slice(), b"/programs/a").expect("bounded environment");
    assert_eq!(
        result.as_ref(),
        &[
            Box::from(b"HOME=/scratch/home".as_slice()),
            Box::from(b"TERM=xterm".as_slice()),
            Box::from(b"CLAUDE_CONFIG_DIR=/programs/a".as_slice()),
            Box::from(b"ANTHROPIC_CONFIG_DIR=/programs/a/anthropic".as_slice()),
        ]
    );
    for name in names {
        assert!(!keeps(name), "stripped name {name:?}");
    }
    for name in [b"HOME".as_slice(), b"TERM", b"CLAUDE_CODE_OTHER", b"ANTHROPIC", b"AWS_REGION"] {
        assert!(keeps(name), "retained name {name:?}");
        let entry = entry(name, b"x");
        assert_eq!(environment(core::slice::from_ref(&entry), b"/a").expect("kept name").first(), Some(&entry));
    }
}

#[test]
fn the_source_s_environment_drops_a_named_profile() {
    let base = [entry(b"ANTHROPIC_PROFILE", b"work"), entry(b"HOME", b"/h")];
    assert_eq!(
        environment(&base, b"/programs/a").expect("bounded environment").as_ref(),
        &[
            entry(b"HOME", b"/h"),
            entry(b"CLAUDE_CONFIG_DIR", b"/programs/a"),
            entry(b"ANTHROPIC_CONFIG_DIR", b"/programs/a/anthropic"),
        ]
    );
    assert!(!keeps(b"ANTHROPIC_PROFILE"));
}

#[test]
fn the_source_s_environment_drops_federation_variables() {
    let rule = entry(b"ANTHROPIC_FEDERATION_RULE_ID", b"rule");
    let organization = entry(b"ANTHROPIC_ORGANIZATION_ID", b"organization");
    for credentials in
        [core::slice::from_ref(&rule), core::slice::from_ref(&organization), &[rule.clone(), organization.clone()]]
    {
        let mut base = List::with_capacity(3);
        for credential in credentials {
            base.push(credential.clone()).expect("base room");
        }
        base.push(entry(b"HOME", b"/h")).expect("base room");
        assert_eq!(
            environment(base.as_slice(), b"/programs/a").expect("bounded environment").as_ref(),
            &[
                entry(b"HOME", b"/h"),
                entry(b"CLAUDE_CONFIG_DIR", b"/programs/a"),
                entry(b"ANTHROPIC_CONFIG_DIR", b"/programs/a/anthropic"),
            ]
        );
    }
    assert!(!keeps(b"ANTHROPIC_FEDERATION_RULE_ID"));
    assert!(!keeps(b"ANTHROPIC_ORGANIZATION_ID"));
}

#[test]
fn the_source_s_environment_drops_an_endpoint() {
    let base = [entry(b"ANTHROPIC_BASE_URL", b"https://example.com"), entry(b"HOME", b"/h")];
    assert_eq!(
        environment(&base, b"/programs/a").expect("bounded environment").as_ref(),
        &[
            entry(b"HOME", b"/h"),
            entry(b"CLAUDE_CONFIG_DIR", b"/programs/a"),
            entry(b"ANTHROPIC_CONFIG_DIR", b"/programs/a/anthropic"),
        ]
    );
    assert!(!keeps(b"ANTHROPIC_BASE_URL"));
}

#[test]
fn an_environment_past_its_bounds_is_refused() {
    let setting_bytes =
        b"CLAUDE_CONFIG_DIR=/a".len().checked_add(b"ANTHROPIC_CONFIG_DIR=/a/anthropic".len()).expect("settings length");
    let entry_bytes = 4096_usize.checked_sub(setting_bytes).expect("setting fits");
    for length in [entry_bytes, entry_bytes.checked_add(1).expect("one past")] {
        let mut entry = skein_lib::bytes::zeroed(length);
        entry.fill(b'x');
        *entry.get_mut(1).expect("equals slot") = b'=';
        let result = environment(&[entry], b"/a");
        if length == entry_bytes {
            assert_eq!(result.expect("exact environment fits").len(), 3);
        } else {
            assert_eq!(result, Err(Error::Environment));
        }
    }
    let mut base = List::with_capacity(64);
    for _ in 0_u32..62 {
        base.push(Box::from(b"A=x".as_slice())).expect("base room");
    }
    assert_eq!(environment(base.as_slice(), b"/a").expect("64 entries").len(), 64);
    base.push(b"A=x".as_slice().into()).expect("base room");
    assert_eq!(environment(base.as_slice(), b"/a"), Err(Error::Environment));
    for entry in [b"no-equal".as_slice(), b"A=x\0", b"ANTHROPIC_KEY=x\0"] {
        assert_eq!(environment(&[entry.into()], b"/a"), Err(Error::Environment));
    }
    assert_eq!(environment(&[], b"/nul\0"), Err(Error::Environment));
}

#[test]
fn each_status_reads_as_programs_md_says() {
    let otherwise = Status::Otherwise(Method::new(b"api_key").expect("printable method"));
    let cases: &[(u8, &[u8], Status)] = &[
        (0, br#"{"loggedIn":true,"authMethod":"claude.ai","extra":42}"#, Status::Usable),
        (0, br#"{"loggedIn":true,"authMethod":"api_key"}"#, otherwise),
        (
            0,
            br#"{"loggedIn":true,"authMethod":"third_party"}"#,
            Status::Otherwise(Method::new(b"third_party").expect("printable method")),
        ),
        (
            0,
            br#"{"loggedIn":true,"authMethod":"oidc_federation"}"#,
            Status::Otherwise(Method::new(b"oidc_federation").expect("printable method")),
        ),
        (0, br#"{"loggedIn":false,"authMethod":"none"}"#, Status::NotSignedIn),
        (1, b"whatever", Status::NotSignedIn),
        (0, b"not JSON", Status::Unreadable),
        (0, br#"{"loggedIn":true}"#, Status::Unreadable),
        (0, br#"{"authMethod":"claude.ai"}"#, Status::Unreadable),
        (0, br#"{"loggedIn":"true","authMethod":"claude.ai"}"#, Status::Unreadable),
        (0, br#"{"loggedIn":true,"authMethod":false}"#, Status::Unreadable),
        (0, br#"{"loggedIn":true,"authMethod":""}"#, Status::Unreadable),
        (0, br#"{"loggedIn":true,"authMethod":"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"}"#, Status::Unreadable),
        (0, br#"{"loggedIn":true,"authMethod":"bad method"}"#, Status::Unreadable),
        (0, br#"{"loggedIn":true,"loggedIn":false,"authMethod":"claude.ai"}"#, Status::Unreadable),
        (0, br#"[{"loggedIn":true,"authMethod":"claude.ai"}]"#, Status::Unreadable),
        (0, br#"{"loggedIn":true,"authMethod":"claude.ai"}{}"#, Status::Unreadable),
    ];
    for (code, bytes, expected) in cases {
        assert_eq!(status(*code, bytes), *expected, "{code}: {bytes:?}");
    }
    let mut padded = [b' '; 4097];
    for (target, byte) in padded.iter_mut().zip(br#"{"loggedIn":true,"authMethod":"claude.ai"}"#) {
        *target = *byte;
    }
    assert_eq!(status(0, padded.get(..4096).expect("exact bound")), Status::Usable);
    assert_eq!(status(0, &padded), Status::Unreadable);
    assert_eq!(status(1, &padded), Status::NotSignedIn);
    assert!(Method::new(b"\x7f").is_none());
    assert!(Method::new(b"\x80").is_none());
    assert_eq!(Method::new(&[b'x'; 32]).expect("maximum method").as_bytes(), &[b'x'; 32]);
}

#[test]
#[expect(clippy::disallowed_macros, reason = "test fixtures format JSON independently of the production collector")]
#[expect(clippy::type_complexity, reason = "each table row carries the exit, output, email and unchanged status")]
fn an_email_is_optional_bounded_text_from_a_usable_status() {
    const EMAIL_BYTES: usize = 255; // protocol/programs.md, section 3.
    let maximum = "x".repeat(EMAIL_BYTES);
    let overlong = "x".repeat(EMAIL_BYTES.checked_add(1).expect("one past"));
    let long = "x".repeat(EMAIL_BYTES.checked_add(2).expect("past collector cap"));
    let maximum_status = format!(r#"{{"loggedIn":true,"authMethod":"claude.ai","email":"{maximum}"}}"#);
    let overlong_status = format!(r#"{{"loggedIn":true,"authMethod":"claude.ai","email":"{overlong}"}}"#);
    let long_status = format!(r#"{{"loggedIn":true,"authMethod":"claude.ai","email":"{long}"}}"#);
    let cases: &[(u8, &[u8], Option<&[u8]>, Status)] = &[
        (
            0,
            br#"{"loggedIn":true,"authMethod":"claude.ai","email":"you@example.com"}"#,
            Some(b"you@example.com"),
            Status::Usable,
        ),
        (0, br#"{"loggedIn":true,"authMethod":"claude.ai"}"#, None, Status::Usable),
        (0, br#"{"loggedIn":true,"authMethod":"claude.ai","email":42}"#, None, Status::Usable),
        (0, br#"{"loggedIn":true,"authMethod":"claude.ai","email":""}"#, None, Status::Usable),
        (0, maximum_status.as_bytes(), Some(maximum.as_bytes()), Status::Usable),
        (0, overlong_status.as_bytes(), None, Status::Usable),
        (0, long_status.as_bytes(), None, Status::Usable),
        (0, br#"{"loggedIn":true,"authMethod":"claude.ai","email":"you\u0000@example.com"}"#, None, Status::Usable),
        (0, br#"{"loggedIn":true,"authMethod":"claude.ai","email":"you\n@example.com"}"#, None, Status::Usable),
        (0, br#"{"loggedIn":true,"authMethod":"claude.ai","email":"you\u007f@example.com"}"#, None, Status::Usable),
        (
            0,
            br#"{"loggedIn":true,"authMethod":"api_key","email":"you@example.com"}"#,
            None,
            Status::Otherwise(Method::new(b"api_key").expect("method")),
        ),
        (0, br#"{"loggedIn":false,"authMethod":"none","email":"you@example.com"}"#, None, Status::NotSignedIn),
        (1, br#"{"loggedIn":true,"authMethod":"claude.ai","email":"you@example.com"}"#, None, Status::NotSignedIn),
        (0, br#"{"loggedIn":true,"email":"you@example.com"}"#, None, Status::Unreadable),
        (0, br#"{"loggedIn":true,"authMethod":"claude.ai","email":"you@example.com""#, None, Status::Unreadable),
    ];
    for (code, stdout, expected, expected_status) in cases {
        assert_eq!(email(*code, stdout).as_deref(), *expected, "{code}: {stdout:?}");
        assert_eq!(status(*code, stdout), *expected_status, "{code}: {stdout:?}");
    }
}

const OWNER: Token = Token::new(100);
const CHILD: Token = Token::new(7);
const OUT: Token = Token::new(8);
const ERR: Token = Token::new(9);

fn command() -> (Command, Queue<io::Request>) {
    let mut below = Queue::with_capacity(64);
    let command = Command::new(
        OWNER,
        Launch {
            program: b"claude".as_slice().into(),
            arguments: Box::new([]),
            environment: Box::new([]),
            root: kernel::Fd::new(3),
            directory: b".".as_slice().into(),
        },
        Time::from_nanos(100),
        8,
        &mut below,
    );
    let io::Request::Spawn { spawn, .. } = below.pop().expect("spawn") else { unreachable!("spawn request") };
    assert_eq!(spawn.pipes.len(), 2);
    assert_eq!(spawn.pipes.first().expect("stdout").child, 1);
    assert_eq!(spawn.pipes.get(1).expect("stderr").child, 2);
    (command, below)
}

fn spawn(command: &mut Command, below: &mut Queue<io::Request>) {
    command.from_io(io::Event::Spawned { owner: OWNER, child: CHILD, pipes: Box::new([OUT, ERR]) }, below);
}

fn close(command: &mut Command, below: &mut Queue<io::Request>) {
    for owner in [OUT, ERR] {
        command.from_io(io::Event::Stream { owner, up: stream::Up::End }, below);
        command.from_io(io::Event::Closed { owner }, below);
        assert!(command.ended().is_none(), "child still owns its close");
    }
    command.from_io(io::Event::Closed { owner: OWNER }, below);
}

#[test]
fn a_refused_spawn_is_unspawned_after_closure() {
    let (mut command, mut below) = command();
    command.from_io(io::Event::Failed { owner: OWNER, error: io::Error::Other }, &mut below);
    assert!(command.ended().is_none());
    command.from_io(io::Event::Closed { owner: OWNER }, &mut below);
    assert_eq!(command.ended(), Some(Ended::Unspawned { detail: b"" }));
    assert_eq!(command.next_deadline(), None);
}

#[test]
fn output_waits_for_the_child_then_both_pipes_to_close() {
    let (mut command, mut below) = command();
    spawn(&mut command, &mut below);
    assert!(command.owns(OWNER));
    assert!(command.owns(OUT));
    assert!(command.owns(ERR));
    assert!(!command.owns(Token::new(999)));
    for (owner, bytes) in [(OUT, b"answer".as_slice()), (ERR, b"0123456789ABCDEFx".as_slice())] {
        command.from_io(io::Event::Stream { owner, up: stream::Up::Bytes(bytes.into()) }, &mut below);
    }
    command.from_io(io::Event::Exited { owner: OWNER, exit: kernel::Exit::Code(0) }, &mut below);
    command.from_io(io::Event::Closed { owner: OWNER }, &mut below);
    assert!(command.ended().is_none());
    for owner in [OUT, ERR] {
        command.from_io(io::Event::Stream { owner, up: stream::Up::End }, &mut below);
        command.from_io(io::Event::Closed { owner }, &mut below);
    }
    assert_eq!(
        command.ended(),
        Some(Ended::Exited { exit: kernel::Exit::Code(0), stdout: Some(b"answer"), detail: b"9ABCDEFx" })
    );
}

#[test]
fn the_deadline_kills_the_child_before_answering() {
    let (mut command, mut below) = command();
    spawn(&mut command, &mut below);
    while below.pop().is_some() {}
    command.fire(Time::from_nanos(99), &mut below);
    assert!(below.is_empty());
    command.fire(Time::from_nanos(100), &mut below);
    let Some(io::Request::Signal { child: CHILD, signal: kernel::Signal::Kill, .. }) = below.pop() else {
        panic!("deadline kill")
    };
    assert!(command.ended().is_none());
    assert_eq!(command.next_deadline(), None);
    close(&mut command, &mut below);
    assert_eq!(command.ended(), Some(Ended::Deadline { detail: b"" }));
}

#[test]
fn stdout_past_its_bound_is_absent() {
    for length in [4096, 4097, 8192] {
        let (mut command, mut below) = command();
        spawn(&mut command, &mut below);
        command.from_io(
            io::Event::Stream { owner: OUT, up: stream::Up::Bytes(skein_lib::bytes::zeroed(length)) },
            &mut below,
        );
        command.from_io(io::Event::Exited { owner: OWNER, exit: kernel::Exit::Code(0) }, &mut below);
        close(&mut command, &mut below);
        let Some(Ended::Exited { stdout, .. }) = command.ended() else { unreachable!("settled exit") };
        assert_eq!(stdout.is_some(), length == 4096);
    }
}

#[test]
fn a_spawn_arriving_after_the_deadline_is_killed() {
    let (mut command, mut below) = command();
    command.fire(Time::from_nanos(100), &mut below);
    spawn(&mut command, &mut below);
    let mut killed = false;
    while let Some(request) = below.pop() {
        if let io::Request::Signal { child: CHILD, signal: kernel::Signal::Kill, .. } = request {
            killed = true;
        }
    }
    assert!(killed);
    close(&mut command, &mut below);
    assert_eq!(command.ended(), Some(Ended::Deadline { detail: b"" }));
}

#[test]
fn cancellation_stops_once_before_settling() {
    let (mut command, mut below) = command();
    spawn(&mut command, &mut below);
    while below.pop().is_some() {}
    command.cancel(&mut below);
    command.cancel(&mut below);
    command.fire(Time::from_nanos(100), &mut below);
    assert_eq!(below.len(), 3);
    let Some(io::Request::Signal { child: CHILD, signal: kernel::Signal::Kill, .. }) = below.pop() else {
        panic!("cancel kill")
    };
    for pipe in [OUT, ERR] {
        let Some(io::Request::Abort { entity }) = below.pop() else { panic!("cancel abort") };
        assert_eq!(entity, pipe);
    }
    assert!(command.ended().is_none());
    close(&mut command, &mut below);
    while below.pop().is_some() {}
    command.cancel(&mut below);
    assert!(below.is_empty());
    assert!(command.ended().is_some());
}

#[test]
fn cancellation_before_spawn_stops_the_late_child() {
    let (mut command, mut below) = command();
    command.cancel(&mut below);
    assert!(below.is_empty());
    spawn(&mut command, &mut below);
    assert_eq!(below.len(), 5);
    assert!(command.ended().is_none());
    close(&mut command, &mut below);
    assert!(command.ended().is_some());
}

#[test]
fn command_launch_debug_omits_environment_values() {
    let launch = Launch {
        program: b"/claude".as_slice().into(),
        arguments: Box::new([]),
        environment: Box::new([b"PATH=/hidden-path".as_slice().into()]),
        root: kernel::Fd::new(3),
        directory: b".".as_slice().into(),
    };
    let expected = b"Launch { environment_entries: 1, .. }";
    let mut rendered = DebugOutput(skein_lib::Writer::new(expected.len()));
    core::fmt::write(&mut rendered, format_args!("{launch:?}")).unwrap();
    assert_eq!(rendered.0.finish().as_ref(), expected);
}

struct DebugOutput(skein_lib::Writer);

impl core::fmt::Write for DebugOutput {
    fn write_str(&mut self, text: &str) -> core::fmt::Result {
        match self.0.put(text.as_bytes()) {
            Ok(()) => Ok(()),
            Err(_) => Err(core::fmt::Error),
        }
    }
}
