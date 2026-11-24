use super::peer::Peer;
use crate::harness::{LISTENING, Process, Run, listen, run};
use skein_fake_peers::Listening;
use skein_fake_person_browser::{
    Harness, Lent, Outcome, Script, Setup, Step,
    harness::{Command, Names, Output, Printed, Scripted, Sent, Shown, Stdin},
    person::{Find, Step as PersonStep},
};
use skein_io::kernel::{Exit, Spawn};
use skein_lib::{Duration, bytes};
use skein_ui::Role;
use skein_world::{Host, HostedProgram, Inherited};
use skein_world_tests::hosted::{Act, Address, Script as Child};

const SERVER: &str = "/scripted-server";
const COMMAND: &str = "/scripted-command";
const SECOND: Duration = Duration::from_secs(1);
const REQUEST: &[u8] = b"GET / HTTP/1.1\r\nHost: fixture\r\nConnection: close\r\n\r\n";
const READY: &[u8] = b"<!doctype html><h1>Ready</h1><input type=hidden value='fixture value'>";
const PEER: &str = "FIXTURE_PEER";
const SERVER_ENV: &str = "FIXTURE_SERVER";

fn harness(args: &[&str]) -> Harness {
    Harness::new(Setup {
        server: SERVER.into(),
        args: args.iter().copied().map(Into::into).collect(),
        printed: Printed { sent: Sent::Page, prefix: LISTENING.as_bytes(), on: Output::Stderr, names: Names::Address },
        restart: vec![],
        downloads: None,
        chromium: "/not-a-hosted-program".into(),
        expected: vec![],
        origins: vec![],
    })
}

fn server<'a, 'p, 'l>(spawn: &Spawn, inherited: &Inherited) -> Process<'a, 'p, 'l> {
    // Connect on startup, before opening the server's own listener or printing its address.
    let acts: Vec<_> = (0..spawn.args.len())
        .map(|index| Act::Exchange { address: Address::Argument(index), request: REQUEST })
        .chain([listen(inherited), Act::AnswerRequestsWithBody { status: 200, body: READY }])
        .collect();
    Process::Child(Box::new(Child::spawned(spawn, inherited, &acts)))
}

