//! Whole-forge removal decisions and restart (draft playground-reset.md, 4).
use super::*;
use crate::removal::kept_key;
use skein_forge::api::Error;

const FAILED: skein_forge_client::Outcome = skein_forge_client::Outcome::Failed(Error::NotConnected);

fn record(number: u16) -> ForgeRecord {
    ForgeRecord { number, setup: Some(b"{}".as_slice().into()), ..ForgeRecord::github_com() }
}

fn restored(under_box: bool, row: ForgeRecord) -> Domain {
    let mut d = setup_domain(&[], under_box);
    outputs(&mut d, Event::Restore { record: Stored::ForgeNumbers { next: 3 } });
    outputs(&mut d, Event::Restore { record: Stored::Forge(row) });
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    d
}

fn application(d: &mut Domain, number: u64, ask: Ask) -> Box<[Request]> {
    outputs(
        d,
        Event::Application {
            request: Token::new(number),
            token: Token::new(number),
            retention: Duration::from_secs(60),
            ask,
            role: Some(jig_domain_people::Role::Owner),
            carried: None,
        },
    )
}

fn saved(record: Stored) -> Request {
    Request::Save { read_afresh: None, release: None, record }
}

fn act(forge: u16, kind: KeptActKind) -> KeptAct {
    let (number, purpose) = match kind {
        KeptActKind::Cancel { task, .. } => (task, b"cancel".as_slice()),
        KeptActKind::RemoveGroup => (0, b"group".as_slice()),
        KeptActKind::EraseSecrets => (0, b"secrets".as_slice()),
    };
    KeptAct { key: kept_key(forge, number, purpose), forge, act: kind }
}

fn sent_acts(out: &[Request]) -> Box<[KeptAct]> {
    let mut acts = skein_lib::List::uncapped();
    for request in out {
        if let Request::SendKeptAct(row) = request {
            acts.push(*row).unwrap();
        }
    }
    acts.into_boxed()
}

#[test]
fn removing_a_forge_ends_it_and_keeps_its_acts_in_one_decision() {
    let mut row = record(1);
    row.first = Some(First {
        playground: false,
        org: b"org".as_slice().into(),
        repository: b"repo".as_slice().into(),
        project: SetUpProject::Existing(5),
        state: FirstState::Making,
        label: Box::new([]),
    });
    let mut d = restored(true, row.clone());
    let mut change = fixtures::opening_row();
    change.pending = None;
    outputs(&mut d, Event::Change { row: change });
    let mut ended = fixtures::opening_row();
    ended.task = 52;
    ended.change.task = 52;
    ended.pending = None;
    ended.change.state = temper_engine_domain_forge_change::State::Ended;
    outputs(&mut d, Event::Change { row: ended });
    outputs(&mut d, Event::Enqueue { entry: status(11, 7) });
    outputs(&mut d, Event::Restore { record: Stored::Flight { task: 7, entry: 11 } });
    outputs(&mut d, Event::Connection { forge: 1, state: Connection::Connecting });
    outputs(&mut d, Event::AppRecorded { forge: 1, app: AppWay::Pasted });
    outputs(
        &mut d,
        Event::Installation {
            forge: 1,
            installation: 9,
            state: Some(Installation { account: None, stray: false, suspended: false }),
        },
    );
    let out = application(&mut d, 100, Ask::RemoveForge { forge: 1 });
    row.state = ForgeState::Removing;
    row.first = None;
    let cancel = act(1, KeptActKind::Cancel { project: 5, task: fixtures::CHANGE });
    let group = act(1, KeptActKind::RemoveGroup);
    let secrets = act(1, KeptActKind::EraseSecrets);
    assert_eq!(cancel.key, [83, 23, 255, 247, 92, 143, 62, 43, 51, 193, 160, 192, 122, 39, 15, 209]);
    assert_eq!(group.key, [190, 135, 107, 178, 228, 219, 128, 189, 25, 103, 248, 25, 13, 215, 132, 164]);
    assert_eq!(secrets.key, [76, 169, 103, 135, 90, 229, 136, 72, 32, 110, 152, 252, 31, 171, 91, 8]);
    assert_eq!(
        out.as_ref(),
        [
            saved(Stored::Forge(row)),
            Request::Unregister { forge: 1 },
            Request::Erase { key: Key::Writer(1) },
            Request::Erase { key: Key::Entry(11) },
            Request::Erase { key: Key::Flight { task: 7, entry: 11 } },
            Request::Outcome {
                entry: 11,
                task: 7,
                outcome: FAILED,
                account: Some(b"set status: forge not connected; resource org/repo status test".as_slice().into())
            },
            saved(Stored::KeptAct(cancel)),
            Request::SendKeptAct(cancel),
            saved(Stored::KeptAct(group)),
            Request::SendKeptAct(group),
            saved(Stored::KeptAct(secrets)),
            Request::SendKeptAct(secrets),
            Request::MarkedForges,
            Request::ApplicationDone { request: Token::new(100), token: Token::new(100) },
        ]
    );
    assert_eq!(d.application_answer(Token::new(100)), Some(Answered::RemoveForge(RemoveAnswer::Removing)));
    assert!(d.forge_ended(1));
    assert!(d.first.is_empty() && d.connections.is_empty() && d.ended.is_empty() && d.apps.is_empty());
    assert!(d.installations.is_empty() && d.writers.is_empty() && d.flights.is_empty());
}

