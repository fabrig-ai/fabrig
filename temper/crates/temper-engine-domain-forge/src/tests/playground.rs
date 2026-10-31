//! Playground starts and reset cuts (temper/docs/design/draft/playground-reset.md, 6.1–6.3).
#![expect(clippy::disallowed_methods, reason = "ordinary tests inspect their captured request stream")]
#![expect(clippy::match_like_matches_macro, reason = "explicit test patterns keep matches out of step crates")]
use super::*;

fn place() -> Place {
    Place {
        api: Address { host: b"localhost".as_slice().into(), port: 3000, tls: None },
        git: b"http://localhost:3000".as_slice().into(),
        web: None,
    }
}

fn configured() -> Domain {
    let mut d = domain();
    d.under_box = true;
    d.set_machine(Machine { owned: None, fake: Some(place()), github: place() });
    d.configure_playground(Some(PlaygroundConfig { provider: 1, deadline: Duration::from_secs(10) }));
    d
}

fn ask(d: &mut Domain, number: u64, ask: Ask) -> Box<[Request]> {
    outputs(
        d,
        Event::Application {
            request: Token::new(number),
            token: Token::new(number.checked_add(100).unwrap()),
            retention: Duration::from_secs(60),
            role: None,
            carried: None,
            ask,
        },
    )
}

fn connect(d: &mut Domain) -> Box<[Request]> {
    let forge = d.playground().unwrap().forge;
    outputs(d, Event::Connection { forge, state: Connection::Connected { id: 7, login: b"temper".as_slice().into() } })
}

fn running() -> Domain {
    let mut d = configured();
    ask(&mut d, 1, Ask::StartPlayground);
    connect(&mut d);
    outputs(&mut d, Event::PlaygroundMade { project: 5, account: 3 });
    d
}

fn reset_act(sent: &[Request]) -> ([u8; 16], ResetAct) {
    for request in sent {
        if let Request::ResetAct { key, act } = request {
            return (*key, *act);
        }
    }
    panic!("one reset act")
}

#[test]
fn start_makes_a_fake_forge_and_asks_the_box_for_its_group() {
    let mut d = configured();
    let sent = ask(&mut d, 1, Ask::StartPlayground);
    let row = *d.playground().unwrap();
    assert_eq!(row, Playground { state: PlaygroundState::Starting, forge: 1, project: None, account: None });
    let forge = d.record(row.forge).unwrap();
    assert_eq!(forge.kind, Kind::Forgejo);
    assert_eq!(forge.mode, Mode::Fake);
    assert_eq!(forge.place, place());
    assert_eq!(forge.bot.login.as_ref(), b"temper");
    assert!(!forge.asked);
    assert_eq!(forge.first, None);
    assert!(sent.contains(&Request::BoxGroup { forge: row.forge, kind: BoxKind::Fake, request: Token::new(1) }));
    let save = sent
        .iter()
        .position(
            |request| if let Request::Save { record: Stored::Playground(_), .. } = request { true } else { false },
        )
        .unwrap();
    let group =
        sent.iter().position(|request| if let Request::BoxGroup { .. } = request { true } else { false }).unwrap();
    assert!(save < group);
}

#[test]
fn a_real_forge_kept_refuses_the_start_not_offered() {
    let mut d = configured();
    outputs(
        &mut d,
        Event::Restore {
            record: Stored::Forge(ForgeRecord {
                number: 2,
                kind: Kind::Forgejo,
                mode: Mode::Connected,
                state: ForgeState::Kept,
                place: place(),
                read_only: false,
                bot: Bot { login: Box::new([]), name: Box::new([]), email: Box::new([]) },
                asked: false,
                first: None,
            }),
        },
    );
    let sent = ask(&mut d, 1, Ask::StartPlayground);
    assert_eq!(sent.as_ref(), [Request::ApplicationDone { request: Token::new(1), token: Token::new(101) }]);
    assert_eq!(d.application_answer(Token::new(101)), Some(Answered::StartPlayground(StartAnswer::NotOffered)));
    assert_eq!(d.playground(), None);
}

