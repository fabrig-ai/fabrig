//! Journal barriers and cold starts (jig/docs/design/domain/engine/restart.md).
use super::{Domain, Env, Event, Limits, Output, Released, Store, core, fixture, step, take};
use crate::{Key, Range, Record, Write};
use core::engine::{Resume, store};
use core::start::Phase;
use skein_lib::{List, Map, Token};

pub(super) fn page(load: Token, items: Box<[store::Item]>, next: Option<store::Cursor>) -> Event {
    Event::Store(Store::Engine(store::In::Loaded {
        load,
        result: store::Answer::Rows(store::Rows::Start { items, next }),
    }))
}

pub(super) fn switched() -> Event {
    Event::Store(Store::Engine(store::In::Switched))
}

pub(super) fn empty_connector(range: Range) -> Event {
    match range {
        Range::Infrastructure => Event::Infrastructure(super::infrastructure::Event::RestoreDone),
        Range::Observability => Event::Observability(super::observability::Event::RestoreDone),
    }
}

pub(super) fn load_core(domain: &mut Domain, env: &Env<Limits>) -> Token {
    step(domain, env, Event::Restart { pending: false });
    let outputs = take(domain, env);
    let [Output::Store(store::Out::Load { load, range: store::Range::Start { ranges, after: None } })] =
        outputs.as_ref()
    else {
        panic!("the start asks for the core first: {outputs:?}")
    };
    assert_eq!(ranges.as_ref(), core::RestoreRange::LIVE);
    assert_eq!(domain.start_phase(), Some(Phase::LoadCore));
    *load
}

pub(super) fn settings_items(domain: &Domain) -> Box<[store::Item]> {
    let records = jig_test_profile::fixtures::initial_settings_current(&domain.engine.core().current);
    let mut items = List::with_capacity(u32::try_from(records.len()).expect("fixture row count"));
    for row in records {
        items.push(store::Item::Record(row)).expect("one item per settings row");
    }
    items.into_boxed()
}

#[test]
fn the_seed_and_its_start_continuation_share_one_decision() {
    let (mut domain, env) = fixture();
    assert!(!domain.ready(), "the root must start before accepting work");
    let load = load_core(&mut domain, &env);
    step(&mut domain, &env, page(load, Box::new([]), None));
    let outputs = take(&mut domain, &env);
    let [Output::Commit { number, writes }] = outputs.as_ref() else { panic!("one seed commit") };
    assert!(!writes.is_empty());
    assert!(!domain.ready());
    assert!(take(&mut domain, &env).is_empty(), "the continuation waits for the seed");
    step(&mut domain, &env, Event::Store(Store::Committed { number: *number, fill: 0 }));
    let outputs = take(&mut domain, &env);
    let [Output::ToChild(Released::Resume(resume))] = outputs.as_ref() else { panic!("one durable seed continuation") };
    assert!(!domain.ready(), "a released continuation still needs its input");
    step(&mut domain, &env, Event::Released(Released::Resume(*resume)));
    assert_eq!(domain.start_phase(), Some(Phase::LoadInbox));
    let outputs = take(&mut domain, &env);
    let [Output::Store(store::Out::Load { .. })] = outputs.as_ref() else { panic!("the inbox load follows the seed") };
}

