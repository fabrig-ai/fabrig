//! Session trial facts (temper/docs/design/domain/trial.md, 6).
use super::fixtures as f;
use crate::{
    Domain, Event, Request,
    forge::{self, trials},
};
use jig_client::{Ask, Down, Family, Link, Told, Up, Watchable, contract as c};
use skein_lib::{List, Queue, Token, Writer};
use temper_client_protocol_wire::v1::Record;
use temper_client_protocol_wire::{self as wire, v1 as w};

const ACCOUNT: u32 = 17;
const OTHER: u32 = 23;
const TASK: u64 = f::chat::TASK;
const NEXT: u64 = 31;
const WAIT: c::TaskRunWait = c::TaskRunWait::Credential;
// The session fixture delivers two account rows.
const SESSION_ROWS: usize = 2;

fn event(root: &mut Domain, event: Event) -> Queue<Request> {
    let mut out = Queue::uncapped();
    crate::step(root, &f::env(&f::limits()), event, &mut out);
    out
}
fn up(root: &mut Domain, up: Up) -> Queue<Request> {
    event(root, Event::Client { event: jig_client::Event::Up { up } })
}
fn opening(out: &Queue<Request>, expected: Watchable) -> Option<Token> {
    let mut found = None;
    for request in out {
        if let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: Down::Open { watch, subject } } } =
            request
            && *subject == expected
        {
            assert!(found.is_none(), "one watch per subject");
            found = Some(*watch);
        }
    }
    found
}
fn opened() -> (Domain, Token, Token) {
    let mut root = crate::unlinked(&f::limits(), f::SEED);
    let frame = f::start(&mut root, List::with_capacity(0));
    let mut out = Queue::uncapped();
    f::confirm(&mut root, frame, f::PARTY, &mut out);
    let watch = opening(&out, trials::watch(&f::limits().forge).unwrap()).expect("trial watch");
    up(&mut root, Up::Watched { watch, event: c::WatchEvent::Opened });
    (root, watch, frame)
}
fn row(account: u32, task: Option<u64>, state: w::TrialState) -> w::TrialRow {
    w::TrialRow { run_wait: None, account, task, took: Some(OTHER), step: w::TrialStep::Answers, state }
}
fn running(account: u32, task: u64) -> w::TrialRow {
    row(account, Some(task), w::TrialState::Running)
}
fn ended() -> w::TrialRow {
    row(ACCOUNT, Some(TASK), w::TrialState::Failed(w::TrialFailed { why: w::TrialWhy::Waited }))
}
fn deliver(root: &mut Domain, watch: Token, rows: &[w::TrialRow]) -> Queue<Request> {
    let rows = List::from_boxed(rows.into());
    let document = w::Trials { rows };
    let mut writer = Writer::new(
        usize::try_from(document.measure(&temper_client_protocol_wire::v1::CEILINGS).expect("within ceilings"))
            .unwrap(),
    );
    document.write(&temper_client_protocol_wire::v1::CEILINGS, &mut writer).unwrap();
    up(
        root,
        Up::FamilyDelivered {
            watch,
            record: Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: writer.finish() },
        },
    )
}
fn again(root: &mut Domain, account: u32) -> Queue<Request> {
    f::form_act(root, forge::Act::TryAccount { account, again: true })
}
fn sent(out: &mut Queue<Request>) -> Option<skein_lib::RequestKey> {
    let mut found = None;
    while let Some(request) = out.pop() {
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down { down: Down::Send { request: Ask::Family { scope, record }, key, .. } },
        } = request
            && let w::Request::TryAccount(value) = wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().request
        {
            assert!(found.is_none(), "one send");
            assert_eq!(scope, c::Scope::Own);
            assert_eq!(value.account, ACCOUNT);
            assert!(value.again);
            found = Some(key);
        }
    }
    found
}
fn answer(outcome: w::Outcome) -> Family {
    let answer = w::Answer { outcome };
    let mut writer = Writer::new(
        usize::try_from(answer.measure(&temper_client_protocol_wire::v1::CEILINGS).expect("within ceilings")).unwrap(),
    );
    answer.write(&temper_client_protocol_wire::v1::CEILINGS, &mut writer).unwrap();
    Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: writer.finish() }
}
fn link(root: &mut Domain, link: Link) -> Queue<Request> {
    let mut out = Queue::uncapped();
    crate::route::told(root, &f::env(&f::limits()), Told::Link { link }, &mut out);
    out
}

