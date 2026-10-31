//! Journal barriers and cold starts (jig/docs/design/domain/engine.md, 6).
use super::{Domain, Env, Event, Limits, Output, Released, Store, core, fixture, step, take};
use crate::{Key, Range, Record, Write};
use core::start::{Continuation, Phase};
use skein_lib::Map;

#[test]
fn the_seed_and_its_start_continuation_share_one_decision() {
    let (mut domain, env) = fixture();
    step(&mut domain, &env, Event::Restart { pending: false });
    let outputs = take(&mut domain, &env);
    let [Output::Load { phase: Phase::LoadCore, range: Range::Core }] = outputs.as_ref() else {
        panic!("the selected start asks for the core first")
    };
    step(&mut domain, &env, Event::Store(Store::Restored(Phase::LoadCore)));
    let outputs = take(&mut domain, &env);
    let [Output::Commit { number, writes }] = outputs.as_ref() else { panic!("one seed commit") };
    assert!(!writes.is_empty());
    assert!(!domain.ready());
    assert!(take(&mut domain, &env).is_empty(), "the continuation waits for the seed");
    step(&mut domain, &env, Event::Store(Store::Committed { number: *number, fill: 0 }));
    let outputs = take(&mut domain, &env);
    let [Output::ToChild(Released::Start(_))] = outputs.as_ref() else { panic!("one durable seed continuation") };
    assert!(!domain.ready(), "a released continuation still needs its input");
}

#[test]
fn a_recorded_fit_failure_stops_at_its_start_phase() {
    let (mut domain, env) = fixture();
    step(&mut domain, &env, Event::Restart { pending: false });
    take(&mut domain, &env);
    let failure = core::start::Failure::LiveSet {
        set: core::start::LiveSet::People,
        key: core::Key::People(jig_domain_people::Key::Person(1)),
        held: 1,
        limit: 0,
    };
    let action = domain.core.start_refuse(failure.clone());
    crate::domain::decide(&mut domain, &env, crate::domain::Work::Start(action));
    let outputs = take(&mut domain, &env);
    let [Output::Stop] = outputs.as_ref() else { panic!("the recorded fit failure stops the root") };
    assert_eq!(domain.core.start_phase(), Some(Phase::LoadCore));
    assert_eq!(domain.core.start_failure(), Some(failure));
    assert!(!domain.ready());
    assert!(take(&mut domain, &env).is_empty());
}

#[test]
#[ignore = "needs core-inbox"]
fn a_crash_after_the_wires_commit_starts_afresh_and_reaches_open() {
    let (original, env) = fixture();
    let mut rows = Map::uncapped();
    let _abandoned = {
        let mut original = original;
        step(&mut original, &env, Event::Restart { pending: false });
        let continuation = drive(&mut original, &env, &mut rows, true).expect("the durable Wires continuation");
        assert_eq!(original.core.start_phase(), Some(Phase::Wires));
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

/// Serve an empty application's durable snapshot, cutting only after Wires.
fn drive(domain: &mut Domain, env: &Env<Limits>, rows: &mut Map<Key, Record>, cut: bool) -> Option<Continuation> {
    let mut wires = false;
    for _ in 0..64_u32 {
        for output in take(domain, env) {
            match output {
                Output::Load { phase, range } => {
                    if range == Range::Core {
                        for interval in core::RestoreRange::LIVE {
                            restore_range(domain, env, rows, &interval);
                        }
                    } else {
                        let interval = match range {
                            Range::People => core::RestoreRange::People,
                            Range::Census(census) => core::RestoreRange::Census(census),
                            Range::Infrastructure | Range::Observability => {
                                step(domain, env, Event::Store(Store::Restored(phase)));
                                continue;
                            }
                            Range::Core => unreachable!("handled above"),
                        };
                        restore_range(domain, env, rows, &interval);
                    }
                    step(domain, env, Event::Store(Store::Restored(phase)));
                }
                Output::Switch => step(domain, env, Event::Store(Store::Switched)),
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
                            Write::CopySettingsRevision(_) => panic!("unchanged settings"),
                        }
                    }
                    step(domain, env, Event::Store(Store::Committed { number, fill: 0 }));
                    assert!(!cut || !domain.ready(), "Wires release precedes Open");
                }
                Output::ToChild(Released::Start(continuation)) if wires && cut => return Some(continuation),
                Output::ToChild(child) => step(domain, env, Event::Released(child)),
                Output::Now(core::Now::Account(_)) => {}
                other @ (Output::ProjectMade { .. }
                | Output::ApplicationDocument { .. }
                | Output::ApplicationRead { .. }
                | Output::IncidentAssociationRefused { .. }
                | Output::Application { .. }
                | Output::CoreLoad(_)
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
        if domain.ready() {
            return None;
        }
    }
    panic!("the finite start did not finish");
}

fn restore_range(domain: &mut Domain, env: &Env<Limits>, rows: &Map<Key, Record>, range: &core::RestoreRange) {
    for (key, row) in rows {
        if let Key::Core(key) = key
            && range.admits(key)
        {
            step(domain, env, Event::Store(Store::Restore(row.clone())));
        }
    }
}