#[test]
fn a_recorded_fit_failure_stops_at_its_start_phase() {
    use jig_domain_people as people;
    let (mut domain, mut env) = fixture();
    env.limits.core.people.people = 1;
    let load = load_core(&mut domain, &env);
    let mut items = List::with_capacity(2);
    for number in [1_u64, 2] {
        items
            .push(store::Item::Record(core::Record::People(people::Stored::Person {
                number,
                identity: people::Identity {
                    key: people::IdentityKey { provider: 0, subject: number.to_le_bytes().into() },
                    login: b"person".as_slice().into(),
                    name: b"Person".as_slice().into(),
                },
            })))
            .expect("two identities");
    }
    step(&mut domain, &env, page(load, items.into_boxed(), None));
    let outputs = take(&mut domain, &env);
    let [Output::Stop] = outputs.as_ref() else { panic!("the recorded fit failure stops the root: {outputs:?}") };
    assert_eq!(domain.start_phase(), Some(Phase::LoadCore));
    assert_eq!(
        domain.start_failure(),
        Some((
            Phase::LoadCore,
            core::start::Failure::LiveSet {
                set: core::start::LiveSet::PeopleBound(people::RestoreBound::People),
                key: core::Key::People(people::Key::Person(2)),
                held: 2,
                limit: 1,
            }
        ))
    );
    assert!(!domain.ready());
    assert!(take(&mut domain, &env).is_empty());
}

#[test]
fn restoring_numbering_keeps_the_next_page_and_numbers_the_first_commit() {
    const COMMITS: u64 = 41;
    let (mut domain, env) = fixture();
    let load = load_core(&mut domain, &env);
    let mut header = domain.engine.core().counters.deployment();
    header.commits = COMMITS;
    let cursor = store::Cursor { interval: 1, after: Box::from([7]) };
    step(
        &mut domain,
        &env,
        page(
            load,
            Box::new([store::Item::Record(core::Record::Core(core::CoreRecord::Deployment(header)))]),
            Some(store::Cursor { interval: cursor.interval, after: cursor.after.clone() }),
        ),
    );
    let outputs = take(&mut domain, &env);
    let [Output::Store(store::Out::Load { load: next, range: store::Range::Start { after: Some(after), .. } })] =
        outputs.as_ref()
    else {
        panic!("numbering must preserve the held page without a commit: {outputs:?}")
    };
    assert_ne!(*next, load);
    assert_eq!(*after, cursor);
    let items = settings_items(&domain);
    step(&mut domain, &env, page(*next, items, None));
    for _ in 0..16_u32 {
        let outputs = take(&mut domain, &env);
        match outputs.as_ref() {
            [Output::Store(store::Out::Load { load, range: store::Range::Start { after: None, .. } })] => {
                step(&mut domain, &env, page(*load, Box::new([]), None));
            }
            [Output::ConnectorLoad(range)] => step(&mut domain, &env, empty_connector(range.clone())),
            [Output::Store(store::Out::Switch)] => step(&mut domain, &env, switched()),
            [Output::Commit { number, writes }] => {
                assert_eq!(*number, COMMITS + 1);
                let mut wires = false;
                for write in writes {
                    wires |= *write == Write::Wires;
                }
                assert!(wires);
                return;
            }
            other => panic!("unexpected start output: {other:?}"),
        }
    }
    panic!("the first commit must leave");
}

#[test]
fn a_crash_after_the_wires_commit_starts_afresh_and_reaches_open() {
    let (original, env) = fixture();
    let mut rows = Map::uncapped();
    let _abandoned = {
        let mut original = original;
        step(&mut original, &env, Event::Restart { pending: false });
        let continuation = drive(&mut original, &env, &mut rows, true).expect("the durable Wires continuation");
        assert_eq!(original.start_phase(), Some(Phase::Wires));
        assert!(!original.ready());
        continuation
    };
    let (mut restarted, _) = fixture();
    step(&mut restarted, &env, Event::Restart { pending: false });
    assert!(drive(&mut restarted, &env, &mut rows, false).is_none());
    assert!(restarted.ready());
    assert_eq!(restarted.start_phases.as_slice().first(), Some(&Phase::LoadCore));
    assert_eq!(restarted.start_phases.as_slice().last(), Some(&Phase::Open));
}

