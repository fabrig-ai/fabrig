use super::{
    super::{SourceEffects, source_with},
    credentials::configuration,
};
use crate::forge::{
    command_tests::{Effects, Invocation, Output, login, owned_directory},
    test_fixtures::{self, SOURCE_LIST, capture, setup_bytes},
};
use skein_http::{
    Header, Version,
    client::{Framing, Response},
};
use skein_lib::Time;
use std::{cell::RefCell, collections::VecDeque};
use temper_config_shell::Shape;
use temper_forge_setup::machine::BoxKind;

const ISSUER: &str = "http://127.0.0.1:31000";
const DISCOVERY: &str = "http://127.0.0.1:31000/.well-known/openid-configuration";
const FORGE_SIGN_IN: &str = "http://127.0.0.1:3000/user/oauth2/temper";
const ADDED: &[u8] = b"source added\n";
const NOT_SERVED: &[u8] = b"source failed Forgejo does not serve the source\n";
const NOW: Time = Time::from_nanos(81);
const LIST: &[&str] = &["admin", "auth", "list"];
const ADD: &[&str] = &[
    "admin",
    "auth",
    "add-oauth",
    "--name",
    "temper",
    "--provider",
    "openidConnect",
    "--key",
    "forgejo",
    "--auto-discover-url",
    DISCOVERY,
];

#[derive(Debug, PartialEq, Eq)]
enum Effect {
    Now,
    Get(String),
    Command(Vec<String>),
}

fn response(status: u16, location: Option<&str>) -> Response {
    let headers = location.map_or_else(Vec::new, |location| {
        vec![Header { name: b"Location".as_slice().into(), value: location.as_bytes().into() }]
    });
    Response { version: Version::Http11, status, headers: headers.into_boxed_slice(), framing: Framing::Length(0) }
}

fn source_command(replies: impl IntoIterator<Item = Result<Response, String>>, listed: bool) -> (Output, Vec<Effect>) {
    let mut file = configuration(false);
    file.sign_in = None;
    file.test.google = None;
    file.openid.as_mut().expect("provider").clients[0].secret = None;
    let config = temper_config_shell::render(&file, Shape::EngineAlone);
    let setup = setup_bytes(false, true);
    let replies = RefCell::new(replies.into_iter().collect::<VecDeque<_>>());
    let effects = RefCell::new(Vec::new());
    let stdout = RefCell::new(Vec::new());
    let output = Invocation { number: 1, group: Some(BoxKind::Owned), setup: &setup, configuration: &config }.call(
        "source",
        &[],
        None,
        &Effects {
            directory: &owned_directory,
            login: &|user| Ok(login(user)),
            source: &|bytes, named, owned, place, deadline, client| {
                assert_eq!(place, &test_fixtures::place());
                assert_eq!(client, Some(b"forgejo".as_slice()));
                let until = NOW.saturating_add(deadline);
                source_with(
                    bytes,
                    named,
                    owned,
                    place,
                    deadline,
                    client,
                    &mut SourceEffects {
                        now: &|| {
                            effects.borrow_mut().push(Effect::Now);
                            NOW
                        },
                        get: &mut |url, cutoff| {
                            assert_eq!(cutoff, until);
                            effects.borrow_mut().push(get(std::str::from_utf8(url).expect("HTTP URL")));
                            replies.borrow_mut().pop_front().expect("only the supplied GETs, no redirect followed")
                        },
                        command: &mut |spec, cutoff| {
                            assert_eq!(cutoff, until);
                            assert_eq!(spec.program.as_ref(), b"/programs/forgejo");
                            let args = spec
                                .arguments
                                .iter()
                                .skip(4)
                                .map(|arg| String::from_utf8_lossy(arg).into_owned())
                                .collect();
                            effects.borrow_mut().push(Effect::Command(args));
                            let list = if listed {
                                SOURCE_LIST
                            } else {
                                b"ID Name Type Enabled\n26 temper-other OAuth2 true\n"
                            };
                            Ok(capture(0, list, b""))
                        },
                        out: &mut *stdout.borrow_mut(),
                    },
                )
            },
            ..Effects::recording(&stdout)
        },
    );
    assert!(replies.into_inner().is_empty(), "not all supplied GET replies were read: {output:?}");
    (output, effects.into_inner())
}

fn get(url: &str) -> Effect {
    Effect::Get(url.to_owned())
}

fn command(args: &[&str]) -> Effect {
    Effect::Command(args.iter().map(|s| (*s).to_owned()).collect())
}

fn output(stdout: &[u8]) -> Output {
    Output { stdout: stdout.into(), ..Output::silent() }
}

#[test]
fn source_discovery_rows() {
    for discovery in [Err("no answer".into()), Ok(response(502, None))] {
        assert_eq!(
            source_command([discovery], false),
            (output(b"source failed temper's discovery did not answer\n"), vec![Effect::Now, get(DISCOVERY)]),
        );
    }
}

#[test]
fn source_adds_after_discovery() {
    assert_eq!(
        source_command([Ok(response(200, None))], false),
        (output(ADDED), vec![Effect::Now, get(DISCOVERY), command(LIST), command(ADD)]),
    );
}

#[test]
fn listed_source_redirect_rows() {
    for (status, location, added) in [
        (307, "ISSUER/oauth/authorize?client_id=forgejo", true),
        (302, "ISSUER/oauth/authorize", true),
        (307, "ISSUER/oauth/authorize#continue", true),
        (307, "", false),
        (307, "/oauth/authorize", false),
        (200, "ISSUER/oauth/authorize", false),
        (500, "", false),
        (307, "https://other.invalid/oauth/authorize", false),
        (307, "ISSUER/oauth/authorize-other", false),
        (307, "ISSUER/oauth/authorize/child", false),
    ] {
        let target = location.replace("ISSUER", ISSUER);
        let replies = [Ok(response(200, None)), Ok(response(status, Some(&target)))];
        assert_eq!(
            source_command(replies, true),
            (
                output(if added { ADDED } else { NOT_SERVED }),
                vec![Effect::Now, get(DISCOVERY), command(LIST), get(FORGE_SIGN_IN)]
            ),
            "{status} {location}",
        );
    }
}
