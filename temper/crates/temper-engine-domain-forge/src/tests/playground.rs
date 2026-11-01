//! Playground decisions and restart cuts (temper/docs/design/draft/playground-reset.md, 6, 9).
use super::*;

fn template() -> ForgeRecord {
    ForgeRecord {
        kind: Kind::Forgejo,
        mode: Mode::Fake,
        playground: true,
        setup: Some(br#"{"mode":"fake"}"#.as_slice().into()),
        ..ForgeRecord::github_com()
    }
}

fn playground(under_box: bool) -> Domain {
    let mut d = setup_domain(&[], under_box);
    d.machine.playground = Some(template());
    d
}

fn application(d: &mut Domain, number: u64, ask: Ask) -> Box<[Request]> {
    outputs(
        d,
        Event::Application {
            request: Token::new(number),
            token: Token::new(number.checked_add(100).unwrap()),
            retention: Duration::from_secs(60),
            ask,
            role: Some(jig_domain_people::Role::Owner),
            carried: None,
        },
    )
}

fn start(d: &mut Domain, number: u64) -> Box<[Request]> {
    application(d, number, Ask::StartPlayground)
}

fn end(d: &mut Domain, number: u64) -> Box<[Request]> {
    application(d, number, Ask::EndPlayground)
}

fn running() -> Playground {
    Playground { state: PlaygroundState::Running, forge: 1, project: None, account: None }
}

fn save(row: Playground) -> Request {
    Request::Save { read_afresh: None, release: None, record: Stored::Playground(row) }
}

fn resetting(row: Playground, step: ResetStep) -> Playground {
    Playground { state: PlaygroundState::Resetting(step), ..row }
}

fn done(number: u64) -> Request {
    Request::ApplicationDone { request: Token::new(number), token: Token::new(number.checked_add(100).unwrap()) }
}

fn answer(d: &Domain, number: u64) -> Option<Answered> {
    d.application_answer(Token::new(number.checked_add(100).unwrap()))
}

#[test]
fn a_start_makes_the_row_and_the_fake_forge_in_one_decision() {
    for under_box in [false, true] {
        let mut d = playground(under_box);
        let sent = start(&mut d, 1);
        let record = ForgeRecord { number: 1, ..template() };
        let expected = [
            Request::Save { read_afresh: None, release: None, record: Stored::Forge(record.clone()) },
            Request::Register(record.clone()),
            Request::MarkedForges,
            Request::Save { read_afresh: None, release: None, record: Stored::ForgeNumbers { next: 2 } },
            save(running()),
        ];
        assert_eq!(&sent[..5], &expected);
        if under_box {
            assert_eq!(
                sent[5],
                Request::SetUpGroup { forge: 1, request: Token::new(1), entry: record.setup.clone().unwrap() }
            );
        }
        assert_eq!(sent.last(), Some(&Request::PlaygroundStart { request: Token::new(1) }));
        assert_eq!(d.playground(), Some(&running()));
        assert_eq!(d.records.get(&1).expect("kept forge").as_ref(), &record);
        assert_eq!(d.forges().rows[0].number, 1);
        assert!(!d.forge_ended(1));
        assert_eq!(answer(&d, 1), None);
        let project = Playground { project: Some(7), ..running() };
        assert_eq!(outputs(&mut d, Event::PlaygroundProject { project: 7 }).as_ref(), [save(project)]);
        let account = Playground { account: Some(9), ..project };
        assert_eq!(outputs(&mut d, Event::PlaygroundAccount { account: 9 }).as_ref(), [save(account)]);
        assert_eq!(d.playground(), Some(&account));
    }
}

#[test]
fn a_start_is_not_offered_beside_a_real_forge_or_while_resetting() {
    for case in 0_u8..5 {
        let mut d = playground(true);
        match case {
            0 => d.machine.playground = None,
            1 => {
                outputs(
                    &mut d,
                    Event::Restore { record: Stored::Forge(ForgeRecord { number: 2, ..ForgeRecord::github_com() }) },
                );
            }
            2..=4 => {
                let step = match case {
                    2 => ResetStep::Project,
                    3 => ResetStep::Forge,
                    _ => ResetStep::Account,
                };
                outputs(&mut d, Event::Restore { record: Stored::Playground(resetting(running(), step)) });
            }
            _ => unreachable!(),
        }
        assert_eq!(start(&mut d, 1).as_ref(), [done(1)]);
        assert_eq!(answer(&d, 1), Some(Answered::StartPlayground(StartAnswer::NotOffered)));
    }
    let mut d = playground(false);
    let mut removing = ForgeRecord { number: 2, ..ForgeRecord::github_com() };
    removing.state = ForgeState::Removing;
    outputs(&mut d, Event::Restore { record: Stored::Forge(removing) });
    assert!(start(&mut d, 1).contains(&Request::PlaygroundStart { request: Token::new(1) }));
}

#[test]
fn starts_share_one_answer() {
    let mut d = playground(true);
    start(&mut d, 1);
    for number in 2..=u64::from(LIMITS.applications) {
        assert!(start(&mut d, number).is_empty());
    }
    assert_eq!(
        start(&mut d, 20).as_ref(),
        [Request::ApplicationAgain { request: Token::new(20), why: jig_domain_people::Again::Busy }]
    );
    let expected = StartAnswer::Failed(StartFailure::Account);
    let sent = outputs(&mut d, Event::PlaygroundAnswered { answer: expected.clone() });
    assert_eq!(sent.len(), usize::try_from(LIMITS.applications).unwrap());
    for number in 1..=u64::from(LIMITS.applications) {
        assert!(sent.contains(&done(number)));
        assert_eq!(answer(&d, number), Some(Answered::StartPlayground(expected.clone())));
    }
    assert!(outputs(&mut d, Event::PlaygroundAnswered { answer: StartAnswer::Running }).is_empty());
    assert_eq!(answer(&d, 1), Some(Answered::StartPlayground(expected)));
    assert_eq!(
        start(&mut d, 30).as_ref(),
        [
            Request::SetUpGroup { forge: 1, request: Token::new(30), entry: template().setup.unwrap() },
            Request::PlaygroundStart { request: Token::new(30) },
        ]
    );
    assert_eq!(answer(&d, 1), None, "a new operation releases earlier answers as the demo's did");
}

fn with_numbers(d: &mut Domain) -> Playground {
    start(d, 1);
    outputs(d, Event::PlaygroundProject { project: 7 });
    outputs(d, Event::PlaygroundAccount { account: 9 });
    outputs(d, Event::PlaygroundAnswered { answer: StartAnswer::Running });
    *d.playground().unwrap()
}

#[test]
fn the_reset_removes_project_forge_and_account_in_order_then_the_row() {
    let mut d = playground(true);
    let row = with_numbers(&mut d);
    let project = ResetAct::RemoveProject { project: 7 };
    let account = ResetAct::RemoveAccount { account: 9 };
    assert_eq!(end(&mut d, 2).as_ref(), [save(resetting(row, ResetStep::Project)), Request::ResetAct(project)]);
    assert!(end(&mut d, 3).is_empty());
    assert!(outputs(&mut d, Event::ResetActed { act: project, keep: true }).is_empty());
    assert!(outputs(&mut d, Event::ResetActed { act: account, keep: false }).is_empty());
    assert!(outputs(&mut d, Event::ResetActed { act: ResetAct::RemoveProject { project: 8 }, keep: false }).is_empty());
    let forge = outputs(&mut d, Event::ResetActed { act: project, keep: false });
    assert_eq!(forge.first(), Some(&save(resetting(row, ResetStep::Forge))));
    assert!(forge.contains(&Request::Unregister { forge: 1 }));
    assert_eq!(d.records.get(&1).unwrap().state, ForgeState::Removing);
    assert!(d.forge_ended(1));
    assert!(d.forges().rows.is_empty());
    assert_eq!(d.playground(), Some(&resetting(row, ResetStep::Forge)));
    assert_eq!(answer(&d, 2), None);
    let group = outputs(&mut d, Event::GroupRemoved { forge: 1 });
    assert!(!group.contains(&Request::ResetAct(account)));
    assert!(d.records.contains_key(&1));
    let erased = outputs(&mut d, Event::SecretsErased { forge: 1 });
    let mut forge_end = None;
    let mut account_start = None;
    for (index, request) in erased.iter().enumerate() {
        if *request == (Request::Erase { key: Key::Forge(1) }) {
            forge_end = Some(index);
        }
        if *request == save(resetting(row, ResetStep::Account)) {
            account_start = Some(index);
        }
    }
    assert!(forge_end.unwrap() < account_start.unwrap());
    assert_eq!(erased.last(), Some(&Request::ResetAct(account)));
    assert!(!d.records.contains_key(&1));
    assert_eq!(answer(&d, 2), None);
    assert!(outputs(&mut d, Event::ResetActed { act: account, keep: true }).is_empty());
    assert_eq!(
        outputs(&mut d, Event::ResetActed { act: account, keep: false }).as_ref(),
        [Request::Erase { key: Key::Playground }, done(2), done(3),]
    );
    assert_eq!(d.playground(), None);
    for number in [2, 3] {
        assert_eq!(answer(&d, number), Some(Answered::EndPlayground(EndAnswer::Reset)));
    }
    assert!(outputs(&mut d, Event::ResetActed { act: account, keep: false }).is_empty());
}

#[test]
fn a_reset_without_project_or_account_skips_those_steps() {
    let mut d = playground(false);
    start(&mut d, 1);
    let sent = end(&mut d, 2);
    assert_eq!(
        &sent[..2],
        &[save(resetting(running(), ResetStep::Project)), save(resetting(running(), ResetStep::Forge))]
    );
    assert_eq!(answer(&d, 2), None);
    let erased = outputs(&mut d, Event::SecretsErased { forge: 1 });
    assert!(erased.contains(&Request::Erase { key: Key::Forge(1) }));
    assert!(erased.contains(&save(resetting(running(), ResetStep::Account))));
    assert_eq!(&erased[erased.len() - 2..], &[Request::Erase { key: Key::Playground }, done(2)]);
    assert_eq!(answer(&d, 2), Some(Answered::EndPlayground(EndAnswer::Reset)));
    assert_eq!(d.playground(), None);
    assert_eq!(end(&mut d, 3).as_ref(), [done(3)]);
    assert_eq!(answer(&d, 3), Some(Answered::EndPlayground(EndAnswer::NotOffered)));
}

fn restore(row: Playground, record: Option<ForgeRecord>) -> Domain {
    let mut d = playground(true);
    if let Some(record) = record {
        outputs(&mut d, Event::Restore { record: Stored::Forge(record) });
    }
    outputs(&mut d, Event::Restore { record: Stored::ForgeNumbers { next: 2 } });
    outputs(&mut d, Event::Restore { record: Stored::Playground(row) });
    assert_eq!(d.playground(), Some(&row));
    assert_eq!(d.restore_fit_failure(), None);
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    d
}

#[test]
fn each_step_is_resumed_at_start_from_the_row() {
    let row = Playground { project: Some(7), account: Some(9), ..running() };
    for step in [ResetStep::Project, ResetStep::Forge, ResetStep::Account] {
        let record = if step == ResetStep::Project { Some(ForgeRecord { number: 1, ..template() }) } else { None };
        let row = resetting(row, step);
        let mut d = restore(row, record);
        let first = outputs(&mut d, Event::SendKeptActs);
        let expected = if step == ResetStep::Project {
            ResetAct::RemoveProject { project: 7 }
        } else {
            ResetAct::RemoveAccount { account: 9 }
        };
        assert_eq!(first.first(), Some(&save(row)));
        assert_eq!(first.last(), Some(&Request::ResetAct(expected)));
        let again = outputs(&mut d, Event::SendKeptActs);
        assert_eq!(again.last(), Some(&Request::ResetAct(expected)));
        assert_eq!(answer(&d, 1), None);
    }
    let row = resetting(row, ResetStep::Forge);
    let mut d = restore(row, Some(ForgeRecord { number: 1, ..template() }));
    let first = outputs(&mut d, Event::SendKeptActs);
    assert!(first.contains(&Request::Unregister { forge: 1 }));
    assert_eq!(d.playground(), Some(&row));
    let mut restored = restore(row, Some(d.records.get(&1).expect("removing forge").as_ref().clone()));
    for request in first {
        if let Request::Save { record: Stored::KeptAct(act), .. } = request {
            outputs(&mut restored, Event::Restore { record: Stored::KeptAct(act) });
        }
    }
    let sent = outputs(&mut restored, Event::SendKeptActs);
    let mut count = 0_u32;
    for request in sent {
        if let Request::SendKeptAct(_) = request {
            count += 1;
        }
    }
    assert_eq!(count, 2, "the in-flight forge removal's acts are replayed");
    outputs(&mut restored, Event::GroupRemoved { forge: 1 });
    let sent = outputs(&mut restored, Event::SecretsErased { forge: 1 });
    assert_eq!(sent.last(), Some(&Request::ResetAct(ResetAct::RemoveAccount { account: 9 })));
}

#[test]
fn a_start_that_finds_the_playground_running_resets_it() {
    let row = Playground { project: Some(7), account: Some(9), ..running() };
    let mut d = restore(row, Some(ForgeRecord { number: 1, ..template() }));
    assert_eq!(
        outputs(&mut d, Event::SendKeptActs).as_ref(),
        [save(resetting(row, ResetStep::Project)), Request::ResetAct(ResetAct::RemoveProject { project: 7 }),]
    );
    assert_eq!(d.playground(), Some(&resetting(row, ResetStep::Project)));
    assert_eq!(start(&mut d, 1).as_ref(), [done(1)]);
    assert_eq!(answer(&d, 1), Some(Answered::StartPlayground(StartAnswer::NotOffered)));
}

#[test]
fn another_forge_connecting_no_longer_ends_the_playground() {
    let mut d = playground(true);
    start(&mut d, 1);
    outputs(&mut d, Event::Restore { record: Stored::Forge(ForgeRecord { number: 2, ..ForgeRecord::github_com() }) });
    outputs(
        &mut d,
        Event::Connection { forge: 2, state: Connection::Connected { id: 7, login: b"temper".as_slice().into() } },
    );
    assert_eq!(d.playground(), Some(&running()));
    assert_eq!(d.records.get(&1).unwrap().state, ForgeState::Kept);
    assert!(!d.forge_ended(1));
    assert_eq!(d.forges().rows.len(), 2);
    assert_eq!(answer(&d, 1), None, "the start is still waiting for the root's answer");
    assert_eq!(outputs(&mut d, Event::PlaygroundAnswered { answer: StartAnswer::Running }).as_ref(), [done(1)]);
}

#[test]
fn the_playground_uses_the_next_record_number_and_exhaustion_is_not_offered() {
    let mut d = playground(true);
    outputs(&mut d, Event::Restore { record: Stored::ForgeNumbers { next: 3 } });
    let sent = start(&mut d, 1);
    assert_eq!(d.playground().unwrap().forge, 3);
    assert_eq!(d.next_number, 4);
    assert!(sent.contains(&Request::Register(ForgeRecord { number: 3, ..template() })));
    for case in 0_u8..3 {
        let mut d = playground(false);
        match case {
            0 => d.next_number = 0,
            1 => {
                for number in 1..=u16::try_from(LIMITS.forges).unwrap() {
                    outputs(
                        &mut d,
                        Event::Restore {
                            record: Stored::Forge(ForgeRecord {
                                number,
                                state: ForgeState::Removing,
                                ..ForgeRecord::github_com()
                            }),
                        },
                    );
                }
            }
            _ => {
                let mut record = configured()[0].clone();
                record.number = u16::MAX;
                d = setup_domain(&[record], false);
                d.machine.playground = Some(template());
            }
        }
        assert_eq!(start(&mut d, 1).as_ref(), [done(1)]);
        assert_eq!(answer(&d, 1), Some(Answered::StartPlayground(StartAnswer::NotOffered)));
        assert_eq!(d.playground(), None);
    }
}

#[test]
fn end_waiters_are_bounded_and_answer_only_after_the_row_is_erased() {
    let row = Playground { account: Some(9), ..running() };
    let mut d = restore(row, None);
    let sent = end(&mut d, 1);
    assert_eq!(sent.last(), Some(&Request::ResetAct(ResetAct::RemoveAccount { account: 9 })));
    for number in 2..=u64::from(LIMITS.applications) {
        assert!(end(&mut d, number).is_empty());
    }
    assert_eq!(
        end(&mut d, 20).as_ref(),
        [Request::ApplicationAgain { request: Token::new(20), why: jig_domain_people::Again::Busy }]
    );
    let sent = outputs(&mut d, Event::ResetActed { act: ResetAct::RemoveAccount { account: 9 }, keep: false });
    assert_eq!(sent.len(), usize::try_from(LIMITS.applications).unwrap() + 1);
    assert_eq!(sent.first(), Some(&Request::Erase { key: Key::Playground }));
    for number in 1..=u64::from(LIMITS.applications) {
        assert!(sent.contains(&done(number)));
        assert_eq!(answer(&d, number), Some(Answered::EndPlayground(EndAnswer::Reset)));
    }
}

#[test]
fn a_running_start_asks_only_a_dormant_group_under_the_box() {
    for under_box in [false, true] {
        for connection in [
            Connection::NotConnected,
            Connection::NotSetUp,
            Connection::SettingUp { step: None },
            Connection::SetUpFailed { why: b"failed".as_slice().into() },
            Connection::Stopped { reason: b"stopped".as_slice().into() },
            Connection::Connecting,
            Connection::Connected { id: 7, login: b"temper".as_slice().into() },
        ] {
            let mut d = playground(under_box);
            start(&mut d, 1);
            outputs(&mut d, Event::PlaygroundAnswered { answer: StartAnswer::Running });
            outputs(&mut d, Event::Connection { forge: 1, state: connection.clone() });
            let sent = start(&mut d, 2);
            let needs_group = under_box
                && match connection {
                    Connection::Connecting | Connection::Connected { .. } => false,
                    Connection::NotConnected
                    | Connection::NotSetUp
                    | Connection::SettingUp { .. }
                    | Connection::SetUpFailed { .. }
                    | Connection::Stopped { .. } => true,
                };
            assert_eq!(sent.len(), if needs_group { 2 } else { 1 });
            if needs_group {
                assert_eq!(
                    sent.first(),
                    Some(&Request::SetUpGroup { forge: 1, request: Token::new(2), entry: template().setup.unwrap() })
                );
            }
            assert_eq!(sent.last(), Some(&Request::PlaygroundStart { request: Token::new(2) }));
        }
    }
}

#[test]
fn a_resumed_removal_with_no_acts_begins_the_account_step_once() {
    let row = resetting(Playground { project: Some(7), account: Some(9), ..running() }, ResetStep::Forge);
    let mut d = restore(row, Some(ForgeRecord { number: 1, state: ForgeState::Removing, ..template() }));
    let sent = outputs(&mut d, Event::SendKeptActs);
    assert_eq!(
        sent.as_ref(),
        [
            Request::Erase { key: Key::Writer(1) },
            Request::Erase { key: Key::Forge(1) },
            save(resetting(row, ResetStep::Account)),
            Request::ResetAct(ResetAct::RemoveAccount { account: 9 }),
        ]
    );
    assert_eq!(d.playground(), Some(&resetting(row, ResetStep::Account)));
}