fn trial_pages() -> [crate::Address; 3] {
    [
        crate::Address::Welcome { step: crate::Step::Model { way: None } },
        crate::Address::Welcome { step: crate::Step::Ready },
        crate::Address::ConnectModel { way: None },
    ]
}
fn account_session(address: crate::Address) -> (Domain, Token, Token) {
    let mut root = crate::unlinked(&f::limits(), f::SEED);
    let me = f::start(&mut root, List::with_capacity(0));
    let mut out = Queue::uncapped();
    f::confirm(&mut root, me, f::PARTY, &mut out);
    let accounts = f::accounts_watch(&out);
    let trials = opening(&out, trials::watch(&f::limits().forge).unwrap()).expect("trial watch");
    up(&mut root, Up::Watched { watch: trials, event: c::WatchEvent::Opened });
    event(&mut root, Event::Went { place: crate::Place::Address(address) });
    (root, accounts, trials)
}
fn account(root: &mut Domain, watch: Token, usable: bool) -> Queue<Request> {
    let mut document = super::playground::accounts(Some(17), usable, false);
    document.accounts.get_mut(0).unwrap().number = ACCOUNT;
    up(root, Up::Watched { watch, event: c::WatchEvent::Delivered(c::Delivery::Accounts(Box::new(document))) })
}
fn tries(out: Queue<Request>) -> List<(u32, bool)> {
    let mut found = List::uncapped();
    for request in &out {
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down { down: Down::Send { request: Ask::Family { scope, record }, .. } },
        } = request
            && let w::Request::TryAccount(value) = wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().request
        {
            assert_eq!(*scope, c::Scope::Own);
            found.push((value.account, value.again)).unwrap();
        }
    }
    found
}

#[test]
fn an_account_newly_usable_on_the_welcome_is_tried() {
    for address in trial_pages() {
        let (mut root, watch, _) = account_session(address);
        assert!(tries(account(&mut root, watch, false)).is_empty());
        assert_eq!(tries(account(&mut root, watch, true)).as_slice(), [(ACCOUNT, false)]);
        assert_eq!(root.forge().trial_asked(), Some(ACCOUNT));
        assert!(tries(account(&mut root, watch, true)).is_empty());
        assert!(tries(account(&mut root, watch, false)).is_empty());
        assert!(tries(account(&mut root, watch, true)).is_empty(), "an ask is already out");
    }
}

#[test]
fn the_first_delivery_tries_nothing() {
    for address in trial_pages() {
        let (mut root, watch, _) = account_session(address);
        assert!(tries(account(&mut root, watch, true)).is_empty());
        assert_eq!(root.forge().trial_asked(), None);
        assert!(tries(account(&mut root, watch, true)).is_empty());
        assert_eq!(root.forge().trial_asked(), None);
    }
}

#[test]
fn elsewhere_tries_nothing() {
    use crate::{Address, Step};
    for address in [
        Address::Welcome { step: Step::WelcomeAsked },
        Address::Welcome { step: Step::Forge { forge: None } },
        Address::Accounts,
        Address::Forges,
        Address::ConnectForge { forge: None },
        Address::Inbox { filter: crate::address::Filter::ALL },
        Address::Chats { filter: crate::ChatsFilter::All },
        Address::NewChat { draft: false },
        Address::Chat { task: TASK },
        Address::Task { task: TASK },
        Address::Home { project: f::PROJECT },
        Address::Goals { project: f::PROJECT, filter: crate::GoalsFilter::All },
        Address::Changes { project: f::PROJECT },
        Address::Settings { project: f::PROJECT },
        Address::Repositories { project: f::PROJECT },
        Address::Builders { project: f::PROJECT },
        Address::Ask { project: f::PROJECT },
        Address::Adopt { project: f::PROJECT },
    ] {
        let (mut root, watch, _) = account_session(address.clone());
        account(&mut root, watch, false);
        assert!(tries(account(&mut root, watch, true)).is_empty(), "{address:?}");
        assert_eq!(root.forge().trial_asked(), None);
        event(&mut root, Event::Went { place: crate::Place::Address(Address::Welcome { step: Step::Ready }) });
        assert!(tries(account(&mut root, watch, true)).is_empty(), "seen advances on {address:?}");
    }
}

#[test]
fn the_playground_is_never_tried() {
    for address in trial_pages() {
        let (mut root, watch, _) = account_session(address);
        super::playground::deliver_playground_read(
            &mut root,
            super::playground::page_with_state(w::PlaygroundState::Running),
        );
        account(&mut root, watch, false);
        assert!(tries(account(&mut root, watch, true)).is_empty());
        assert_eq!(root.forge().trial_asked(), None);
    }
}