#[test]
fn a_removed_forge_is_in_no_list_and_its_repositories_stay_readable_by_tag() {
    let mut d = restored(true, record(1));
    application(&mut d, 100, Ask::RemoveForge { forge: 1 });
    outputs(&mut d, Event::GroupRemoved { forge: 1 });
    outputs(&mut d, Event::SecretsErased { forge: 1 });
    assert!(d.removed(1) && d.forge_ended(1));
    assert!(d.forges().rows.is_empty());
    assert!(d.readable_repositories(5).is_empty() && d.writable_repositories(5).is_empty());
    assert!(d.context_repositories(5, 2).unwrap().is_empty() && d.home(5).is_none());
    assert_eq!(d.repository_tag(5, tag(1, 2)), Some(&adopted()));
    let out = application(&mut d, 101, Ask::Adopt(fixtures::adoption()));
    assert!(!out.is_empty());
    assert!(!d.is_ready(), "refusal starts no client read");
    assert!(d.applying(Token::new(101)).is_none());
    // Late routing events cannot bring a removed forge back.
    outputs(&mut d, Event::Connection { forge: 1, state: Connection::Connected { id: 7, login: Box::new([]) } });
    outputs(
        &mut d,
        Event::Installation {
            forge: 1,
            installation: 8,
            state: Some(Installation { account: None, stray: false, suspended: false }),
        },
    );
    assert!(d.connections.is_empty() && d.installations.is_empty());
}

#[test]
fn the_playground_and_an_unknown_forge_are_refused() {
    let mut row = record(1);
    row.playground = true;
    let mut d = restored(true, row);
    for (forge, refusal) in [(1, RemoveRefusal::Playground), (9, RemoveRefusal::NoSuchForge)] {
        let out = application(&mut d, u64::from(forge), Ask::RemoveForge { forge });
        assert_eq!(
            out.as_ref(),
            [Request::ApplicationDone { request: Token::new(u64::from(forge)), token: Token::new(u64::from(forge)) }]
        );
        assert_eq!(
            d.application_answer(Token::new(u64::from(forge))),
            Some(Answered::RemoveForge(RemoveAnswer::Refused(refusal)))
        );
    }
    assert_eq!(d.records.get(&1).unwrap().state, ForgeState::Kept);
    assert!(d.kept.is_empty());
    let mut out = Queue::uncapped();
    crate::removal::remove(&mut d, &env(), 1, &mut out);
    assert!(d.forge_ended(1), "reset bypasses the request's playground refusal");
}

