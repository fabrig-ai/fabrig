//! Source scheduling uses typed fakes here; command I/O belongs to channel.
use jig_domain_host::{
    self as host,
    sources::{Answer, Operation},
};
use jig_domain_link::{LoginState, SignOutResult, SourceFailure, SourceName};
use skein_lib::{Duration, Env, Queue, Time, Token, Wall};

// Two sources exercise replacement at capacity and rejection one past it.
const SOURCE_CAPACITY: u32 = 2;

struct World {
    host: host::Hub,
    env: Env<host::Limits>,
    out: Queue<host::Request>,
}
impl World {
    fn new() -> Self {
        let mut limits = jig_host_world::Settings::calm(1).host;
        limits.program_sources = SOURCE_CAPACITY;
        limits.slots = 0;
        Self {
            host: host::Hub::new(&limits),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            out: Queue::uncapped(),
        }
    }
    fn checking(names: &[&[u8]]) -> (Self, Token) {
        let mut world = Self::new();
        let owner = command(&world.sources(names), names[0], Operation::Check);
        (world, owner)
    }
    fn usable() -> Self {
        let (mut world, owner) = Self::checking(&[b"a"]);
        world.checked(owner, LoginState::Usable);
        world
    }
    fn signing_out() -> (Self, Token) {
        let mut world = Self::usable();
        let owner = command(
            &world.step(host::Event::Down(host::LinkDown::SignOut { name: name_of(b"a") })),
            b"a",
            Operation::SignOut,
        );
        (world, owner)
    }
    fn step(&mut self, event: host::Event) -> Vec<host::Request> {
        host::step(&mut self.host, &self.env, event, &mut self.out);
        let mut out = Vec::new();
        while let Some(request) = self.out.pop() {
            out.push(request);
        }
        out
    }
    fn sources(&mut self, names: &[&[u8]]) -> Vec<host::Request> {
        self.step(host::Event::Down(host::LinkDown::Sources {
            sources: names.iter().map(|name| name_of(name)).collect(),
        }))
    }
    fn checked(&mut self, owner: Token, state: LoginState) -> Vec<host::Request> {
        self.step(host::Event::SourceAnswered { owner, answer: Answer::Checked { state, email: None } })
    }
}
fn name_of(name: &[u8]) -> SourceName {
    SourceName::new(name).expect("fixture source name")
}
fn command(requests: &[host::Request], name: &[u8], operation: Operation) -> Token {
    let [host::Request::Source { owner, name: actual, operation: actual_op, .. }] = requests else {
        panic!("{requests:?}")
    };
    assert_eq!(*actual, name_of(name));
    assert_eq!(*actual_op, operation);
    *owner
}
fn snapshot(request: &host::Request, names: &[&[u8]]) {
    let host::Request::Up(host::LinkUp::Logins { logins }) = request else { panic!("{request:?}") };
    assert_eq!(
        logins.iter().map(|login| login.name).collect::<Vec<_>>(),
        names.iter().map(|name| name_of(name)).collect::<Vec<_>>()
    );
}
#[test]
fn duplicate_sources_fault_the_link() {
    let mut world = World::new();
    assert_eq!(world.sources(&[b"a", b"a"]), [host::Request::SourceFault]);
}
#[test]
fn sources_one_past_capacity_fault_the_link() {
    let mut world = World::new();
    assert_eq!(world.sources(&[b"a", b"b", b"c"]), [host::Request::SourceFault]);
}
#[test]
fn replacement_installs_added_names_while_a_check_is_busy() {
    let (mut world, old) = World::checking(&[b"a", b"b"]);
    world.sources(&[b"b", b"c"]);
    assert_eq!(world.step(host::Event::Down(host::LinkDown::Check { name: name_of(b"c") })).len(), 0);
    let b = command(&world.checked(old, LoginState::Usable), b"b", Operation::Check);
    let c = command(&world.checked(b, LoginState::NotSignedIn), b"c", Operation::Check);
    snapshot(&world.checked(c, LoginState::NotSignedIn)[0], &[b"b", b"c"]);
}
#[test]
fn replacement_preserves_kept_sign_outs_while_a_check_is_busy() {
    let (mut world, old) = World::checking(&[b"a", b"b"]);
    world.step(host::Event::Down(host::LinkDown::SignOut { name: name_of(b"b") }));
    world.sources(&[b"b", b"c"]);
    let b = command(&world.checked(old, LoginState::Usable), b"b", Operation::SignOut);
    let answer =
        world.step(host::Event::SourceAnswered { owner: b, answer: Answer::SignedOut(SignOutResult::SignedOut) });
    assert_eq!(
        answer[0],
        host::Request::Up(host::LinkUp::SignedOut { name: name_of(b"b"), result: SignOutResult::SignedOut })
    );
}
#[test]
fn replacement_answers_unknown_only_for_a_removed_queued_sign_out() {
    let mut world = World::new();
    world.sources(&[b"a", b"b"]);
    world.step(host::Event::Down(host::LinkDown::SignOut { name: name_of(b"b") }));
    assert_eq!(
        world.sources(&[b"a", b"c"]),
        vec![host::Request::Up(host::LinkUp::SignedOut {
            name: name_of(b"b"),
            result: SignOutResult::Failed { reason: SourceFailure::Unknown, detail: Box::new([]) },
        })]
    );
}
#[test]
fn overlapping_explicit_checks_coalesce_into_one_fresh_check() {
    let (mut world, a) = World::checking(&[b"a"]);
    for _ in 0..2 {
        assert_eq!(world.step(host::Event::Down(host::LinkDown::Check { name: name_of(b"a") })).len(), 0);
    }
    let fresh = command(&world.checked(a, LoginState::NotSignedIn), b"a", Operation::Check);
    let answer = world.checked(fresh, LoginState::NotSignedIn);
    assert_eq!(answer.len(), 1);
    snapshot(&answer[0], &[b"a"]);
    assert!(world.host.sources_settled());
}
#[test]
fn periodic_checks_publish_only_when_state_or_email_changes() {
    for (state, before, after, changed) in [
        (LoginState::Usable, None, None, false),
        (LoginState::Usable, Some(b"you@example.com".as_slice()), Some(b"you@example.com".as_slice()), false),
        (LoginState::Usable, Some(b"you@example.com".as_slice()), Some(b"other@example.com".as_slice()), true),
        (LoginState::Usable, Some(b"you@example.com".as_slice()), None, true),
        (LoginState::NotSignedIn, Some(b"you@example.com".as_slice()), None, true),
    ] {
        let (mut world, owner) = World::checking(&[b"a"]);
        world.step(host::Event::SourceAnswered {
            owner,
            answer: Answer::Checked { state: LoginState::Usable, email: before.map(Box::from) },
        });
        world.env.now = Time::ZERO.saturating_add(Duration::from_secs(60));
        host::resume(&mut world.host, &world.env, &mut world.out);
        let periodic = world.out.pop().expect("interval check");
        let periodic = command(&[periodic], b"a", Operation::Check);
        let expected =
            jig_domain_link::Login { name: name_of(b"a"), state: state.clone(), email: after.map(Box::from) };
        let requests = world.step(host::Event::SourceAnswered {
            owner: periodic,
            answer: Answer::Checked { state, email: after.map(Box::from) },
        });
        let expected = if changed {
            vec![host::Request::Up(host::LinkUp::Logins { logins: Box::new([expected]) })]
        } else {
            vec![]
        };
        assert_eq!(requests, expected);
    }
}
#[test]
fn a_fresh_explicit_check_answers_even_when_unchanged() {
    let mut world = World::usable();
    world.env.now = Time::ZERO.saturating_add(Duration::from_secs(59));
    let explicit =
        command(&world.step(host::Event::Down(host::LinkDown::Check { name: name_of(b"a") })), b"a", Operation::Check);
    world.env.now = Time::ZERO.saturating_add(Duration::from_secs(60));
    host::resume(&mut world.host, &world.env, &mut world.out);
    assert!(world.out.is_empty(), "the active check also covers the expired interval");
    let answer = world.checked(explicit, LoginState::Usable);
    assert_eq!(answer.len(), 1);
    snapshot(&answer[0], &[b"a"]);
    assert!(world.host.sources_settled());
}
#[test]
fn empty_sources_still_publish_an_inventory() {
    let mut world = World::new();
    snapshot(&world.sources(&[])[0], &[]);
}
#[test]
fn an_unknown_source_check_faults_the_link() {
    let mut world = World::new();
    assert_eq!(
        world.step(host::Event::Down(host::LinkDown::Check { name: name_of(b"unknown") })),
        vec![host::Request::SourceFault]
    );
}
#[test]
fn an_unknown_source_sign_out_answers_unknown() {
    let mut world = World::new();
    let answer = world.step(host::Event::Down(host::LinkDown::SignOut { name: name_of(b"unknown") }));
    assert_eq!(
        answer,
        vec![host::Request::Up(host::LinkUp::SignedOut {
            name: name_of(b"unknown"),
            result: SignOutResult::Failed { reason: SourceFailure::Unknown, detail: Box::new([]) }
        })]
    );
}
#[test]
fn a_lost_sign_out_is_settled_without_replaying_its_reply_on_redial() {
    let mut world = World::new();
    let a = command(&world.sources(&[b"a"]), b"a", Operation::Check);
    world.checked(a, LoginState::Usable);
    let sign_out = command(
        &world.step(host::Event::Down(host::LinkDown::SignOut { name: name_of(b"a") })),
        b"a",
        Operation::SignOut,
    );
    assert_eq!(world.step(host::Event::SourceLost).len(), 0);
    assert_eq!(world.sources(&[b"a"]).len(), 0);
    assert_eq!(world.step(host::Event::Down(host::LinkDown::Check { name: name_of(b"a") })).len(), 0);
    let next = world
        .step(host::Event::SourceAnswered { owner: sign_out, answer: Answer::SignedOut(SignOutResult::SignedOut) });
    let check = command(&next, b"a", Operation::Check);
    assert_ne!(check, sign_out);
    snapshot(&world.checked(check, LoginState::NotSignedIn)[0], &[b"a"]);
    assert_eq!(
        world
            .step(host::Event::SourceAnswered { owner: sign_out, answer: Answer::SignedOut(SignOutResult::SignedOut) })
            .len(),
        0
    );
}
#[test]
fn shutdown_preserves_the_active_sign_outs_real_result() {
    let (mut world, sign_out) = World::signing_out();
    world.step(host::Event::CancelAll { reason: host::Reason::Shutdown });
    assert!(!world.host.sources_settled());
    let answer = world
        .step(host::Event::SourceAnswered { owner: sign_out, answer: Answer::SignedOut(SignOutResult::SignedOut) });
    assert_eq!(
        answer,
        vec![host::Request::Up(host::LinkUp::SignedOut { name: name_of(b"a"), result: SignOutResult::SignedOut })]
    );
    assert!(world.host.sources_settled());
    assert_eq!(world.host.next_deadline(), None);
}