#[test]
fn a_rehearsing_snapshot_stops_before_switch_while_a_normal_start_switches() {
    let (mut original, env) = fixture();
    let mut rows = Map::uncapped();
    step(&mut original, &env, Event::Restart { pending: false });
    assert!(drive(&mut original, &env, &mut rows, false).is_none());
    assert!(original.ready());
    let snapshot = rows.clone();

    let (mut rehearsal, _) = fixture();
    rehearsal.rehearse();
    assert!(!rehearsal.rehearsed());
    step(&mut rehearsal, &env, Event::Restart { pending: false });
    assert!(drive(&mut rehearsal, &env, &mut rows, false).is_none());
    assert!(rehearsal.rehearsed());
    assert_eq!(rehearsal.start_phase(), Some(Phase::Switch));
    assert_eq!(rehearsal.start_failure(), None);
    assert!(!rehearsal.ready());
    assert!(rows.iter().eq(snapshot.iter()), "a rehearsal makes no start mark");
    assert!(take(&mut rehearsal, &env).is_empty(), "no switch or effect follows a rehearsal");

    let (mut restarted, _) = fixture();
    step(&mut restarted, &env, Event::Restart { pending: false });
    assert!(drive(&mut restarted, &env, &mut rows, false).is_none());
    assert!(!restarted.rehearsed());
    assert!(restarted.ready());
    assert_eq!(restarted.start_phase(), None);
}

/// Serve an empty application's durable snapshot, cutting only after Wires.
pub(super) fn drive(domain: &mut Domain, env: &Env<Limits>, rows: &mut Map<Key, Record>, cut: bool) -> Option<Resume> {
    let mut wires = false;
    for _ in 0..64_u32 {
        for output in take(domain, env) {
            match output {
                Output::Store(store::Out::Load { load, range: store::Range::Start { ranges, after: None } }) => {
                    let mut items = List::with_capacity(rows.len());
                    for range in ranges {
                        for (key, row) in &*rows {
                            if let Key::Core(key) = key
                                && let Record::Core(row) = row
                                && range.admits(key)
                            {
                                items.push(store::Item::Record(row.clone())).expect("each row belongs to one interval");
                            }
                        }
                    }
                    step(domain, env, page(load, items.into_boxed(), None));
                }
                Output::ConnectorLoad(range) => step(domain, env, empty_connector(range)),
                Output::Store(store::Out::Switch) => {
                    assert!(!domain.rehearsed(), "a rehearsal emits no switch request");
                    step(domain, env, switched());
                }
                Output::Commit { number, mut writes } => {
                    for _ in 0..writes.len() {
                        match writes.pop().expect("snapshot transaction writes") {
                            Write::Save(row) => {
                                let _previous = rows.insert(row.key(), row).unwrap();
                            }
                            Write::Erase(key) => {
                                let _previous = rows.remove(&key);
                            }
                            Write::Wires => wires = true,
                            Write::DropHistory { .. } | Write::DropTranscript { .. } => {
                                panic!("start preserves transcripts")
                            }
                            Write::CopySettingsRevision(_) => panic!("unchanged settings"),
                        }
                    }
                    step(domain, env, Event::Store(Store::Committed { number, fill: 0 }));
                    assert!(!cut || !domain.ready(), "Wires release precedes Open");
                }
                Output::ToChild(Released::Resume(continuation)) if wires && cut => return Some(continuation),
                Output::ToChild(child) => step(domain, env, Event::Released(child)),
                Output::Now(core::Now::Account(_)) => {}
                other @ (Output::ProjectMade { .. }
                | Output::ApplicationDocument { .. }
                | Output::ApplicationRead { .. }
                | Output::IncidentAssociationRefused { .. }
                | Output::Application { .. }
                | Output::CoreLoad(_)
                | Output::Store(_)
                | Output::Core(_)
                | Output::Now(_)
                | Output::Observability(_)
                | Output::Infrastructure(_)
                | Output::ReadResult { .. }
                | Output::Read { .. }
                | Output::Agent(_)
                | Output::Stop) => panic!("an empty start only loads and commits: {other:?}"),
            }
        }
        if domain.ready() || domain.rehearsed() {
            return None;
        }
    }
    panic!("the finite start did not finish");
}
