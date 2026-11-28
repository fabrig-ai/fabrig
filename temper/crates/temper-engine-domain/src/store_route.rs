//! Route bounded store pages and correlate their answers (temper/docs/design/domain/root.md, 7).
use crate::archive::{Waiting, goal_failed, goal_loaded, note_failed, settings_note_loaded, waiting};
use crate::boundary::{Request, Work};
use crate::domain::{Domain, close};
use crate::limits::Limits;
use crate::{Decision, Key, Range, Record, loads};
use alloc::boxed::Box;
use jig_domain_core::engine::store;
use loads::start::Place;
use skein_lib::{Env, Queue, Token};

/// Store terminals always open a decision, including the cold restore route.
pub(crate) fn load_decision(domain: &mut Domain, limits: &Limits) -> Decision {
    domain.settled = None;
    Decision::decide(&mut domain.journal, &limits.journal)
}

pub(crate) fn accept_load(domain: &mut Domain, env: &Env<Limits>, decision: Option<Decision>) {
    if let Some(decision) = decision {
        crate::decision::accept_routed(
            &mut domain.journal,
            &mut domain.engine.core_mut().counters,
            &env.limits.journal,
            decision,
            false,
        )
        .expect("read-only store terminal was admitted before changing its owner");
    }
}

pub(crate) fn request_load(
    domain: &mut Domain,
    waiter: Token,
    range: Range,
    after: Option<Key>,
    out: &mut Queue<Request>,
) {
    if waiter == RESTORE && (domain.engine.core().stopping() || domain.settling.active()) {
        return;
    }
    let mut load_out = Queue::with_capacity(1);
    let most = match &range {
        Range::History(_) => {
            let Some(Some(crate::boundary::Read::History { most, .. })) =
                domain.archive_reads.get(skein_lib::Id::from_token(waiter))
            else {
                return;
            };
            (*most).min(domain.limits.loads.rows)
        }
        Range::Core(
            crate::CoreRange::Deployment
            | crate::CoreRange::RequesterWord { .. }
            | crate::CoreRange::TaskResult { .. }
            | crate::CoreRange::ApplicationNotes(jig_domain_notes::Range::Entry { .. }),
        ) => 1,
        Range::Core(
            crate::CoreRange::Calls
            | crate::CoreRange::Tasks
            | crate::CoreRange::Settings
            | crate::CoreRange::Accounts
            | crate::CoreRange::People
            | crate::CoreRange::Sweep
            | crate::CoreRange::RunProofs
            | crate::CoreRange::Turns { .. },
        )
        | Range::Census(_)
        | Range::Forge => domain.limits.loads.rows,
        Range::Core(crate::CoreRange::ApplicationNotes(
            jig_domain_notes::Range::Lines { .. } | jig_domain_notes::Range::Project { .. },
        )) => domain.limits.loads.rows.min(domain.limits.notes.load_rows),
    };
    if loads::begin(&mut domain.loads, waiter, range.clone(), after, most, &mut load_out).is_none() {
        if waiting(domain, waiter) == Some(Waiting::ChangeFrom) {
            crate::archive::change_from_loaded(domain, waiter, None);
            return;
        }
        if waiting(domain, waiter) == Some(Waiting::Parts) {
            domain.work.push(Work::PartsFailed { waiter });
            return;
        }
        match range {
            Range::History(_) => crate::archive::history_failed(domain, waiter),
            Range::Core(crate::CoreRange::RequesterWord { .. }) => goal_failed(domain, waiter),
            Range::Core(crate::CoreRange::ApplicationNotes(_)) => note_failed(domain, waiter),
            Range::Core(
                crate::CoreRange::Calls
                | crate::CoreRange::Deployment
                | crate::CoreRange::Tasks
                | crate::CoreRange::Settings
                | crate::CoreRange::Accounts
                | crate::CoreRange::People
                | crate::CoreRange::Sweep
                | crate::CoreRange::RunProofs
                | crate::CoreRange::Turns { .. }
                | crate::CoreRange::TaskResult { .. },
            )
            | Range::Census(_)
            | Range::Forge => {
                unreachable!("startup/result load room reserved")
            }
        }
        return;
    }
    match load_out.pop().expect("load issued") {
        loads::Request::Load { owner, range, after, most, bytes } => {
            out.push(Request::Store(crate::StoreRequest::Load { owner, range, after, most, bytes }));
        }
        loads::Request::Loaded { .. } | loads::Request::Unloaded { .. } => unreachable!("begin only issues IO"),
    }
}