#[test]
fn source_updates_during_shutdown_leave_the_link_open_for_run_acknowledgements() {
    let (mut world, check) = World::checking(&[b"a"]);
    world.step(host::Event::CancelAll { reason: host::Reason::Shutdown });
    assert!(world.sources(&[]).is_empty(), "a late list must not fault the draining link");
    assert!(world.sources(&[b"b"]).is_empty(), "shutdown admits no replacement source work");
    assert_eq!(world.step(host::Event::Down(host::LinkDown::Check { name: name_of(b"a") })).len(), 0);
    assert!(!world.host.sources_settled(), "the admitted check still owns its command");
    assert_eq!(world.checked(check, LoginState::Usable).len(), 0);
    assert!(world.host.sources_settled());
    assert_eq!(world.host.next_deadline(), None);
}
#[test]
fn sign_out_after_shutdown_answers_closed_without_starting_work() {
    let mut world = World::usable();
    world.step(host::Event::CancelAll { reason: host::Reason::Shutdown });
    for _ in 0..2 {
        assert_eq!(
            world.step(host::Event::Down(host::LinkDown::SignOut { name: name_of(b"a") })),
            vec![host::Request::Up(host::LinkUp::SignedOut {
                name: name_of(b"a"),
                result: SignOutResult::Failed { reason: SourceFailure::Closed, detail: Box::new([]) }
            })]
        );
    }
    assert!(world.host.sources_settled());
    assert_eq!(world.host.next_deadline(), None);
}