fn ready(args: &[&str], peers: &mut [Lent<'_>]) -> Child {
    let mut person = Script::new(Box::new([Step::Person(PersonStep::See {
        find: Find::exactly_named(Role::Heading { level: 1 }, b"Ready"),
        within: SECOND,
    })]));
    let browser = Scripted::new().page(
        b"/",
        Shown { nodes: vec![(b"heading".as_slice().into(), b"Ready".as_slice().into())], elements: vec![] },
    );
    let run = run(
        &mut harness(args),
        &mut [&mut person],
        peers,
        browser,
        [HostedProgram { program: SERVER.as_bytes().into(), make: server, instances: 1, operations: 8 }],
    );
    assert_eq!(run.outcome, Outcome::Done);
    let [server]: [Child; 1] = run.children.try_into().expect("one scripted server");
    assert_eq!(server.replies().len(), args.len());
    assert!(server.replies().iter().all(|reply| reply.ends_with(b"peer ready")));
    server
}

#[test]
fn a_lent_peer_listens_before_the_server_requests_it() {
    let mut peer = Peer::new();
    ready(&["{neighbor}"], &mut [Lent { name: b"neighbor", peer: &mut peer }]);
    assert_eq!(peer.observations, [REQUEST]);
}

#[test]
fn a_lent_peer_returns_empty_with_its_handle_usable() {
    let mut peer = Peer::new();
    let handle = peer.handle.clone();
    ready(&["{neighbor}"], &mut [Lent { name: b"neighbor", peer: &mut peer }]);
    assert!(peer.is_empty());
    assert_eq!(handle.get(), 1);
    assert_eq!(peer.observations.len(), 1);
}

#[test]
fn colliding_peer_tokens_reach_their_own_hosts() {
    let mut first = Peer::new();
    let mut second = Peer::new();
    let server = ready(
        &["{first}", "{second}"],
        &mut [Lent { name: b"first", peer: &mut first }, Lent { name: b"second", peer: &mut second }],
    );
    let expected = [first.address().to_string(), second.address().to_string()];
    assert!(server.args.iter().map(AsRef::as_ref).eq(expected.iter().map(String::as_bytes)));
    assert_eq!((first.observations.len(), second.observations.len()), (1, 1));
    assert!(first.is_empty() && second.is_empty());
}

#[test]
fn repeated_peer_arguments_each_reach_the_server() {
    let mut peer = Peer::new();
    ready(&["{neighbor}", "{neighbor}"], &mut [Lent { name: b"neighbor", peer: &mut peer }]);
    assert_eq!(peer.handle.get(), 2);
}

fn exchange<'a, 'p, 'l>(spawn: &Spawn, inherited: &Inherited, addresses: [Address; 2]) -> Process<'a, 'p, 'l> {
    let acts = addresses.map(|address| Act::Exchange { address, request: REQUEST });
    Process::Child(Box::new(Child::spawned(spawn, inherited, &acts)))
}

fn command_args<'a, 'p, 'l>(spawn: &Spawn, inherited: &Inherited) -> Process<'a, 'p, 'l> {
    exchange(spawn, inherited, [Address::Argument(0), Address::Argument(1)])
}

fn command_env<'a, 'p, 'l>(spawn: &Spawn, inherited: &Inherited) -> Process<'a, 'p, 'l> {
    exchange(spawn, inherited, [Address::Environment(PEER.as_bytes()), Address::Environment(SERVER_ENV.as_bytes())])
}

fn command(peer: &mut Peer, args: &[&str], env: &[(&str, &str)]) -> Run {
    let mut person = Script::new(Box::new([
        Step::Run(Command {
            program: COMMAND.into(),
            args: args.iter().copied().map(Into::into).collect(),
            env: env.iter().map(|&(name, value)| (name.into(), value.into())).collect(),
            stdin: Stdin::Closed,
            opens: None,
            within: SECOND,
        }),
        Step::Exited { code: 0, within: SECOND },
    ]));
    let run = run(
        &mut harness(&[]),
        &mut [&mut person],
        &mut [Lent { name: b"neighbor", peer }],
        Scripted::new(),
        [
            HostedProgram { program: SERVER.as_bytes().into(), make: server, instances: 1, operations: 8 },
            HostedProgram {
                program: COMMAND.as_bytes().into(),
                make: if env.is_empty() { command_args } else { command_env },
                instances: 1,
                operations: 8,
            },
        ],
    );
    assert_eq!(run.outcome, Outcome::Done);
    let child = &run.children[1];
    assert_eq!(child.exit(), Some(Exit::Code(0)));
    let [peer_reply, server_reply] = child.replies() else { panic!("one reply from each address") };
    assert!(peer_reply.ends_with(b"peer ready"));
    assert!(bytes::find(server_reply, b"<h1>Ready</h1>").is_some());
    run
}

#[test]
fn command_arguments_receive_peer_and_server_addresses() {
    let mut peer = Peer::new();
    command(&mut peer, &["{neighbor}", "{address}"], &[]);
    assert_eq!(peer.observations.len(), 1);
}

#[test]
fn command_environment_values_receive_peer_and_server_addresses() {
    let mut peer = Peer::new();
    let run = command(&mut peer, &[], &[(PEER, "{neighbor}"), (SERVER_ENV, "{address}")]);
    let server = run.children[0].listening_address().expect("scripted server listened");
    let expected = [format!("{PEER}={}", peer.address()), format!("{SERVER_ENV}={server}")];
    // Compare the entire environment: no inherited entries, including HOME.
    assert!(run.children[1].env().iter().map(AsRef::as_ref).eq(expected.iter().map(String::as_bytes)));
    assert_eq!(peer.observations.len(), 1);
}