pub(crate) fn load_outputs(
    domain: &mut Domain,
    env: &Env<Limits>,
    load_out: &mut Queue<loads::Request>,
    decision: &mut Option<Decision>,
    out: &mut Queue<Request>,
) {
    for _ in 0..load_out.len() {
        match load_out.pop().expect("load terminal count") {
            loads::Request::Loaded { waiter, rows, next, cut } => {
                if waiter == Token::new(u64::MAX) {
                    if cut.is_some() {
                        failed(domain, env, loads::Failure::Bytes, decision.take().expect("restore page"), out);
                    } else {
                        restored(domain, env, rows, next, decision.take().expect("restore page"), out);
                    }
                    return;
                }
                match waiting(domain, waiter) {
                    Some(Waiting::ChangeDelegate) => crate::route::forge_route::delegate_record::loaded(
                        domain,
                        env,
                        decision.as_mut().expect("admitted historical terminal"),
                        waiter,
                        if cut.is_none() && next.is_none() { Some(rows) } else { None },
                    ),
                    Some(Waiting::History) => crate::archive::history_loaded(domain, waiter, rows, next, cut),
                    Some(Waiting::Parts) => crate::route::parts::loaded(
                        domain,
                        env,
                        decision.as_mut().expect("part terminal"),
                        waiter,
                        rows,
                        next,
                        cut,
                    ),
                    Some(Waiting::Document) => crate::route::document::loaded(domain, waiter, rows, next, cut, out),
                    Some(Waiting::CloserReport) => {
                        crate::route::forge_route::goal_report::loaded(domain, waiter, rows, next, cut, out);
                    }
                    Some(Waiting::Remember) => crate::route::remember::loaded(domain, waiter, rows, next, cut, out),
                    Some(Waiting::SettingsNote) => settings_note_loaded(domain, waiter, rows, next, cut),
                    Some(Waiting::GoalWords) if cut.is_none() && next.is_none() => {
                        goal_loaded(domain, waiter, rows, out);
                    }
                    Some(Waiting::GoalWords) => goal_failed(domain, waiter),
                    Some(Waiting::ChangeFrom) => {
                        crate::archive::change_from_loaded(
                            domain,
                            waiter,
                            if cut.is_none() && next.is_none() { Some(rows) } else { None },
                        );
                    }
                    None => {}
                }
            }
            loads::Request::Unloaded { waiter, failure } => {
                if waiter == Token::new(u64::MAX) {
                    failed(domain, env, failure, decision.take().expect("restore failure"), out);
                    return;
                }
                match waiting(domain, waiter) {
                    Some(Waiting::ChangeDelegate) => crate::route::forge_route::delegate_record::loaded(
                        domain,
                        env,
                        decision.as_mut().expect("admitted historical failure"),
                        waiter,
                        None,
                    ),
                    Some(Waiting::History) => crate::archive::history_failed(domain, waiter),
                    Some(Waiting::Parts) => crate::route::parts::failed(
                        domain,
                        env,
                        decision.as_mut().expect("part terminal"),
                        waiter,
                        jig_domain_core::PartFailure::Unreadable,
                    ),
                    Some(Waiting::Document) => crate::route::document::failed(domain, waiter),
                    Some(Waiting::Remember) => crate::route::remember::failed(domain, waiter),
                    Some(Waiting::SettingsNote | Waiting::CloserReport) => note_failed(domain, waiter),
                    Some(Waiting::GoalWords) => goal_failed(domain, waiter),
                    Some(Waiting::ChangeFrom) => crate::archive::change_from_loaded(domain, waiter, None),
                    None => {}
                }
            }
            loads::Request::Load { .. } => unreachable!("terminal methods never issue IO"),
        }
    }
}

const RESTORE: Token = Token::new(u64::MAX);

/// The connector's store IO takes the same slots as the engine's walks.
pub(crate) fn load_forge(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision, after: Option<Key>) {
    let range = Range::Forge;
    domain.start_page = Some(loads::Kind::Forge);
    domain.startup_range = Some(range.clone());
    crate::domain::emit(decision, &env.limits, crate::Delivery::Load { waiter: RESTORE, range, after });
}

