//! Kept forge removal (temper/docs/design/draft/playground-reset.md, 4).
#![expect(
    clippy::disallowed_methods,
    clippy::match_like_matches_macro,
    reason = "ordinary test assertions inspect captured requests"
)]
use super::*;
use jig_domain_people::Role as PartyRole;

fn record(number: u16, mode: Mode) -> ForgeRecord {
    ForgeRecord {
        number,
        kind: Kind::Forgejo,
        mode,
        state: ForgeState::Kept,
        place: Place {
            api: Address { host: b"forge.example".as_slice().into(), port: 443, tls: None },
            git: b"https://forge.example".as_slice().into(),
            web: Some(b"https://forge.example".as_slice().into()),
        },
        read_only: false,
        bot: Bot { login: b"bot".as_slice().into(), name: b"Bot".as_slice().into(), email: Box::new([]) },
        asked: true,
        first: None,
    }
}

fn kept(mode: Mode) -> Domain {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Forge(record(1, mode)) });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    d
}

fn remove(d: &mut Domain, forge: u16, role: Option<PartyRole>) -> Box<[Request]> {
    outputs(
        d,
        Event::Application {
            request: Token::new(4),
            token: Token::new(104),
            retention: Duration::from_secs(60),
            ask: Ask::RemoveForge { forge },
            role,
            carried: None,
        },
    )
}

fn answer(d: &Domain, expected: RemoveAnswer) {
    assert_eq!(d.application_answer(Token::new(104)), Some(Answered::RemoveForge(expected)));
}

#[test]
fn remove_ends_the_forge_in_one_decision_and_answers_removed() {
    let mut d = kept(Mode::Owned);
    outputs(&mut d, Event::Restore { record: Stored::Forge(record(2, Mode::Fake)) });
    d.connections.insert(1, (Connection::Connected { id: 7, login: b"bot".as_slice().into() }, None)).unwrap();
    d.first
        .insert(
            1,
            First {
                playground: false,
                org: b"org".as_slice().into(),
                repository: b"repo".as_slice().into(),
                project: SetUpProject::Existing(5),
                state: FirstState::Made { id: 2 },
                label: Box::new([]),
            },
        )
        .unwrap();
    d.apps.insert(1, AppWay::Pasted).unwrap();
    for forge in [1, 2] {
        d.installations.insert((forge, 7), Installation { account: None, stray: false, suspended: false }).unwrap();
    }
    let sent = remove(&mut d, 1, Some(PartyRole::Owner));
    let mut ended = record(1, Mode::Owned);
    ended.state = ForgeState::Ended { secrets_erased: false, box_removed: false };
    assert_eq!(d.record(1), Some(&ended));
    assert!(d.forge_ended(1));
    assert_eq!(
        sent.as_ref(),
        &[
            Request::Save { record: Stored::Forge(ended), release: None, read_afresh: None },
            Request::ForgeGone { forge: 1 },
            Request::BoxRemove { forge: 1 },
            Request::Erase { key: Key::Writer(1) },
            Request::MarkedForges,
            Request::ApplicationDone { request: Token::new(4), token: Token::new(104) },
        ]
    );
    answer(&d, RemoveAnswer::Removed);
    assert!(d.connections.is_empty());
    assert!(d.first.is_empty());
    assert!(d.apps.is_empty());
    assert_eq!(d.installation(1, 7), None);
    assert!(d.installation(2, 7).is_some());
    assert!(!d.forge_ended(2));
    assert!(!crate::removal::begin(&mut d, 1, &mut Queue::uncapped()));
}