#[test]
fn an_account_with_a_row_is_not_asked_again() {
    for address in trial_pages() {
        let (mut root, watch, trial_watch) = account_session(address);
        account(&mut root, watch, false);
        deliver(&mut root, trial_watch, &[ended()]);
        assert!(tries(account(&mut root, watch, true)).is_empty());
        assert_eq!(root.forge().trial_asked(), None);
    }
}

#[test]
fn the_rows_arrive_for_the_session() {
    let mut root = crate::unlinked(&f::limits(), f::SEED);
    f::start(&mut root, List::with_capacity(0));
    let mut child = Queue::uncapped();
    trials::configure(&mut root.forge, f::limits().forge, &mut child);
    assert!(child.is_empty(), "no trial watch while starting");
    let (mut root, watch, frame) = opened();
    let Watchable::Family { scope, record, .. } = trials::watch(&f::limits().forge).unwrap() else {
        panic!("family watch")
    };
    assert_eq!(scope, c::Scope::Own);
    assert_eq!(&wire::decode_watching(&record.bytes, &w::CEILINGS).unwrap().watch, &w::Watch::Trials);
    let mut out = Queue::uncapped();
    f::confirm(&mut root, frame, f::PARTY, &mut out);
    assert_eq!(opening(&out, trials::watch(&f::limits().forge).unwrap()), None);
    deliver(&mut root, watch, &[ended(), row(OTHER, None, w::TrialState::Running)]);
    assert_eq!(
        root.forge().trials(),
        &[
            trials::Row {
                run_wait: None,
                account: ACCOUNT,
                task: Some(TASK),
                took: Some(OTHER),
                step: trials::Step::Answers,
                state: trials::State::Failed { why: trials::Why::Waited }
            },
            trials::Row {
                run_wait: None,
                account: OTHER,
                task: None,
                took: Some(OTHER),
                step: trials::Step::Answers,
                state: trials::State::Running
            },
        ]
    );
    event(&mut root, Event::Went { place: crate::Place::Missing });
    assert_eq!(root.forge().trials().len(), SESSION_ROWS);
    up(&mut root, Up::SignedOut);
    assert_eq!((root.forge().trials()).len(), 0);
    let mut out = up(&mut root, Up::SignedIn);
    let frame = Token::new(0);
    f::confirm(&mut root, frame, f::PARTY, &mut out);
    assert!(opening(&out, trials::watch(&f::limits().forge).unwrap()).is_some());
}

#[test]
fn try_again_sends_once() {
    let (mut root, watch, _) = opened();
    let mut out = again(&mut root, ACCOUNT);
    assert!(sent(&mut out).is_some());
    assert_eq!(root.forge().trial_asked(), Some(ACCOUNT));
    assert!(sent(&mut again(&mut root, ACCOUNT)).is_none());
    assert!(sent(&mut again(&mut root, OTHER)).is_none());
    deliver(&mut root, watch, &[row(OTHER, None, w::TrialState::Running)]);
    assert_eq!(root.forge().trial_asked(), Some(ACCOUNT));
    deliver(&mut root, watch, &[running(ACCOUNT, TASK)]);
    assert_eq!(root.forge().trial_asked(), None);
    for (outcome, expected) in [
        (jig_client::Outcome::Answered(jig_client::Answered::Family(answer(w::Outcome::Tried))), Some(ACCOUNT)),
        (jig_client::Outcome::Unknown, None),
        (jig_client::Outcome::Answered(jig_client::Answered::Family(answer(w::Outcome::Unknown))), None),
        (jig_client::Outcome::Answered(jig_client::Answered::Family(answer(w::Outcome::NotTheirs))), None),
        (jig_client::Outcome::Limit, None),
    ] {
        let mut out = Queue::uncapped();
        forge::step(
            &mut root.forge,
            &skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits: f::limits().forge },
            forge::Event::Act { act: forge::Act::TryAccount { account: ACCOUNT, again: true } },
            &mut out,
        );
        let Some(forge::Request::Send { request, .. }) = out.pop() else { panic!("trial ask") };
        let key = jig_client::Key { asker: f::FORGE_ASKER, number: request.raw() };
        crate::route::told(&mut root, &f::env(&f::limits()), Told::Answered { key, outcome }, &mut Queue::uncapped());
        assert_eq!(root.forge().trial_asked(), expected);
        deliver(&mut root, watch, &[running(ACCOUNT, TASK)]);
    }
}