#[test]
fn each_answer_erases_its_act_and_the_last_erases_the_record() {
    let mut d = restored(true, record(1));
    let mut change = fixtures::opening_row();
    change.pending = None;
    outputs(&mut d, Event::Change { row: change });
    application(&mut d, 100, Ask::RemoveForge { forge: 1 });
    let cancel = act(1, KeptActKind::Cancel { project: 5, task: fixtures::CHANGE });
    assert!(outputs(&mut d, Event::KeptActAnswered { key: cancel.key, keep: true }).is_empty());
    assert!(outputs(&mut d, Event::GroupRemoved { forge: 2 }).is_empty());
    assert!(outputs(&mut d, Event::SecretsErased { forge: 2 }).is_empty());
    assert!(outputs(&mut d, Event::KeptActAnswered { key: [0; 16], keep: false }).is_empty());
    assert_eq!(
        outputs(&mut d, Event::KeptActAnswered { key: cancel.key, keep: false }).as_ref(),
        [Request::Erase { key: Key::KeptAct(cancel.key) }]
    );
    assert_eq!(
        outputs(&mut d, Event::GroupRemoved { forge: 1 }).as_ref(),
        [Request::Erase { key: Key::KeptAct(act(1, KeptActKind::RemoveGroup).key) }]
    );
    assert!(!d.removed(1));
    assert_eq!(
        outputs(&mut d, Event::SecretsErased { forge: 1 }).as_ref(),
        [
            Request::Erase { key: Key::KeptAct(act(1, KeptActKind::EraseSecrets).key) },
            Request::Erase { key: Key::Forge(1) }
        ]
    );
    assert!(d.removed(1) && d.kept.is_empty());
    assert!(outputs(&mut d, Event::SecretsErased { forge: 1 }).is_empty());
}