#[test]
fn a_removed_forge_leaves_every_list_and_refuses_calls() {
    let mut d = kept(Mode::Owned);
    let home = adopted();
    let mut context = home.clone();
    context.provider.repository = 3;
    context.home = false;
    context.role = Role::Context;
    for row in [home.clone(), context] {
        outputs(&mut d, Event::Restore { record: Stored::Repository(row) });
    }
    outputs(&mut d, Event::Restore { record: Stored::Names { task: 7, resources: Box::new([branch()]) } });
    assert!(d.home(5).is_some());
    remove(&mut d, 1, Some(PartyRole::Owner));
    assert!(d.forges().rows.is_empty());
    assert!(d.readable_repositories(5).is_empty());
    assert!(d.writable_repositories(5).is_empty());
    assert!(d.context_repositories(5, 2).unwrap().is_empty());
    assert!(d.adopted_ids(1).is_empty());
    assert_eq!(d.home(5), None);
    assert_eq!(d.offer(1), Offer::NotOffered);
    assert_eq!(d.repository(REPO), Some(&home));
    assert_eq!(d.repository_tag(5, tag(1, 2)), Some(&home));
    assert!(outputs(&mut d, Event::Connection { forge: 1, state: Connection::Connecting }).is_empty());
    assert_eq!(d.connection(1), Connection::NotConnected);
    let sent = outputs(
        &mut d,
        Event::Application {
            request: Token::new(5),
            token: Token::new(105),
            retention: Duration::from_secs(60),
            ask: Ask::Adopt(fixtures::adoption()),
            role: Some(PartyRole::Owner),
            carried: None,
        },
    );
    assert_eq!(
        d.application_answer(Token::new(105)),
        Some(Answered::Adopt(ApplicationOutcome::ForgeRefused(AdoptionRefusal::Unreachable)))
    );
    assert!(!sent.iter().any(|r| if let Request::Call { .. } = r { true } else { false }));
    assert!(ready(&mut d).is_empty());
}

#[test]
fn remove_asks_the_box_only_for_owned_and_fake() {
    for (mode, box_removed) in
        [(Mode::Owned, false), (Mode::Fake, false), (Mode::Managed, true), (Mode::Connected, true)]
    {
        let mut d = kept(mode);
        let sent = remove(&mut d, 1, Some(PartyRole::Owner));
        answer(&d, RemoveAnswer::Removed);
        assert_eq!(d.record(1).unwrap().state, ForgeState::Ended { secrets_erased: false, box_removed });
        assert_eq!(
            sent.iter().filter(|r| if let Request::BoxRemove { forge: 1 } = r { true } else { false }).count(),
            usize::from(!box_removed)
        );
        assert_eq!(
            sent.iter().filter(|r| if let Request::ForgeGone { forge: 1 } = r { true } else { false }).count(),
            1
        );
    }
}

#[test]
fn a_non_owner_is_refused_role() {
    for role in [None, Some(PartyRole::Member), Some(PartyRole::Maintainer), Some(PartyRole::Observer)] {
        for forge in [1, 2] {
            let mut d = kept(Mode::Fake);
            let sent = remove(&mut d, forge, role);
            answer(&d, RemoveAnswer::Refused(ForgeRefusal::Role));
            assert_eq!(d.record(1).unwrap().state, ForgeState::Kept);
            assert_eq!(sent.as_ref(), &[Request::ApplicationDone { request: Token::new(4), token: Token::new(104) }]);
        }
    }
    let mut d = kept(Mode::Fake);
    remove(&mut d, 2, Some(PartyRole::Owner));
    answer(&d, RemoveAnswer::Refused(ForgeRefusal::NoSuchForge));
    remove(&mut d, 1, Some(PartyRole::Owner));
    remove(&mut d, 1, Some(PartyRole::Owner));
    answer(&d, RemoveAnswer::Refused(ForgeRefusal::NoSuchForge));
}