#[test]
fn a_running_rows_wait_arrives_without_a_task_watch() {
    use jig_client_protocol_wire::TaskRunWait as J;
    let (mut root, watch, _) = opened();
    for (value, expected) in
        [(Some(J::Credential), Some(WAIT)), (Some(J::Host), Some(c::TaskRunWait::Host)), (None, None)]
    {
        let out = deliver(&mut root, watch, &[waiting(running(ACCOUNT, TASK), value)]);
        assert!(opening(&out, Watchable::Jig(c::Subject::Task { task: TASK })).is_none());
        assert_eq!(root.forge().trial_wait(), expected.as_ref());
    }
    deliver(&mut root, watch, &[waiting(running(OTHER, NEXT), Some(J::Host)), running(ACCOUNT, TASK)]);
    assert_eq!(root.forge().trial_wait(), Some(&c::TaskRunWait::Host));
    deliver(&mut root, watch, &[ended()]);
    assert_eq!(root.forge().trial_wait(), Some(&c::TaskRunWait::Host));
    again(&mut root, ACCOUNT);
    assert_eq!(root.forge().trial_wait(), None);
}

#[test]
fn malformed_or_inapplicable_row_waits_are_no_wait() {
    use jig_client_protocol_wire::TaskRunWait as J;
    let (mut root, watch, _) = opened();
    let valid = waiting(running(ACCOUNT, TASK), Some(J::Credential));
    let mut trailing = valid.clone();
    trailing.run_wait = Some(skein_lib::bytes::concat(&[&trailing.run_wait.take().unwrap(), &[0]]));
    let mut malformed = valid.clone();
    malformed.run_wait = Some(b"invalid".as_slice().into());
    for row in [
        trailing,
        malformed,
        waiting(row(ACCOUNT, None, w::TrialState::Running), Some(J::Credential)),
        waiting(ended(), Some(J::Credential)),
    ] {
        deliver(&mut root, watch, core::slice::from_ref(&valid));
        deliver(&mut root, watch, &[row]);
        assert_eq!(root.forge().trials().first().unwrap().run_wait, None);
        if root.forge().trials().first().unwrap().state == trials::State::Running {
            assert_eq!(root.forge().trial_wait(), None);
        }
    }
}

#[test]
fn a_timed_out_trial_keeps_its_last_wait_after_task_ends() {
    let (mut root, watch, _) = opened();
    deliver(
        &mut root,
        watch,
        &[waiting(running(ACCOUNT, TASK), Some(jig_client_protocol_wire::TaskRunWait::Credential))],
    );
    deliver(&mut root, watch, &[running(ACCOUNT, TASK)]);
    deliver(&mut root, watch, &[ended()]);
    assert_eq!(root.forge().trial_wait(), Some(&WAIT), "the timed-out row keeps the reason seen before cancellation");
    deliver(&mut root, watch, &[running(ACCOUNT, NEXT)]);
    assert_eq!(root.forge().trial_wait(), None, "a different running task forgets the wait");
    deliver(&mut root, watch, &[ended()]);
    assert_eq!(root.forge().trial_wait(), None);
    again(&mut root, ACCOUNT);
    assert_eq!(root.forge().trial_wait(), None, "a new try forgets the previous wait");
}

fn waiting(row: w::TrialRow, wait: Option<jig_client_protocol_wire::TaskRunWait>) -> w::TrialRow {
    let mut row = row;
    row.run_wait = match wait {
        Some(wait) => {
            let mut writer = Writer::new(
                usize::try_from(wait.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")).unwrap(),
            );
            wait.write(&jig_client_protocol_wire::CEILINGS, &mut writer).unwrap();
            Some(writer.finish())
        }
        None => None,
    };
    row
}

#[test]
fn an_ended_watch_stays_ended() {
    for end in [c::WatchEnd::NotAllowed, c::WatchEnd::SubjectEnded] {
        let (mut root, watch, frame) = opened();
        deliver(&mut root, watch, &[ended()]);
        up(&mut root, Up::WatchEnded { watch, end });
        assert_eq!((root.forge().trials()).len(), 0);
        let mut out = Queue::uncapped();
        f::confirm(&mut root, frame, f::PARTY, &mut out);
        assert_eq!(opening(&out, trials::watch(&f::limits().forge).unwrap()), None);
        assert_eq!(opening(&link(&mut root, Link::Offline), trials::watch(&f::limits().forge).unwrap()), None);
        assert_eq!(opening(&link(&mut root, Link::Live), trials::watch(&f::limits().forge).unwrap()), None);
        deliver(&mut root, watch, &[ended()]);
        assert!(root.forge().trials().is_empty(), "late rows stay ended");
    }
}