#[test]
fn kept_acts_are_sent_again_at_each_start_until_answered() {
    let mut d = restored(true, record(1));
    application(&mut d, 100, Ask::RemoveForge { forge: 1 });
    let mut rows = skein_lib::List::with_capacity(d.kept.len());
    for (_, row) in &d.kept {
        rows.push(*row).unwrap();
    }
    let rows = rows.into_boxed();
    for remaining in (0..=rows.len()).rev() {
        let mut next = setup_domain(&[], true);
        // Acts may arrive before their forge; reconciliation links after all restore inputs.
        for row in rows.get(..remaining).unwrap() {
            outputs(&mut next, Event::Restore { record: Stored::KeptAct(*row) });
        }
        outputs(&mut next, Event::Restore { record: Stored::ForgeNumbers { next: 3 } });
        let mut record = record(1);
        record.state = ForgeState::Removing;
        outputs(&mut next, Event::Restore { record: Stored::Forge(record) });
        outputs(&mut next, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
        let out = outputs(&mut next, Event::SendKeptActs);
        assert_eq!(sent_acts(&out).as_ref(), rows.get(..remaining).unwrap());
        if remaining == 0 {
            assert!(out.contains(&Request::Erase { key: Key::Forge(1) }));
            assert!(next.removed(1));
        } else {
            for row in rows.get(..remaining).unwrap() {
                assert!(crate::reconcile::references_resolve(&next, &LIMITS, &Stored::KeptAct(*row), &[]));
            }
            let last = rows[remaining - 1];
            outputs(&mut next, Event::KeptActAnswered { key: last.key, keep: false });
            assert_eq!(
                sent_acts(&outputs(&mut next, Event::SendKeptActs)).as_ref(),
                rows.get(..remaining - 1).unwrap()
            );
        }
    }
}

#[test]
fn a_forge_without_a_setup_entry_or_outside_the_box_keeps_no_group_act() {
    for (under_box, setup) in [(false, true), (true, false), (false, false)] {
        let mut row = record(1);
        if !setup {
            row.setup = None;
        }
        let mut d = restored(under_box, row);
        let out = application(&mut d, 100, Ask::RemoveForge { forge: 1 });
        assert_eq!(sent_acts(&out).as_ref(), [act(1, KeptActKind::EraseSecrets)]);
    }
}

#[test]
fn kept_acts_past_their_bound_do_not_fit() {
    let mut d = setup_domain(&[], true);
    let limit = LIMITS.tasks + 2 * LIMITS.forges;
    for number in 0..limit {
        let row = act(1, KeptActKind::Cancel { project: 5, task: u64::from(number) });
        outputs(&mut d, Event::Restore { record: Stored::KeptAct(row) });
    }
    let duplicate = act(1, KeptActKind::Cancel { project: 5, task: 0 });
    outputs(&mut d, Event::Restore { record: Stored::KeptAct(duplicate) });
    assert!(d.restore_fit_failure().is_none());
    assert!(!crate::reconcile::references_resolve(&d, &LIMITS, &Stored::KeptAct(duplicate), &[]));
    outputs(&mut d, Event::Restore { record: Stored::KeptAct(act(1, KeptActKind::EraseSecrets)) });
    assert_eq!(
        d.restore_fit_failure(),
        Some(&RestoreFit {
            family: Some(RecordKind::KeptAct),
            set: RestoreSet::Rows(RecordKind::KeptAct),
            held: u64::from(limit),
            additional: 1,
            limit: u64::from(limit)
        })
    );
    assert!(!valid(&Limits { tasks: u32::MAX, ..LIMITS }));
}

fn assert_failed(out: &[Request], entry: u64, task: u64) {
    assert!(out.contains(&Request::Erase { key: Key::Entry(entry) }));
    assert!(out.contains(&Request::Erase { key: Key::Flight { task, entry } }));
    let mut count = 0_u32;
    for request in out {
        if let Request::Outcome { entry: number, task: owner, outcome, .. } = request
            && *number == entry
        {
            assert_eq!((*owner, *outcome), (task, FAILED));
            count = count.checked_add(1).unwrap();
        }
    }
    assert_eq!(count, 1, "one terminal per retired entry");
}

#[test]
fn a_removed_forges_entries_settle_and_none_is_named_afterwards() {
    let mut d = restored(true, record(1));
    outputs(&mut d, Event::Restore { record: Stored::Forge(record(2)) });
    let mut other_repository = adopted();
    other_repository.provider.forge = 2;
    outputs(&mut d, Event::Restore { record: Stored::Repository(other_repository) });
    // A sent entry waits for its call's terminal even when that terminal says made.
    let sent = status(20, 8);
    outputs(&mut d, Event::Enqueue { entry: sent.clone() });
    outputs(&mut d, Event::Committed { entry: 20 });
    fixtures::answer_next(
        &mut d,
        Ok(skein_forge::api::Answer::Items { items: Box::new([]), more: false, now: env().now }),
    );
    let (call, op) = fixtures::next_call(&mut d);
    assert_eq!(op, skein_forge::api::Op::Write(sent.effect.write));
    assert!(d.flights.contains_key(&(8, 20)));
    // The unsent entry has not crossed its commit fence.
    outputs(&mut d, Event::Enqueue { entry: status(21, 9) });
    outputs(&mut d, Event::Restore { record: Stored::Flight { task: 9, entry: 21 } });
    let mut held = held_issue();
    held.number = 22;
    outputs(&mut d, Event::Restore { record: Stored::Entry { entry: held, held: true } });
    let mut other = status(23, 10);
    other.repository.forge = 2;
    outputs(&mut d, Event::Enqueue { entry: other });
    let out = application(&mut d, 100, Ask::RemoveForge { forge: 1 });
    assert_failed(&out, 21, 9);
    assert!(!d.entries.contains_key(&21));
    assert!(d.entries.contains_key(&20) && d.entries.contains_key(&22) && d.entries.contains_key(&23));
    assert!(d.flights.contains_key(&(8, 20)));
    assert!(!out.contains(&Request::Erase { key: Key::Entry(20) }));
    let out = fixtures::answer(&mut d, call, Ok(skein_forge::api::Answer::Done));
    assert_failed(&out, 20, 8);
    assert!(!d.entries.contains_key(&20) && d.flights.is_empty());
    outputs(&mut d, Event::GroupRemoved { forge: 1 });
    outputs(&mut d, Event::SecretsErased { forge: 1 });
    assert!(d.removed(1));
    let out =
        outputs(&mut d, Event::Release { task: 7, root: 7, ending: ReleaseEnding::Cancelled, reason: None, entry: 30 });
    assert!(out.contains(&Request::Erase { key: Key::Entry(22) }));
    assert!(out.contains(&Request::NotMade { task: 7, entry: 22 }));
    assert!(!d.entries.contains_key(&22));
    outputs(&mut d, Event::ContinueRelease { task: 7, entry: 30 });
    outputs(&mut d, Event::Committed { entry: 21 });
    assert!(d.entries.contains_key(&23), "the other forge's unsent entry is untouched");
    let mut later = Queue::uncapped();
    fire(&mut d, &Env { now: env().now.saturating_add(Duration::from_secs(100)), ..env() }, &mut later);
    resume(&mut d, &env(), &mut later);
    step(&mut d, &env(), Event::Settle, &mut later);
    step(&mut d, &env(), Event::Client(skein_forge_client::Event::PauseOutbox), &mut later);
    step(&mut d, &env(), Event::ReadAfresh, &mut later);
    step(&mut d, &env(), Event::SettleOutbox, &mut later);
    while let Some(request) = later.pop() {
        assert_no_retired(&request);
    }
}

#[test]
fn restored_removing_forges_retire_entries_before_resending_acts() {
    let mut d = setup_domain(&[], true);
    let mut row = record(1);
    row.state = ForgeState::Removing;
    for record in [
        Stored::ForgeNumbers { next: 2 },
        Stored::Forge(row),
        Stored::Repository(adopted()),
        Stored::Entry { entry: held_issue(), held: false },
        Stored::Flight { task: 7, entry: 11 },
        Stored::KeptAct(act(1, KeptActKind::EraseSecrets)),
    ] {
        outputs(&mut d, Event::Restore { record });
    }
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    let out = outputs(&mut d, Event::SendKeptActs);
    assert_failed(&out, 11, 7);
    assert_eq!(out.last(), Some(&Request::SendKeptAct(act(1, KeptActKind::EraseSecrets))));
    assert!(d.entries.is_empty() && d.flights.is_empty() && d.writers.is_empty());
    assert!(sent_acts(&outputs(&mut d, Event::SendKeptActs)).as_ref() == [act(1, KeptActKind::EraseSecrets)]);
    assert!(ready(&mut d).is_empty());
}

#[test]
fn removal_drops_watches_and_late_connection_diagnostics() {
    let mut d = restored(true, record(1));
    let name = branch();
    let resource = skein_forge_client::Resource {
        repository: REPO,
        what: skein_forge_client::What::Branch(b"temper/7".as_slice().into()),
    };
    outputs(&mut d, Event::Names { task: 7, resources: Box::new([name.clone()]) });
    let connect = application(&mut d, 10, Ask::Connect { forge: 1, how: How::GitHubManifest { back: None } });
    assert!(connect.contains(&Request::Manifest { forge: 1, request: Token::new(10), back: None }));
    let out = application(&mut d, 100, Ask::RemoveForge { forge: 1 });
    assert!(out.contains(&Request::Erase { key: Key::Client(skein_forge_client::Key::Live(resource)) }));
    assert!(out.contains(&Request::Erase { key: Key::Client(skein_forge_client::Key::Repository(REPO)) }));
    // Unregister ends the setup host's run through its ordinary terminal events.
    outputs(
        &mut d,
        Event::ConnectEnded {
            forge: 1,
            request: Token::new(10),
            connected: false,
            why: Some(b"setup host stopped".as_slice().into()),
            stop: None,
        },
    );
    let out = outputs(
        &mut d,
        Event::ConnectAnswered { forge: 1, request: Token::new(10), answer: ConnectAnswer::Connecting },
    );
    assert!(out.contains(&Request::ApplicationDone { request: Token::new(10), token: Token::new(10) }));
    assert_eq!(d.application_answer(Token::new(10)), Some(Answered::Connect(ConnectAnswer::Connecting)));
    assert!(d.connections.is_empty() && d.ended.is_empty() && d.apps.is_empty());
    outputs(&mut d, Event::Names { task: 8, resources: Box::new([name]) });
    for request in ready(&mut d) {
        if let Request::Call { repository, .. } = request {
            assert_ne!(repository.forge, 1);
        }
    }
}

fn assert_no_retired(request: &Request) {
    if let Request::Outcome { entry, .. } | Request::NotMade { entry, .. } = request {
        assert!(![20, 21, 22].contains(entry));
    }
    if let Request::Save { record: Stored::Entry { entry, .. }, .. } = request {
        assert!(![20, 21, 22].contains(&entry.number));
    }
    if let Request::Settled { unfinished } = request {
        for row in unfinished {
            assert!(![20, 21, 22].contains(&row.entry));
        }
    }
    if let Request::Call { repository, .. } = request {
        assert_ne!(repository.forge, 1);
    }
}