#[test]
fn the_playground_forge_is_not_taken() {
    for state in [PlaygroundState::Starting, PlaygroundState::Running, PlaygroundState::Resetting(ResetStep::Forge)] {
        let mut d = kept(Mode::Fake);
        outputs(
            &mut d,
            Event::Restore { record: Stored::Playground(Playground { state, forge: 1, project: None, account: None }) },
        );
        let sent = remove(&mut d, 1, Some(PartyRole::Owner));
        answer(&d, RemoveAnswer::Refused(ForgeRefusal::NotTaken));
        assert_eq!(d.record(1).unwrap().state, ForgeState::Kept);
        assert_eq!(sent.as_ref(), &[Request::ApplicationDone { request: Token::new(4), token: Token::new(104) }]);
        outputs(&mut d, Event::Restore { record: Stored::Forge(record(2, Mode::Fake)) });
        remove(&mut d, 2, Some(PartyRole::Owner));
        answer(&d, RemoveAnswer::Removed);
        assert!(crate::removal::begin(&mut d, 1, &mut Queue::uncapped()), "reset can remove its forge");
    }
}

#[test]
fn answers_set_their_flags_and_both_call_settle() {
    for secrets_first in [false, true] {
        let mut d = kept(Mode::Fake);
        for event in
            [Event::SecretsErased { forge: 1 }, Event::BoxRemoved { forge: 1 }, Event::SecretsErased { forge: 2 }]
        {
            assert!(outputs(&mut d, event).is_empty(), "unended and absent forges ignore late answers");
        }
        outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
        outputs(&mut d, Event::Restore { record: Stored::Names { task: 7, resources: Box::new([branch()]) } });
        remove(&mut d, 1, Some(PartyRole::Owner));
        for secrets in [secrets_first, secrets_first, !secrets_first, !secrets_first] {
            let sent = outputs(
                &mut d,
                if secrets { Event::SecretsErased { forge: 1 } } else { Event::BoxRemoved { forge: 1 } },
            );
            let expected = if secrets == secrets_first {
                ForgeState::Ended { secrets_erased: secrets_first, box_removed: !secrets_first }
            } else {
                ForgeState::Ended { secrets_erased: true, box_removed: true }
            };
            assert_eq!(d.record(1).unwrap().state, expected);
            let mut saved = record(1, Mode::Fake);
            saved.state = expected;
            assert_eq!(
                sent.as_ref(),
                &[Request::Save { record: Stored::Forge(saved), release: None, read_afresh: None }]
            );
        }
    }
}

#[test]
fn a_restart_resends_only_the_unanswered_steps() {
    for secrets_erased in [false, true] {
        for box_removed in [false, true] {
            let mut d = domain();
            let mut ended = record(1, Mode::Fake);
            ended.state = ForgeState::Ended { secrets_erased, box_removed };
            outputs(&mut d, Event::Restore { record: Stored::Forge(ended) });
            let live = record(2, Mode::Fake);
            outputs(&mut d, Event::Restore { record: Stored::Forge(live.clone()) });
            let sent = outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
            let mut expected = skein_lib::List::with_capacity(4);
            expected.push(Request::ForgeKept(live)).unwrap();
            if !secrets_erased {
                expected.push(Request::ForgeGone { forge: 1 }).unwrap();
            }
            if !box_removed {
                expected.push(Request::BoxRemove { forge: 1 }).unwrap();
            }
            expected.push(Request::RestartDone { stage: RestartStage::Restored }).unwrap();
            assert_eq!(sent.as_ref(), expected.as_slice());
            assert!(d.forge_ended(1));
        }
    }
}

#[test]
fn removal_retires_waiting_reads_and_keeps_watches_from_returning() {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Forge(record(1, Mode::Fake)) });
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    let name = branch();
    outputs(&mut d, Event::Restore { record: Stored::Names { task: 7, resources: Box::new([name.clone()]) } });
    outputs(
        &mut d,
        Event::Restore {
            record: Stored::Hold(Hold {
                name: name.clone(),
                task: 7,
                writer: Some(Writer::Lost { task: 9, attempt: 1 }),
                drift: None,
            }),
        },
    );
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    fixtures::answer_next(&mut d, Err(skein_forge::api::Error::NotConnected));
    remove(&mut d, 1, Some(PartyRole::Owner));
    outputs(&mut d, Event::Names { task: 7, resources: Box::new([name.clone()]) });
    assert!(outputs(&mut d, Event::Connection { forge: 1, state: Connection::Connecting }).is_empty());
    assert!(ready(&mut d).is_empty());
    assert_eq!(d.hold(&name).unwrap().writer, Some(Writer::Lost { task: 9, attempt: 1 }));
}

