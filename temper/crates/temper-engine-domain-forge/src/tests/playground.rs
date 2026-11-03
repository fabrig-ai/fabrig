//! Playground starts and reset cuts (temper/docs/design/protocol/setup.md, 8.4).
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
    let mut d = Domain::new(
        &LIMITS,
        4,
        skein_forge_client::Config { namespace: b"test".as_slice().into(), writers: Box::new([]) },
        &[],
        &[],
        true,
    )
    .expect("empty connector");
    d.set_machine(Machine { owned: None, fake: Some(place()), github: place(), github_git_program: None });
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
                git_program: None,
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
    let sent = outputs(&mut d, Event::ResetActAnswered { key, keep: false });
    assert_eq!(d.playground().unwrap().state, PlaygroundState::Resetting(ResetStep::Forge));
    assert!(sent.contains(&Request::ForgeGone { forge: 1 }));
    assert!(sent.contains(&Request::BoxRemove { forge: 1 }));
    assert!(!sent.iter().any(|request| if let Request::ResetAct { .. } = request { true } else { false }));
    let first = outputs(&mut d, Event::SecretsErased { forge: 1 });
    assert!(!first.iter().any(|request| if let Request::ResetAct { .. } = request { true } else { false }));
    let sent = outputs(&mut d, Event::BoxRemoved { forge: 1 });
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
            ResetStep::Forge => assert!(sent.contains(&Request::ForgeGone { forge: row.forge })),
            ResetStep::Account => assert_eq!(
                reset_act(&sent),
                (crate::playground::reset_key(row.forge, step), ResetAct::RemoveAccount { account: 3 })
            ),
        }
    }
}