#[test]
fn the_acts_wait_for_the_fake_forge_to_connect() {
    let mut d = configured();
    let sent = ask(&mut d, 1, Ask::StartPlayground);
    assert!(!sent.iter().any(|request| if let Request::PlaygroundActs { .. } = request { true } else { false }));
    assert!(connect(&mut d).contains(&Request::PlaygroundActs { request: Token::new(1), forge: 1 }));
    assert!(!connect(&mut d).iter().any(|request| if let Request::PlaygroundActs { .. } = request {
        true
    } else {
        false
    }));
}

#[test]
fn made_saves_running_and_answers_held_starts() {
    let mut d = configured();
    ask(&mut d, 1, Ask::StartPlayground);
    assert!(ask(&mut d, 2, Ask::StartPlayground).is_empty());
    connect(&mut d);
    let sent = outputs(&mut d, Event::PlaygroundMade { project: 5, account: 3 });
    let row = Playground { state: PlaygroundState::Running, forge: 1, project: Some(5), account: Some(3) };
    assert_eq!(
        sent.first(),
        Some(&Request::Save { record: Stored::Playground(row), release: None, read_afresh: None })
    );
    assert_eq!(d.playground(), Some(&row));
    for number in [1, 2] {
        assert!(sent.contains(&Request::ApplicationDone {
            request: Token::new(number),
            token: Token::new(number.checked_add(100).unwrap())
        }));
        assert_eq!(
            d.application_answer(Token::new(number.checked_add(100).unwrap())),
            Some(Answered::StartPlayground(StartAnswer::Running))
        );
    }
    assert_eq!(
        ask(&mut d, 3, Ask::StartPlayground).as_ref(),
        [Request::ApplicationDone { request: Token::new(3), token: Token::new(103) }]
    );
}

#[test]
fn end_resets_in_order_project_forge_account_then_erases_the_row() {
    let mut d = running();
    let sent = ask(&mut d, 2, Ask::EndPlayground);
    assert_eq!(d.playground().unwrap().state, PlaygroundState::Resetting(ResetStep::Project));
    let (key, act) = reset_act(&sent);
    assert_eq!(act, ResetAct::RemoveProject { project: 5 });
    assert_eq!(
        sent.first(),
        Some(&Request::Save { record: Stored::Playground(*d.playground().unwrap()), release: None, read_afresh: None })
    );
    assert!(ask(&mut d, 3, Ask::EndPlayground).is_empty());
    let mut sent = outputs(&mut d, Event::ResetActAnswered { key, keep: false });
    // Forge removal keeps its own answers; account removal follows their completion.
    if d.playground().unwrap().state == PlaygroundState::Resetting(ResetStep::Forge) {
        assert!(sent.contains(&Request::ForgeGone { forge: 1 }));
        let first = outputs(&mut d, Event::SecretsErased { forge: 1 });
        assert!(!first.iter().any(|request| if let Request::ResetAct { .. } = request { true } else { false }));
        sent = outputs(&mut d, Event::BoxRemoved { forge: 1 });
    }
    let (key, act) = reset_act(&sent);
    assert_eq!(act, ResetAct::RemoveAccount { account: 3 });
    assert_eq!(d.playground().unwrap().state, PlaygroundState::Resetting(ResetStep::Account));
    let sent = outputs(&mut d, Event::ResetActAnswered { key, keep: false });
    assert_eq!(sent.first(), Some(&Request::Erase { key: Key::Playground }));
    assert_eq!(d.playground(), None);
    for number in [2, 3] {
        assert!(sent.contains(&Request::ApplicationDone {
            request: Token::new(number),
            token: Token::new(number.checked_add(100).unwrap())
        }));
        assert_eq!(
            d.application_answer(Token::new(number.checked_add(100).unwrap())),
            Some(Answered::EndPlayground(EndAnswer::Reset))
        );
    }
}

#[test]
fn a_kept_answer_holds_its_step_for_the_next_start() {
    let mut d = running();
    let sent = ask(&mut d, 2, Ask::EndPlayground);
    let (key, act) = reset_act(&sent);
    assert!(outputs(&mut d, Event::ResetActAnswered { key, keep: true }).is_empty());
    assert_eq!(d.playground().unwrap().state, PlaygroundState::Resetting(ResetStep::Project));
    ask(&mut d, 3, Ask::StartPlayground);
    assert_eq!(d.application_answer(Token::new(103)), Some(Answered::StartPlayground(StartAnswer::NotOffered)));
    let mut restored = configured();
    outputs(&mut restored, Event::Restore { record: Stored::Playground(*d.playground().unwrap()) });
    let mut out = Queue::uncapped();
    crate::playground::restart(&mut restored, &mut out);
    assert_eq!(out.pop(), Some(Request::ResetAct { key, act }));
}