#[test]
fn remove_cancels_each_change_on_the_forge_and_its_task_ends_cancelled() {
    use temper_engine_domain_forge_change as change;
    let mut d = kept(Mode::Fake);
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    for task in [51, 52] {
        let mut row = fixtures::opening_row();
        row.task = task;
        row.change.task = task;
        row.pending = None;
        outputs(&mut d, Event::Change { row });
    }
    let mut other = fixtures::opening_row();
    other.task = 53;
    other.change.task = 53;
    other.repository.forge = 2;
    let mut repository = adopted();
    repository.provider.forge = 2;
    outputs(&mut d, Event::Restore { record: Stored::Forge(record(2, Mode::Fake)) });
    outputs(&mut d, Event::Restore { record: Stored::Repository(repository) });
    outputs(&mut d, Event::Change { row: other.clone() });
    let sent = remove(&mut d, 1, Some(PartyRole::Owner));
    for task in [51, 52] {
        assert_eq!(d.change(task).unwrap().change.state, change::State::Ended);
        assert!(sent.contains(&Request::ChangeDecision {
            task,
            decision: change::Decision::Cancel,
            entry: None,
            evidence: None
        }));
    }
    assert_eq!(d.change(53), Some(&other));
}

#[test]
fn writer_and_flights_go_at_once() {
    let mut d = kept(Mode::Fake);
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    for (number, held) in [(11, false), (12, true)] {
        let entry = skein_forge_client::Entry {
            number,
            task: 51,
            repository: REPO,
            effect: skein_forge_client::Effect {
                write: skein_forge::api::Write::Close { number: 1 },
                condition: skein_forge_client::Condition::None,
            },
            start: None,
            attempt: None,
            failures: 0,
            queued: false,
        };
        outputs(&mut d, Event::Restore { record: Stored::Entry { entry, held } });
        outputs(&mut d, Event::Restore { record: Stored::Flight { task: 51, entry: number } });
    }
    let sent = remove(&mut d, 1, Some(PartyRole::Owner));
    assert!(!d.writers.contains_key(&1));
    assert!(sent.contains(&Request::Erase { key: Key::Writer(1) }));
    for entry in [11, 12] {
        assert!(!d.entries.contains_key(&entry));
        assert!(!d.flights.contains_key(&(51, entry)));
        assert!(sent.contains(&Request::Erase { key: Key::Entry(entry) }));
        assert!(sent.contains(&Request::Erase { key: Key::Flight { task: 51, entry } }));
        assert!(sent.contains(&Request::NotMade { entry, task: 51 }));
    }
}

#[test]
fn a_repository_named_by_a_kept_task_stays_until_that_task_is_erased() {
    let mut d = kept(Mode::Fake);
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(&mut d, Event::Names { task: 7, resources: Box::new([branch()]) });
    remove(&mut d, 1, Some(PartyRole::Owner));
    outputs(&mut d, Event::SecretsErased { forge: 1 });
    outputs(&mut d, Event::BoxRemoved { forge: 1 });
    assert_eq!(d.repository(REPO), Some(&adopted()));
    assert!(d.record(1).is_some());
    let sent = outputs(&mut d, Event::Unnamed { task: 7 });
    assert_eq!(d.repository(REPO), None);
    assert!(sent.contains(&Request::Erase { key: Key::Repository(REPO) }));
    assert!(sent.contains(&Request::Erase { key: Key::Client(skein_forge_client::Key::Repository(REPO)) }));
    assert_eq!(d.record(1), None);
}