#[test]
fn a_restart_keeps_resetting_until_both_forge_removal_steps_answer() {
    for (secrets_erased, box_removed) in [(false, false), (false, true), (true, false)] {
        let d = running();
        let mut row = *d.playground().unwrap();
        row.state = PlaygroundState::Resetting(ResetStep::Forge);
        let mut forge = d.record(row.forge).unwrap().clone();
        forge.state = ForgeState::Ended { secrets_erased, box_removed };
        let mut restored = configured();
        outputs(&mut restored, Event::Restore { record: Stored::Forge(forge) });
        outputs(&mut restored, Event::Restore { record: Stored::Playground(row) });
        let sent = outputs(&mut restored, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
        assert_eq!(sent.contains(&Request::ForgeGone { forge: row.forge }), !secrets_erased);
        assert_eq!(sent.contains(&Request::BoxRemove { forge: row.forge }), !box_removed);
        outputs(&mut restored, Event::ReadAfresh);
        assert_eq!(restored.playground(), Some(&row), "the reload still reads resetting");
        assert!(ask(&mut restored, 2, Ask::EndPlayground).is_empty(), "another end waits on the same removal");
        let mut sent: Box<[Request]> = Box::new([]);
        if !secrets_erased {
            sent = outputs(&mut restored, Event::SecretsErased { forge: row.forge });
            if !box_removed {
                assert_eq!(restored.playground(), Some(&row));
                assert!(!sent.iter().any(|request| if let Request::ResetAct { .. } = request { true } else { false }));
            }
        }
        if !box_removed {
            sent = outputs(&mut restored, Event::BoxRemoved { forge: row.forge });
        }
        let (key, act) = reset_act(&sent);
        assert_eq!(act, ResetAct::RemoveAccount { account: row.account.unwrap() });
        assert_eq!(restored.playground().unwrap().state, PlaygroundState::Resetting(ResetStep::Account));
        let sent = outputs(&mut restored, Event::ResetActAnswered { key, keep: false });
        assert_eq!(restored.playground(), None);
        assert_eq!(sent.first(), Some(&Request::Erase { key: Key::Playground }));
        assert_eq!(restored.application_answer(Token::new(102)), Some(Answered::EndPlayground(EndAnswer::Reset)));
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
fn a_start_cut_by_a_restart_resets_the_playground() {
    for (project, account) in [(None, None), (None, Some(3)), (Some(5), Some(3))] {
        let mut d = configured();
        ask(&mut d, 1, Ask::StartPlayground);
        let mut row = *d.playground().unwrap();
        row.project = project;
        row.account = account;
        let forge = d.record(row.forge).unwrap().clone();
        let mut restored = configured();
        outputs(&mut restored, Event::Restore { record: Stored::Forge(forge) });
        outputs(&mut restored, Event::Restore { record: Stored::Playground(row) });
        outputs(&mut restored, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
        let sent = outputs(&mut restored, Event::ReadAfresh);
        if let Some(project) = project {
            assert_eq!(reset_act(&sent).1, ResetAct::RemoveProject { project });
        } else {
            assert!(sent.contains(&Request::ForgeGone { forge: 1 }));
        }
        assert!(!connect(&mut restored).iter().any(|request| if let Request::PlaygroundActs { .. } = request {
            true
        } else {
            false
        }));
    }
}

#[test]
fn a_restart_with_the_playground_running_resets_it() {
    let d = running();
    let row = *d.playground().unwrap();
    let forge = d.record(row.forge).unwrap().clone();
    let mut restored = configured();
    outputs(&mut restored, Event::Restore { record: Stored::Forge(forge) });
    outputs(&mut restored, Event::Restore { record: Stored::Playground(row) });
    outputs(&mut restored, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    let sent = outputs(&mut restored, Event::ReadAfresh);
    let resetting = Playground { state: PlaygroundState::Resetting(ResetStep::Project), ..row };
    assert_eq!(restored.playground(), Some(&resetting));
    assert_eq!(
        sent.first(),
        Some(&Request::Save { record: Stored::Playground(resetting), release: None, read_afresh: None })
    );
    assert_eq!(
        reset_act(&sent),
        (crate::playground::reset_key(row.forge, ResetStep::Project), ResetAct::RemoveProject { project: 5 })
    );
    assert!(!sent.iter().any(|request| if let Request::BoxGroup { .. } = request { true } else { false }));
    assert!(!connect(&mut restored).iter().any(|request| if let Request::PlaygroundActs { .. } = request {
        true
    } else {
        false
    }));
}

#[test]
fn reset_keys_use_the_forge_and_step_and_ignore_unrelated_answers() {
    assert_eq!(
        crate::playground::reset_key(1, ResetStep::Project),
        [191, 33, 239, 218, 184, 126, 213, 54, 171, 141, 50, 254, 98, 49, 70, 234]
    );
    assert_eq!(
        crate::playground::reset_key(1, ResetStep::Account),
        [54, 52, 76, 31, 237, 240, 117, 162, 96, 107, 155, 130, 105, 91, 124, 177]
    );
    assert_eq!(
        crate::playground::reset_key(2, ResetStep::Project),
        [4, 144, 89, 83, 69, 90, 149, 104, 187, 90, 248, 123, 62, 238, 220, 46]
    );
    let mut d = running();
    ask(&mut d, 2, Ask::EndPlayground);
    assert!(outputs(&mut d, Event::ResetActAnswered { key: [0; 16], keep: false }).is_empty());
    assert_eq!(d.playground().unwrap().state, PlaygroundState::Resetting(ResetStep::Project));
}

#[test]
fn ending_a_start_skips_absent_acts_and_answers_held_starts() {
    let mut d = configured();
    ask(&mut d, 1, Ask::StartPlayground);
    let sent = ask(&mut d, 2, Ask::EndPlayground);
    assert!(sent.contains(&Request::ForgeGone { forge: 1 }));
    assert_eq!(d.application_answer(Token::new(101)), Some(Answered::StartPlayground(StartAnswer::NotOffered)));
    assert_eq!(d.playground().unwrap().state, PlaygroundState::Resetting(ResetStep::Forge));
    outputs(&mut d, Event::BoxRemoved { forge: 1 });
    let sent = outputs(&mut d, Event::SecretsErased { forge: 1 });
    assert!(sent.contains(&Request::Erase { key: Key::Playground }));
    assert_eq!(d.playground(), None);
    assert_eq!(d.application_answer(Token::new(102)), Some(Answered::EndPlayground(EndAnswer::Reset)));
}