#[test]
fn a_restart_resends_the_step_it_cut() {
    for step in [ResetStep::Project, ResetStep::Forge, ResetStep::Account] {
        let d = running();
        let mut row = *d.playground().unwrap();
        row.state = PlaygroundState::Resetting(step);
        let forge = d.record(row.forge).unwrap().clone();
        let mut restored = configured();
        outputs(&mut restored, Event::Restore { record: Stored::Forge(forge) });
        outputs(&mut restored, Event::Restore { record: Stored::Playground(row) });
        outputs(&mut restored, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
        let sent = outputs(&mut restored, Event::ReadAfresh);
        match step {
            ResetStep::Project => assert_eq!(
                reset_act(&sent),
                (crate::playground::reset_key(row.forge, step), ResetAct::RemoveProject { project: 5 })
            ),
            ResetStep::Forge => assert!(
                sent.contains(&Request::ForgeGone { forge: row.forge })
                    || sent.iter().any(|request| {
                        if let Request::ResetAct { act: ResetAct::RemoveAccount { .. }, .. } = request {
                            true
                        } else {
                            false
                        }
                    })
            ),
            ResetStep::Account => assert_eq!(
                reset_act(&sent),
                (crate::playground::reset_key(row.forge, step), ResetAct::RemoveAccount { account: 3 })
            ),
        }
    }
}

#[test]
fn end_with_no_playground_is_not_offered() {
    let mut d = configured();
    assert_eq!(
        ask(&mut d, 1, Ask::EndPlayground).as_ref(),
        [Request::ApplicationDone { request: Token::new(1), token: Token::new(101) }]
    );
    assert_eq!(d.application_answer(Token::new(101)), Some(Answered::EndPlayground(EndAnswer::NotOffered)));
}

#[test]
fn failed_and_expired_starts_keep_starting_and_retry_the_same_forge() {
    for timeout in [false, true] {
        let mut d = configured();
        ask(&mut d, 1, Ask::StartPlayground);
        connect(&mut d);
        if timeout {
            let mut out = Queue::uncapped();
            let mut env = env();
            env.now = env.now.saturating_add(Duration::from_secs(10));
            fire(&mut d, &env, &mut out);
        } else {
            outputs(&mut d, Event::PlaygroundFailed { why: StartFailure::Account });
        }
        assert_eq!(
            d.application_answer(Token::new(101)),
            Some(Answered::StartPlayground(StartAnswer::Failed(if timeout {
                StartFailure::NotConnected
            } else {
                StartFailure::Account
            })))
        );
        assert_eq!(d.playground().unwrap().state, PlaygroundState::Starting);
        assert!(
            ask(&mut d, 2, Ask::StartPlayground)
                .contains(&Request::PlaygroundActs { request: Token::new(2), forge: 1 })
        );
        assert_eq!(d.records.len(), 1);
    }
}

#[test]
fn starting_and_running_restart_the_group_and_only_starting_makes_acts() {
    for running in [false, true] {
        let mut d = configured();
        ask(&mut d, 1, Ask::StartPlayground);
        if running {
            outputs(&mut d, Event::PlaygroundMade { project: 5, account: 3 });
        }
        let row = *d.playground().unwrap();
        let forge = d.record(row.forge).unwrap().clone();
        let mut restored = configured();
        outputs(&mut restored, Event::Restore { record: Stored::Forge(forge) });
        outputs(&mut restored, Event::Restore { record: Stored::Playground(row) });
        outputs(&mut restored, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
        assert!(outputs(&mut restored, Event::ReadAfresh).contains(&Request::BoxGroup {
            forge: 1,
            kind: BoxKind::Fake,
            request: Token::new(u64::MAX)
        }));
        assert_eq!(
            connect(&mut restored).iter().any(|request| if let Request::PlaygroundActs { .. } = request {
                true
            } else {
                false
            }),
            !running
        );
    }
}
