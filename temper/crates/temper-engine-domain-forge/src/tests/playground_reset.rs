//! Reset decisions (temper/docs/design/draft/playground-reset.md, 6.2).
use super::*;

const FORGE: u16 = 1;
const PROJECT: u32 = 2;
const ACCOUNT: u32 = 3;

fn row(state: PlaygroundState) -> Playground {
    Playground { state, forge: Some(FORGE), project: Some(PROJECT), account: Some(ACCOUNT) }
}
fn configured(state: PlaygroundState) -> Domain {
    let mut d = domain();
    d.under_box = true;
    d.configure_demo(Some(DemoConfig {
        forge: None,
        provider: 1,
        adoption: fixtures::adoption(),
        deadline: Duration::from_secs(10),
    }));
    let mut forge = records::forge();
    forge.number = FORGE;
    forge.playground = true;
    forge.setup = Some(record::Setup {
        directory: b"playground".as_slice().into(),
        bot: record::Account { login: b"temper".as_slice().into(), email: Box::new([]) },
        orgs: Box::new([]),
        asked: true,
        mode: record::SetupMode::Fake(record::FakeEntry {
            listen: ([127, 0, 0, 1], 3001).into(),
            git: b"fake".as_slice().into(),
            owner: b"playground".as_slice().into(),
            repositories: Box::new([]),
        }),
    });
    outputs(&mut d, Event::Restore { record: Stored::Forge(forge) });
    outputs(&mut d, Event::Restore { record: Stored::Playground(row(state)) });
    d
}
fn apply(d: &mut Domain, ask: Ask, number: u64) -> Box<[Request]> {
    outputs(
        d,
        Event::Application {
            role: None,
            carried: None,
            request: Token::new(number),
            token: Token::new(number.checked_add(100).expect("fixture token")),
            retention: Duration::from_secs(60),
            ask,
        },
    )
}
fn step(number: u64, act: KeptAct) -> Kept {
    Kept { key: crate::kept::key(b"playground reset", &[number]), act }
}
fn saved(row: Playground) -> Request {
    Request::Save { record: Stored::Playground(row), read_afresh: None, release: None }
}
fn saved_step(row: Kept) -> Request {
    Request::Save { record: Stored::Kept(row), read_afresh: None, release: None }
}
fn answer(d: &Domain, number: u64) -> Option<Answered> {
    d.application_answer(Token::new(number.checked_add(100).expect("fixture token")))
}
fn erase(d: &mut Domain, number: u64) -> Box<[Request]> {
    outputs(d, Event::KeptAnswered { key: crate::kept::key(b"playground reset", &[number]), keep: false })
}
#[test]
fn end_with_no_row_is_reset_at_once() {
    let mut d = configured(PlaygroundState::Running);
    d.playground = None;
    assert_eq!(
        apply(&mut d, Ask::EndPlayground, 1).as_ref(),
        [Request::ApplicationDone { request: Token::new(1), token: Token::new(101) }]
    );
    assert_eq!(answer(&d, 1), Some(Answered::EndPlayground(EndAnswer::Reset)));
}
#[test]
fn end_without_config_is_not_offered() {
    let mut d = domain();
    apply(&mut d, Ask::EndPlayground, 1);
    assert_eq!(answer(&d, 1), Some(Answered::EndPlayground(EndAnswer::NotOffered)));
}
#[test]
fn end_keeps_the_project_first_and_reads_resetting() {
    let mut d = configured(PlaygroundState::Running);
    let first = step(1, KeptAct::RemoveProject { project: PROJECT });
    assert_eq!(
        apply(&mut d, Ask::EndPlayground, 1).as_ref(),
        [saved(row(PlaygroundState::Resetting)), saved_step(first), Request::SendKept(first)]
    );
    assert_eq!(d.playground(), Some(&row(PlaygroundState::Resetting)));
    assert_eq!(answer(&d, 1), None);
}
#[test]
fn each_erased_step_keeps_the_next_in_order() {
    let mut d = configured(PlaygroundState::Running);
    apply(&mut d, Ask::EndPlayground, 1);
    let second = step(2, KeptAct::RemoveForge { forge: FORGE });
    let third = step(3, KeptAct::RemoveAccount { account: ACCOUNT });
    let sent = erase(&mut d, 1);
    assert!(sent.contains(&saved(Playground { project: None, ..row(PlaygroundState::Resetting) })));
    assert!(sent.contains(&saved_step(second)));
    assert!(d.forge_record(FORGE).unwrap().removing);
    assert_eq!(d.kept.get(&second.key), Some(&second.act));
    let sent = outputs(&mut d, Event::BoxRemoved { forge: FORGE, removed: true });
    assert!(sent.contains(&Request::Erase { key: Key::Kept(second.key) }));
    assert!(sent.contains(&saved(Playground { project: None, forge: None, ..row(PlaygroundState::Resetting) })));
    assert!(sent.contains(&saved_step(third)));
    assert!(sent.contains(&Request::SendKept(third)));
    let sent = erase(&mut d, 3);
    assert!(sent.contains(&Request::Erase { key: Key::Playground }));
    assert_eq!(d.playground(), None);
    assert!(d.kept.is_empty());
    assert_eq!(answer(&d, 1), Some(Answered::EndPlayground(EndAnswer::Reset)));
}
#[test]
fn a_step_with_no_field_is_skipped() {
    let mut d = configured(PlaygroundState::Running);
    d.playground.as_mut().unwrap().project = None;
    d.playground.as_mut().unwrap().forge = None;
    let last = step(3, KeptAct::RemoveAccount { account: ACCOUNT });
    let sent = apply(&mut d, Ask::EndPlayground, 1);
    assert!(sent.contains(&saved_step(last)));
    assert_eq!(d.kept.len(), 1);
    erase(&mut d, 3);
    assert_eq!(answer(&d, 1), Some(Answered::EndPlayground(EndAnswer::Reset)));
}
#[test]
fn a_second_end_waits_for_the_first() {
    let mut d = configured(PlaygroundState::Running);
    d.playground.as_mut().unwrap().forge = None;
    d.playground.as_mut().unwrap().account = None;
    apply(&mut d, Ask::EndPlayground, 1);
    assert!(apply(&mut d, Ask::EndPlayground, 2).is_empty());
    assert_eq!(answer(&d, 2), None);
    erase(&mut d, 1);
    for number in [1, 2] {
        assert_eq!(answer(&d, number), Some(Answered::EndPlayground(EndAnswer::Reset)));
    }
}
#[test]
fn a_running_playground_found_at_a_start_is_reset() {
    let mut d = configured(PlaygroundState::Running);
    let sent = outputs(&mut d, Event::SendKept);
    assert!(sent.contains(&saved(row(PlaygroundState::Resetting))));
    assert!(sent.contains(&saved_step(step(1, KeptAct::RemoveProject { project: PROJECT }))));
}
#[test]
fn a_resetting_playground_found_at_a_start_goes_on() {
    let mut d = configured(PlaygroundState::Resetting);
    d.playground.as_mut().unwrap().project = None;
    d.forge_records.get_mut(&FORGE).unwrap().removing = true;
    let second = step(2, KeptAct::RemoveForge { forge: FORGE });
    let box_remove =
        Kept { key: crate::kept::key(b"box", &[u64::from(FORGE)]), act: KeptAct::BoxRemove { forge: FORGE } };
    for kept in [second, box_remove] {
        outputs(&mut d, Event::Restore { record: Stored::Kept(kept) });
    }
    assert_eq!(outputs(&mut d, Event::SendKept).as_ref(), [Request::SendKept(box_remove)]);
    assert_eq!(d.kept.get(&second.key), Some(&second.act));
}
#[test]
fn a_resetting_row_without_a_kept_step_keeps_the_first_undone_step() {
    let mut d = configured(PlaygroundState::Resetting);
    let first = step(1, KeptAct::RemoveProject { project: PROJECT });
    assert_eq!(outputs(&mut d, Event::SendKept).as_ref(), [saved_step(first), Request::SendKept(first)]);
}
#[test]
fn a_start_during_the_reset_is_not_offered() {
    let mut d = configured(PlaygroundState::Running);
    apply(&mut d, Ask::EndPlayground, 1);
    apply(&mut d, Ask::StartPlayground, 2);
    assert_eq!(answer(&d, 2), Some(Answered::StartPlayground(StartAnswer::NotOffered)));
}
#[test]
fn reset_answers_pending_starts_not_offered() {
    let mut d = configured(PlaygroundState::Running);
    apply(&mut d, Ask::StartPlayground, 1);
    let sent = apply(&mut d, Ask::EndPlayground, 2);
    assert!(sent.contains(&Request::ApplicationDone { request: Token::new(1), token: Token::new(101) }));
    assert_eq!(answer(&d, 1), Some(Answered::StartPlayground(StartAnswer::NotOffered)));
    assert!(outputs(&mut d, Event::PlaygroundMade { project: Some(9), account: None }).is_empty());
}