#[test]
fn the_record_is_erased_once_whole_and_its_number_stays_spent() {
    let mut d = kept(Mode::Fake);
    outputs(&mut d, Event::Restore { record: Stored::ForgeNumber { last: 1 } });
    remove(&mut d, 1, Some(PartyRole::Owner));
    outputs(&mut d, Event::SecretsErased { forge: 1 });
    assert!(d.record(1).is_some());
    let sent = outputs(&mut d, Event::BoxRemoved { forge: 1 });
    assert!(sent.contains(&Request::Erase { key: Key::Forge(1) }));
    assert_eq!(d.record(1), None);
    assert!(outputs(&mut d, Event::BoxRemoved { forge: 1 }).is_empty());
    let row = record(1, Mode::Fake);
    let mut out = Queue::uncapped();
    assert_eq!(forges::make(&mut d, row.kind, row.mode, row.place, row.bot, false, None, &mut out), Some(2));
}

#[test]
fn a_removed_changes_open_pull_does_not_block_its_release() {
    let mut d = kept(Mode::Fake);
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    let mut row = fixtures::opening_row();
    row.pending = None;
    row.pull = Some(1);
    outputs(&mut d, Event::Change { row });
    remove(&mut d, 1, Some(PartyRole::Owner));
    outputs(&mut d, Event::SettleEffects { task: 51, root: 51, ending: ReleaseEnding::Cancelled });
    let sent = outputs(
        &mut d,
        Event::Release { task: 51, root: 51, ending: ReleaseEnding::Cancelled, reason: None, entry: 100 },
    );
    assert!(sent.contains(&Request::Released { task: 51 }), "removed forge needs no remote cleanup: {sent:?}");
}

#[test]
fn removal_keeps_retained_change_history_reconcilable() {
    let mut d = kept(Mode::Fake);
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    let mut row = fixtures::opening_row();
    row.pending = None;
    outputs(&mut d, Event::Change { row });
    remove(&mut d, 1, Some(PartyRole::Owner));
    outputs(&mut d, Event::SecretsErased { forge: 1 });
    outputs(&mut d, Event::BoxRemoved { forge: 1 });
    let history = Stored::Change(d.change(51).unwrap().clone());
    assert!(
        crate::reconcile::references_resolve(&d, &LIMITS, &history, &[]),
        "the retained change panel must restore after its repository is erased"
    );
}

#[test]
fn removal_settles_pending_release_cleanup_without_touching_the_forge() {
    let mut d = kept(Mode::Fake);
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    let mut row = fixtures::opening_row();
    row.pending = None;
    row.pull = Some(1);
    outputs(&mut d, Event::Change { row });
    outputs(&mut d, Event::SettleEffects { task: 51, root: 51, ending: ReleaseEnding::Cancelled });
    let sent = outputs(
        &mut d,
        Event::Release { task: 51, root: 51, ending: ReleaseEnding::Cancelled, reason: None, entry: 100 },
    );
    assert!(sent.iter().any(|request| if let Request::Save { record: Stored::Release(row), .. } = request {
        row.pending.is_some()
    } else {
        false
    }));
    let sent = remove(&mut d, 1, Some(PartyRole::Owner));
    assert!(!sent.iter().any(|request| if let Request::ReleaseFailed { .. } = request { true } else { false }));
    assert!(sent.contains(&Request::ContinueRelease { task: 51 }));
    let sent = outputs(&mut d, Event::ContinueRelease { task: 51, entry: 101 });
    assert!(sent.contains(&Request::Released { task: 51 }));
}

#[test]
fn removed_history_restores_without_a_repository_but_live_names_still_require_it() {
    let mut d = setup_domain(&[], false);
    outputs(&mut d, Event::Restore { record: Stored::ForgeNumber { last: 1 } });
    let mut row = fixtures::opening_row();
    row.pending = None;
    row.change.state = temper_engine_domain_forge_change::State::Ended;
    outputs(&mut d, Event::Restore { record: Stored::Change(row) });
    d.reconcile(&reconcile::empty_tasks());
    assert!(!crate::reconcile::references_resolve(
        &d,
        &LIMITS,
        &Stored::Names { task: 7, resources: Box::new([branch()]) },
        &[7]
    ));
}