#[test]
fn shutdown_answers_closed_only_for_sign_outs_that_never_started() {
    let mut world = World::new();
    let check = command(&world.sources(&[b"a"]), b"a", Operation::Check);
    assert_eq!(world.step(host::Event::Down(host::LinkDown::SignOut { name: name_of(b"a") })).len(), 0);
    assert_eq!(
        world.step(host::Event::CancelAll { reason: host::Reason::Shutdown }),
        vec![
            host::Request::Up(host::LinkUp::Draining),
            host::Request::Up(host::LinkUp::SignedOut {
                name: name_of(b"a"),
                result: SignOutResult::Failed { reason: SourceFailure::Closed, detail: Box::new([]) }
            })
        ]
    );
    assert_eq!(world.checked(check, LoginState::Usable).len(), 0);
    assert!(world.host.sources_settled());
}

fn list_grant() -> jig_domain_link::Grant {
    jig_domain_link::Grant {
        account: 17,
        generation: 3,
        valid: Duration::from_secs(10),
        form: jig_domain_link::Form::Credential(jig_domain_link::Scheme::Key),
    }
}
fn read(read: u64) -> host::Event {
    host::Event::Down(host::LinkDown::ReadList { read, endpoint: 7, grant: list_grant() })
}
fn read_request(read: u64) -> host::Request {
    host::Request::ReadList { read, endpoint: 7, grant: list_grant() }
}
fn listed(read: u64) -> host::Event {
    host::Event::ListRead { read, outcome: jig_domain_link::ListOutcome::Listed(Box::new([])) }
}
fn list_answer(read: u64) -> host::Request {
    host::Request::Up(host::LinkUp::ListRead { read, outcome: jig_domain_link::ListOutcome::Listed(Box::new([])) })
}
#[test]
fn list_reads_share_the_source_queue_without_taking_slots() {
    for case in ["full", "check", "sign out", "check after read", "lost", "overflow", "reconnect", "shutdown"] {
        let mut world = World::new();
        world.env.limits.slots = 1;
        world.host = host::Hub::new(&world.env.limits);
        world.sources(&[]);
        world.step(jig_host_world::fixtures::assign(host::RunAssignment {
            identity: [0; 16],
            run: Token::new(41),
            attempt: Token::new(42),
            workspace: None,
            save: false,
            live: host::LiveAsk::NONE,
            charter: b"charter".as_slice().into(),
            grants: Box::new([]),
        }));
        let owner = command(&world.sources(&[b"a"]), b"a", Operation::Check);
        world.checked(owner, LoginState::Usable);
        match case {
            "check" | "sign out" => {
                let operation = if case == "check" { Operation::Check } else { Operation::SignOut };
                let event = if case == "check" {
                    host::LinkDown::Check { name: name_of(b"a") }
                } else {
                    host::LinkDown::SignOut { name: name_of(b"a") }
                };
                let owner = command(&world.step(host::Event::Down(event)), b"a", operation);
                assert!(world.step(read(1)).is_empty(), "{case}: read waits");
                let answer = if case == "check" {
                    Answer::Checked { state: LoginState::Usable, email: None }
                } else {
                    Answer::SignedOut(SignOutResult::SignedOut)
                };
                let outputs = world.step(host::Event::SourceAnswered { owner, answer });
                assert_eq!(outputs.last(), Some(&read_request(1)), "{case}: read follows source");
                let outputs = world.step(listed(1));
                assert_eq!(outputs.first(), Some(&list_answer(1)), "{case}");
                if case == "sign out" {
                    command(&outputs[1..], b"a", Operation::Check);
                }
            }
            _ => {
                assert_eq!(world.host.hosted(), world.env.limits.slots, "{case}: every slot is taken");
                assert_eq!(world.step(read(1)), [read_request(1)], "{case}: the hub forwards only grant names");
                assert_eq!(world.host.hosted(), world.env.limits.slots, "{case}: read takes no slot");
                match case {
                    "full" => assert_eq!(world.step(listed(1)), [list_answer(1)]),
                    "check after read" => {
                        assert_eq!(
                            world.step(host::Event::Down(host::LinkDown::Check { name: name_of(b"a") })).len(),
                            0
                        );
                        let outputs = world.step(listed(1));
                        assert_eq!(outputs[0], list_answer(1));
                        command(&outputs[1..], b"a", Operation::Check);
                    }
                    "lost" | "reconnect" => {
                        assert_eq!(world.step(host::Event::SourceLost), [host::Request::CancelRead { read: 1 }]);
                        assert!(!world.host.sources_settled(), "cancel must settle before reuse");
                        if case == "reconnect" {
                            assert!(world.sources(&[b"a"]).is_empty(), "old read still owns source queue");
                            command(&world.step(listed(1)), b"a", Operation::Check);
                        } else {
                            assert!(world.step(listed(1)).is_empty(), "lost read sends nothing up");
                            assert!(world.host.sources_settled());
                        }
                    }
                    "overflow" => {
                        assert_eq!(world.step(read(2)).len(), 0);
                        assert_eq!(
                            world.step(read(3)),
                            [host::Request::Up(host::LinkUp::ListRead {
                                read: 3,
                                outcome: jig_domain_link::ListOutcome::Failed(jig_domain_link::ListFailure::Agent),
                            })]
                        );
                        assert_eq!(world.step(listed(1)), [list_answer(1), read_request(2)]);
                        assert_eq!(world.step(listed(2)), [list_answer(2)]);
                    }
                    "shutdown" => {
                        assert_eq!(
                            world.step(host::Event::CancelAll { reason: host::Reason::Shutdown }),
                            [
                                host::Request::Up(host::LinkUp::Draining),
                                host::Request::Up(host::LinkUp::ListRead {
                                    read: 1,
                                    outcome: jig_domain_link::ListOutcome::Failed(jig_domain_link::ListFailure::Agent)
                                }),
                                host::Request::CancelRead { read: 1 },
                            ]
                        );
                        assert_eq!(world.step(listed(1)).len(), 0);
                        assert!(world.host.sources_settled());
                    }
                    _ => unreachable!("table case"),
                }
            }
        }
    }
}