/// The engine sees a terminal only for the page it asked for. The store adapter
/// retains the walk's typed position behind the opaque cursor it hands back.
pub(crate) fn released(domain: &mut Domain, request: store::Out, out: &mut Queue<Request>) {
    if domain.engine.core().stopping() || domain.settling.active() {
        return;
    }
    match request {
        store::Out::Load { load, range: store::Range::Start { ranges, after } } => {
            match domain.start_walk.request(ranges, after) {
                Place::Page { range, after } => {
                    assert!(domain.start_load.replace(load).is_none(), "one start page in flight");
                    domain.startup_range = Some(range.clone());
                    request_load(domain, RESTORE, range, after, out);
                }
                Place::CoreEnd => {
                    domain.core_header_loaded = true;
                    domain.work.push(Work::EngineStore(store::In::Loaded {
                        load,
                        result: store::Answer::Rows(store::Rows::Start { items: Box::new([]), next: None }),
                    }));
                }
            }
        }
        store::Out::Switch => {
            if domain.start_pending {
                out.push(Request::Store(crate::StoreRequest::Switch));
            } else {
                domain.work.push(Work::EngineStore(store::In::Switched));
            }
        }
        request @ store::Out::Load {
            range:
                store::Range::Record(_)
                | store::Range::Proposal { .. }
                | store::Range::Window { .. }
                | store::Range::Notes(_)
                | store::Range::Read(_),
            ..
        } => out.push(Request::Store(crate::StoreRequest::Engine(request))),
    }
}

pub(crate) fn missing(domain: &mut Domain, owner: Token, record: jig_domain_core::Key, target: jig_domain_core::Key) {
    if loads::startup_owner(&domain.loads, owner) && domain.start_load.is_some() {
        domain.start_missing.push((record, target));
    }
}

fn restored(
    domain: &mut Domain,
    env: &Env<Limits>,
    rows: Box<[Record]>,
    next: Option<Key>,
    mut decision: Decision,
    out: &mut Queue<Request>,
) {
    domain.start_page = None;
    let range = domain.startup_range.take().expect("restore load in flight");
    if domain.engine.core().stopping() || domain.settling.active() {
        domain.start_load = None;
        domain.start_missing.clear();
    } else if let Some(load) = domain.start_load.take() {
        let count = domain
            .start_missing
            .len()
            .checked_add(u32::try_from(rows.len()).expect("bounded page rows"))
            .expect("one bounded page of rows and missing links");
        let mut items = skein_lib::List::with_capacity(count);
        for _ in 0..domain.start_missing.len() {
            let (record, target) = domain.start_missing.pop().expect("missing item count");
            items.push(store::Item::Missing { record, target }).expect("counted missing links");
        }
        let next = domain.start_walk.answered(range, next);
        for row in rows {
            let Record::Core(record) = row else { unreachable!("engine walk only loads engine records") };
            observe_row(domain, env, &record);
            if domain.start_stop.is_some() {
                close(domain, env, decision, out);
                return;
            }
            items.push(store::Item::Record(record)).expect("one item per restored record");
        }
        domain.work.push(Work::EngineStore(store::In::Loaded {
            load,
            result: store::Answer::Rows(store::Rows::Start { items: items.into_boxed(), next }),
        }));
    } else {
        assert_eq!(range, Range::Forge, "connector restore owns this page");
        crate::route::forge_route::restore_page(domain, env, &mut decision, rows, next);
    }
    crate::route::route_into(domain, env, &mut decision);
    close(domain, env, decision, out);
}

pub(crate) fn observe_row(domain: &mut Domain, env: &Env<Limits>, row: &jig_domain_core::Record) {
    use jig_domain_core::{CoreRecord, Record};
    if let Record::Core(CoreRecord::Deployment(deployment)) = row
        && !domain.forge.bind_deployment(deployment.id, &env.limits.forge)
    {
        domain.stop_start(crate::StartStop::Binding);
    }
    // Seed the phase comparison without replaying a lifecycle wake on restore.
    if let Record::Tasks(jig_domain_tasks::Stored::Live(task)) = row
        && let Some(phase) = crate::route::forge_route::LandingUnavailable::of(&task.phase)
    {
        domain.landing_unavailable.insert(task.number, phase).expect("one phase per restored live task");
    }
}

fn failed(
    domain: &mut Domain,
    env: &Env<Limits>,
    failure: loads::Failure,
    mut decision: Decision,
    out: &mut Queue<Request>,
) {
    domain.start_page = None;
    let range = domain.startup_range.take().expect("restore load in flight");
    if let Some(load) = domain.start_load.take() {
        let routed = crate::engine::step(
            &mut domain.engine,
            env,
            jig_domain_core::engine::In::Store(store::In::Loaded {
                load,
                result: store::Answer::Failed(store::Failure::Failed),
            }),
        );
        crate::route::route_engine_requests(domain, env, &mut decision, routed);
    }
    domain.start_missing.clear();
    if !domain.engine.core().stopping() && !domain.settling.active() {
        domain.stop_start(crate::StartStop::Load(loads::RestoreFailure { range: loads::kind(&range), failure }));
    }
    close(domain, env, decision, out);
}