#[test]
fn a_missing_forge_finishes_its_step_in_the_same_decision() {
    let mut d = configured(PlaygroundState::Running);
    d.playground.as_mut().unwrap().project = None;
    d.forge_records.remove(&FORGE);
    let second = step(2, KeptAct::RemoveForge { forge: FORGE });
    let third = step(3, KeptAct::RemoveAccount { account: ACCOUNT });
    let sent = apply(&mut d, Ask::EndPlayground, 1);
    assert!(sent.contains(&saved_step(second)));
    assert!(sent.contains(&Request::Erase { key: Key::Kept(second.key) }));
    assert!(sent.contains(&saved_step(third)));
    assert!(sent.contains(&Request::SendKept(third)));
    assert_eq!(d.kept.len(), 1);
}
#[test]
fn an_empty_playground_is_erased_without_keeping_a_step() {
    let mut d = configured(PlaygroundState::Running);
    d.playground = Some(Playground { state: PlaygroundState::Running, forge: None, project: None, account: None });
    let sent = apply(&mut d, Ask::EndPlayground, 1);
    assert!(sent.contains(&Request::Erase { key: Key::Playground }));
    assert_eq!(answer(&d, 1), Some(Answered::EndPlayground(EndAnswer::Reset)));
    assert!(d.kept.is_empty());
}
