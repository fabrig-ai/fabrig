//! Owner settings decisions, atomic with their keyed answer (engine.md, 17.2).
use crate::{
    Core, Event, Limits, Request,
    routing::{
        Work,
        party::dispatch::{Party, Routed},
    },
    settings as s,
};
use alloc::boxed::Box;
use jig_domain_people as people;
use skein_lib::{Env, Map, Queue, ReplyTo, Token};

#[expect(clippy::too_many_arguments, reason = "the keyed envelope and its value cross admission together")]
pub(crate) fn start(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Work>,
    reply_to: ReplyTo,
    sign_in: u64,
    key: [u8; 16],
    digest: [u8; 32],
    change: Box<s::Change>,
) -> Queue<Request> {
    assert!(core.settings_input.is_none(), "one settings admission at a time");
    let against = change.against;
    let projects = s::application::projects(&change.items);
    let room = route_writes(&env.limits)
        .expect("validated settings room")
        .saturating_sub(1)
        .saturating_sub(people::outputs_bound(&env.limits.people));
    core.settings_input = Some((
        match s::change_shape(&change.items, &env.limits) {
            Ok(()) => Ok(change),
            Err(failure) => Err(failure),
        },
        room,
    ));
    let result = crate::routing::party::people::route_people(
        core,
        env,
        work,
        people::Event::Ask {
            reply_to,
            sign_in,
            key,
            ask: people::Ask::ChangeSettings { against, digest, projects },
            models: Box::new([]),
        },
    );
    drop(core.settings_input.take());
    result
}
pub(crate) fn admitted(core: &mut Core, env: &Env<Limits>, routed: &mut Routed, party: Party, against: u32) {
    let Party { request, person, .. } = party;

    let outcome = match core.settings_input.take() {
        Some(_) if against != core.current.newest() => {
            people::Outcome::Refused(people::Refusal::Stale { newest: core.current.newest() })
        }
        Some((Ok(change), room)) => {
            if has_application(&change.items) {
                if let Err(why) = preflight(core, env, &change, None) {
                    people::Outcome::Refused(why)
                } else {
                    s::application::start(core, routed, request, person, *change, room, None);
                    return;
                }
            } else {
                match decide(core, env, routed, s::Decider::Party(person), None, *change, room) {
                    Ok(revision) => people::Outcome::SettingsChanged { revision },
                    Err(why) => people::Outcome::Refused(why),
                }
            }
        }
        Some((Err(failure), _)) => malformed(failure),
        None => people::Outcome::Refused(people::Refusal::NotOffered),
    };
    routed.work.push(Work::Event(Event::People(people::Event::Decided { request, outcome })));
}
pub(crate) fn malformed(failure: s::Failure) -> people::Outcome {
    people::Outcome::Refused(malformed_refusal(failure))
}
pub(crate) fn malformed_refusal(failure: s::Failure) -> people::Refusal {
    people::Refusal::NotWellFormed {
        at: 0,
        why: people::Malformed::Settings { setting: failure.setting, fault: failure.fault },
    }
}
fn has_application(items: &[s::Item]) -> bool {
    for item in items {
        if s::application::is_application(item.setting) {
            return true;
        }
    }
    false
}
fn preflight(
    core: &Core,
    env: &Env<Limits>,
    change: &s::Change,
    proposal: Option<(people::Proposer, u32)>,
) -> Result<(), people::Refusal> {
    if !withdrawal_room(core, proposal) || core.current.newest().checked_add(1).is_none() {
        return Err(people::Refusal::Limit { which: people::LimitKind::Arithmetic });
    }
    if change.against != core.current.newest() {
        return Err(people::Refusal::Stale { newest: core.current.newest() });
    }
    if let Err(failure) = s::change_shape(&change.items, &env.limits) {
        return Err(malformed_refusal(failure));
    }
    if let Err(failure) = s::application::check(core, &change.items) {
        return Err(malformed_refusal(failure));
    }
    match s::check_change(&core.current, &change.items, &core.settings, &core.connectors, &env.limits) {
        Ok(()) => Ok(()),
        Err(failure) => Err(malformed_refusal(failure)),
    }
}

#[expect(clippy::too_many_arguments, reason = "the keyed change retains its proposal identity through judging")]
pub(crate) fn decide_judged(
    core: &mut Core,
    env: &Env<Limits>,
    routed: &mut Routed,
    person: u64,
    change: s::Change,
    room: u32,
    kept: Box<[s::application::Kept]>,
    proposal: Option<(people::Proposer, u32)>,
) -> Result<u32, people::Refusal> {
    decide_with(core, env, routed, s::Decider::Party(person), proposal, change, room, kept)
}

pub(crate) fn decide(
    core: &mut Core,
    env: &Env<Limits>,
    routed: &mut Routed,
    by: s::Decider,
    proposal: Option<(people::Proposer, u32)>,
    change: s::Change,
    room: u32,
) -> Result<u32, people::Refusal> {
    decide_with(core, env, routed, by, proposal, change, room, Box::default())
}
#[expect(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    reason = "the decision envelope and judged metadata commit atomically along the existing settings path"
)]
fn decide_with(
    core: &mut Core,
    env: &Env<Limits>,
    routed: &mut Routed,
    by: s::Decider,
    proposal: Option<(people::Proposer, u32)>,
    mut change: s::Change,
    room: u32,
    kept: Box<[s::application::Kept]>,
) -> Result<u32, people::Refusal> {
    let previous = core.current.newest();
    if !withdrawal_room(core, proposal) {
        return Err(people::Refusal::Limit { which: people::LimitKind::Arithmetic });
    }
    if change.against != previous {
        return Err(people::Refusal::Stale { newest: previous });
    }
    if kept.is_empty() && !s::changes_current(&core.current, &change.items) {
        return Err(people::Refusal::NotWellFormed { at: 0, why: people::Malformed::SettingsUnchanged });
    }
    preflight(core, env, &change, proposal)?;
    if let Err(failure) = s::application::references(&core.current, &change.items, &kept) {
        return Err(malformed_refusal(failure));
    }
    let mut jig_items = skein_lib::List::with_capacity(u32::try_from(change.items.len()).expect("bounded items"));
    for item in change.items {
        if !s::application::is_application(item.setting) {
            jig_items.push(item).expect("jig items");
        }
    }
    change.items = jig_items.into_boxed();
    let charters = s::check::changed_charters(&core.current, &change.items, &core.settings, &env.limits);
    let Some(number) = previous.checked_add(1) else {
        return Err(people::Refusal::Limit { which: people::LimitKind::Arithmetic });
    };
    let mut after = Map::with_capacity(env.limits.settings.revisions.checked_add(1).expect("held names"));
    let moves = change.running == s::Running::Moves;
    if moves {
        core.tasks.held_after_moves(None, number, &charters, &mut after);
    }
    let keep_previous = if moves { after.contains_key(&previous) } else { core.tasks.holders(previous) > 0 };
    if !moves {
        for (&revision, _) in &core.held.heads {
            after.insert(revision, ()).expect("existing held names");
        }
        if keep_previous {
            after.insert(previous, ()).expect("previous newest name");
        }
    }
    let plan = core.held.plan_held(&core.current, &change.items, number, &after, &env.limits);
    let held_count = after.len();
    if held_count > env.limits.settings.revisions || plan.versions() > env.limits.settings.versions {
        let revision = match after.first() {
            Some((&revision, ())) => revision,
            None => previous,
        };
        let (tasks, more) = core.tasks.holding(revision, env.limits.tasks.circle);
        return Err(people::Refusal::Full { revision, tasks, more });
    }
    let mut templates = false;
    let mut rules = false;
    let mut charters = false;
    for item in &change.items {
        if s::changes_current(&core.current, core::slice::from_ref(item)) {
            match item.setting {
                s::Setting::Charter(_) => charters = true,
                s::Setting::Template(_) | s::Setting::Policy(_) => templates = true,
                s::Setting::Rules => rules = true,
                jig_domain_people::Setting::Application { .. } => unreachable!("an application's setting: apps-core"),
            }
        }
    }
    let before = routed.out.len();
    charters |= core.held.retain(&core.current, &plan, &mut routed.out);
    let held_writes = routed.out.len().checked_sub(before).expect("held writes appended");
    drop(plan);
    drop(after);
    if keep_previous {
        core.held.heads.insert(previous, core.current.head.clone()).expect("held heads preflighted");
    }
    let writes = s::apply(&mut core.current, &change.items, number);
    core.current.head =
        s::Head { number, against: previous, at: env.wall, by, proposal, moved: change.running == s::Running::Moves };
    let application_writes = u32::try_from(kept.len()).expect("bounded application writes");
    let proposal_number = match proposal {
        Some((_, number)) => Some(u64::from(number)),
        None => None,
    };
    let applications = s::application::keep(core, &mut routed.out, kept, number, proposal_number);
    let revision = s::Revision { head: core.current.head.clone(), items: change.items, applications };
    let (writes, oldest) = s::restart::revision_writes_with_hold(
        Some(revision),
        writes,
        core.current.oldest,
        core.current.newest(),
        env.limits.settings.history,
        room.saturating_sub(s::withdrawal_writes(&env.limits).expect("withdrawal write bound"))
            .saturating_sub(held_writes)
            .saturating_sub(application_writes)
            .saturating_sub(s::move_writes(&env.limits).expect("move write bound")),
        keep_previous,
    );
    core.current.oldest = oldest;
    for write in writes.into_boxed() {
        routed.out.push(Request::Write(write));
    }
    withdraw_stale(core, env, &mut routed.out, proposal);
    core.tell_replaced_settings(env, rules, charters);
    if moves {
        move_tasks(core, env, &mut routed.out, None);
    }
    if charters || rules {
        core.touch_deployment();
        core.touch_run_waits();
    }
    crate::routing::view::marks::settings_revised(core, &mut routed.work, templates);
    Ok(number)
}

pub(crate) fn route_writes(limits: &Limits) -> Option<u32> {
    s::writes_bound(limits)?.checked_add(people::outputs_bound(&limits.people))
}

#[expect(clippy::too_many_arguments, reason = "one keyed proposal envelope and its held value")]
pub(crate) fn propose_start(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Work>,
    reply_to: ReplyTo,
    sign_in: u64,
    key: [u8; 16],
    digest: [u8; 32],
    change: s::Proposed,
) -> Queue<Request> {
    assert!(core.proposed_input.is_none(), "one settings proposal admission");
    let s::Proposed { against, items, reason } = change;
    prepare_proposal(core, env, against, items);
    let result = crate::routing::party::people::route_people(
        core,
        env,
        work,
        people::Event::Ask {
            reply_to,
            sign_in,
            key,
            models: Box::new([]),
            ask: people::Ask::Propose {
                project: people::DEPLOYMENT,
                action: people::Action::Settings { against, digest },
                reason,
            },
        },
    );
    drop(core.proposed_input.take());
    result
}

pub(crate) fn propose_admitted(
    core: &mut Core,
    env: &Env<Limits>,
    routed: &mut Routed,
    party: Party,
    against: u32,
    reason: Box<[u8]>,
) {
    let Party { request, person, .. } = party;

    let result = proposal_check(core, env, against);
    let change = match result {
        Ok(change) => change,
        Err(why) => {
            routed.work.push(Work::Event(Event::People(people::Event::Decided {
                request,
                outcome: people::Outcome::Refused(why),
            })));
            return;
        }
    };
    let Some(next) = core.counters.deployment().messages.checked_add(1) else {
        return crate::routing::refusals::pool_refused(
            &mut routed.work,
            request,
            people::Refusal::Limit { which: people::LimitKind::Arithmetic },
        );
    };
    let Ok(number) = u32::try_from(next) else {
        return crate::routing::refusals::pool_refused(
            &mut routed.work,
            request,
            people::Refusal::Limit { which: people::LimitKind::Arithmetic },
        );
    };
    let fresh = crate::fresh(&mut core.counters, crate::Family::Message).expect("proposal number preflighted");
    assert!(fresh == u64::from(number), "same proposal sequence");
    let heads = s::proposal_heads(&change.items);
    assert!(core.settings_offering_flights() == 0, "one settings offering");
    core.fly(crate::routing::flight::Flight::SettingsOffering {
        request,
        by: person,
        number,
        value: s::Proposal { applications: Box::default(), against, items: change.items },
    });
    let request = core.fly_goal(
        request,
        crate::GoalRoute::Offering { proposal: fresh, to: people::Role::Owner.number(), stubs: Box::new([]) },
    );
    routed.work.push(Work::Event(Event::Tasks(jig_domain_tasks::Event::ProposePerson {
        reply_to: ReplyTo::new(request),
        proposal: jig_domain_tasks::PersonProposal {
            number: fresh,
            proposer: person,
            project: people::DEPLOYMENT,
            action: jig_domain_tasks::PersonAction::Settings { against, items: heads },
            reason,
            state: jig_domain_tasks::PersonProposalState::Pending { since: env.wall },
        },
    })));
}
fn proposal_check(core: &mut Core, env: &Env<Limits>, against: u32) -> Result<Box<s::Proposal>, people::Refusal> {
    let input = core.proposed_input.take().ok_or(people::Refusal::NotOffered)?;
    if against != core.current.newest() {
        return Err(people::Refusal::Stale { newest: core.current.newest() });
    }
    let change = match input {
        Ok(change) => change,
        Err(why) => return Err(people::Refusal::NotWellFormed { at: 0, why }),
    };
    if change.against != against {
        return Err(people::Refusal::NotOffered);
    }
    if let Err(failure) = s::application::check(core, &change.items) {
        return Err(malformed_refusal(failure));
    }
    if let Err(failure) = s::check_change(&core.current, &change.items, &core.settings, &core.connectors, &env.limits) {
        return Err(malformed_refusal(failure));
    }
    let mut owner = false;
    for holding in core.people.project_members(people::DEPLOYMENT).unwrap_or(&[]) {
        owner |= holding.role == people::Role::Owner;
    }
    if !owner {
        return Err(people::Refusal::Authority {
            lacking: people::Lack::Approver,
            proposable: Some(people::Accepter::Nobody),
        });
    }
    if core.settings_proposals.len() >= env.limits.settings.proposals {
        return Err(people::Refusal::Limit { which: people::LimitKind::SettingsProposals });
    }
    Ok(change)
}

/// Revision changes and release startup share the exact durable withdrawal path.
pub(crate) fn withdraw_stale(
    core: &mut Core,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
    except: Option<(people::Proposer, u32)>,
) {
    let mut pending = skein_lib::List::with_capacity(env.limits.settings.proposals);
    for (&key, _) in &core.settings_proposals {
        if Some(key) != except {
            pending.push(key).expect("bounded pending sets");
        }
    }
    for (proposer, proposal) in pending.into_boxed() {
        if let people::Proposer::Task(task) = proposer {
            let message = crate::fresh(&mut core.counters, crate::Family::Message).expect("settings outcome sequence");
            crate::routing::children::append_tasks(
                core,
                env,
                jig_domain_tasks::Event::DecideSettingsProposal {
                    reply_to: None,
                    decision: jig_domain_tasks::SettingsDecision {
                        proposer: task,
                        proposal: u64::from(proposal),
                        message,
                        by: jig_domain_tasks::Party::Deployment { project: people::DEPLOYMENT },
                        outcome: jig_domain_tasks::SettingsOutcome::Stale { newest: core.current.newest() },
                        reason: Box::new([]),
                    },
                },
                out,
            );
            continue;
        }
        let people::Proposer::Party(person) = proposer else { unreachable!("party or task") };
        crate::routing::children::append_tasks(
            core,
            env,
            jig_domain_tasks::Event::WithdrawPersonProposal {
                reply_to: None,
                proposer: person,
                proposal: u64::from(proposal),
            },
            out,
        );
    }
}

/// Keep or erase the held set beside its task-owned proposal row.
pub(crate) fn proposal_saved(core: &mut Core, record: &jig_domain_tasks::Stored, out: &mut Queue<Request>) {
    if let jig_domain_tasks::Stored::History(history) = record
        && let Some(row) = &history.proposal
        && let jig_domain_tasks::ProposalAction::Settings { .. } = row.action
    {
        let pending = match row.state {
            jig_domain_tasks::ProposalState::Pending { .. } => true,
            jig_domain_tasks::ProposalState::Accepted { .. }
            | jig_domain_tasks::ProposalState::Rejected { .. }
            | jig_domain_tasks::ProposalState::Withdrawn => false,
        };
        keep_set(core, people::Proposer::Task(row.proposer), row.number, pending, out);
        return;
    }
    let row = match record {
        jig_domain_tasks::Stored::PersonProposal(row) | jig_domain_tasks::Stored::DecidedProposal(row) => row,
        jig_domain_tasks::Stored::Live(_)
        | jig_domain_tasks::Stored::Ended(_)
        | jig_domain_tasks::Stored::Activity(_)
        | jig_domain_tasks::Stored::Milestone(_)
        | jig_domain_tasks::Stored::PlacedHold(_)
        | jig_domain_tasks::Stored::Stub(_)
        | jig_domain_tasks::Stored::Ledger(_)
        | jig_domain_tasks::Stored::Writer(_)
        | jig_domain_tasks::Stored::Pool(_)
        | jig_domain_tasks::Stored::ClosedLedger(_)
        | jig_domain_tasks::Stored::History(_)
        | jig_domain_tasks::Stored::Result(_)
        | jig_domain_tasks::Stored::EndedByParty(_)
        | jig_domain_tasks::Stored::EndedByProject(_)
        | jig_domain_tasks::Stored::EndedDelegate(_)
        | jig_domain_tasks::Stored::Delegate(_)
        | jig_domain_tasks::Stored::Update(_) => return,
    };
    if !row.action.is_settings() {
        return;
    }
    keep_set(core, people::Proposer::Party(row.proposer), row.number, row.state.pending_since().is_some(), out);
}
fn keep_set(core: &mut Core, proposer: people::Proposer, number: u64, pending: bool, out: &mut Queue<Request>) {
    let number = u32::try_from(number).expect("settings proposal fits key");
    let key = (proposer, number);
    if pending {
        if core.settings_proposals.contains_key(&key) {
            return;
        }
        let (offered_by, offered, mut value) = core.take_settings_offering(None).expect("admitted held set");
        let who = match proposer {
            people::Proposer::Party(n) | people::Proposer::Task(n) => n,
        };
        assert!(offered_by == who && offered == number, "same proposal set");
        s::application::keep_proposal(core, &mut value, number, out);
        out.push(Request::Write(crate::Write::Save(crate::Record::Core(crate::CoreRecord::Settings(
            s::Record::Proposal { proposer, proposal: number, value: value.clone() },
        )))));
        assert!(core.settings_proposals.insert(key, value).expect("settings proposal bound").is_none(), "fresh set");
    } else {
        let value = core.settings_proposals.remove(&key).expect("decided settings set");
        // An accepted set has already become connector changes in this revision.
        if core.current.head.proposal != Some(key) {
            s::application::drop_proposal(core, &value.applications, number, out);
        }
        out.push(Request::Write(crate::Write::Erase(crate::Key::Core(crate::CoreKey::Settings(s::Key::Proposal {
            proposer,
            proposal: number,
        })))));
    }
    core.touch_settings();
}

/// Counts decided by an owner's move (engine.md, 17.4).
pub(crate) struct Moved {
    pub moved: u32,
    pub stayed: u32,
}
/// An owner's move of a held revision (engine.md, 17.4).
pub(crate) fn move_to_newest(
    core: &mut Core,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
    revision: u32,
) -> Result<Moved, people::Refusal> {
    if revision == core.current.newest() {
        Ok(Moved { moved: 0, stayed: 0 })
    } else if core.held(revision) {
        Ok(move_tasks(core, env, out, Some(revision)))
    } else {
        Err(people::Refusal::Unknown)
    }
}

fn move_tasks(core: &mut Core, env: &Env<Limits>, out: &mut Queue<Request>, from: Option<u32>) -> Moved {
    use jig_domain_tasks as tasks;
    let mut child = Queue::uncapped();
    crate::routing::step::step_tasks(core, env, tasks::Event::MoveToNewest { from }, &mut child);
    let mut counts = None;
    for _ in 0..child.len() {
        let request = child.pop().expect("move output count");
        if let tasks::Request::MovedToNewest { moved, stayed } = request {
            counts = Some(Moved { moved, stayed });
        } else {
            child.push(request);
        }
    }
    let mut routed = crate::routing::tag_task_outputs(core, env, child);
    for _ in 0..routed.len() {
        let request = routed.pop().expect("routed move output count");
        out.push(request);
    }
    core.touch_settings();
    counts.expect("move answers counts")
}

fn prepare_proposal(core: &mut Core, env: &Env<Limits>, against: u32, items: Box<[s::Item]>) {
    core.proposed_input = Some(if !has_application(&items) && !s::changes_current(&core.current, &items) {
        Err(people::Malformed::SettingsUnchanged)
    } else if items.len() > skein_lib::length::usize_of(jig_domain_tasks::SETTINGS_ITEMS) {
        Err(people::Malformed::Settings {
            setting: items.first().expect("past item bound").setting,
            fault: s::Fault::Count,
        })
    } else {
        match s::change_shape(&items, &env.limits) {
            Ok(()) => Ok(Box::new(s::Proposal { applications: Box::default(), against, items })),
            Err(failure) => Err(people::Malformed::Settings { setting: failure.setting, fault: failure.fault }),
        }
    });
}

pub(crate) fn propose_run(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Work>,
    to: ReplyTo,
    key: crate::CallKey,
    change: s::Proposed,
) -> Queue<Request> {
    use jig_domain_tasks as tasks;
    let s::Proposed { against, items, reason } = change;
    prepare_proposal(core, env, against, items);
    let checked = proposal_check(core, env, against);
    let set = match checked {
        Ok(set) => *set,
        Err(why) => {
            return crate::routing::run::calls::route_named_answer(
                core,
                to,
                key,
                crate::CallPart::SettingsRefused(why),
            );
        }
    };
    let Some(next) = core.counters.deployment().messages.checked_add(1) else {
        return crate::routing::run::calls::route_named_answer(
            core,
            to,
            key,
            crate::CallPart::SettingsRefused(people::Refusal::Limit { which: people::LimitKind::Arithmetic }),
        );
    };
    let Ok(number) = u32::try_from(next) else {
        return crate::routing::run::calls::route_named_answer(
            core,
            to,
            key,
            crate::CallPart::SettingsRefused(people::Refusal::Limit { which: people::LimitKind::Arithmetic }),
        );
    };
    let proposal = tasks::Proposal {
        number: next,
        proposer: key.task,
        project: people::DEPLOYMENT,
        action: tasks::ProposalAction::Settings { against, items: s::proposal_heads(&set.items) },
        reason,
        as_holder: false,
        state: tasks::ProposalState::Pending {
            holder: tasks::ProposalHolder::Policy { project: people::DEPLOYMENT, kind: tasks::ProposalKind::Settings },
            since: env.wall,
        },
    };
    // Preflight before retaining the full set; the child repeats its entrance check.
    if let Some(problem) = core.tasks.proposal_problem(&env.limits.tasks, &proposal) {
        return crate::routing::run::calls::route_named_answer(
            core,
            to,
            key,
            crate::CallPart::ProposalRefused(problem),
        );
    }
    let fresh = crate::fresh(&mut core.counters, crate::Family::Message).expect("preflighted proposal number");
    assert!(fresh == next && core.settings_offering_flights() == 0, "one fresh settings offering");
    let token = to.into_token();
    core.fly(crate::routing::flight::Flight::SettingsOffering { request: token, by: key.task, number, value: set });
    core.pending_calls.insert(key, true).expect("call room reserved");
    let token = core.fly_call(token, crate::RoutedCall::Propose { key, proposal: next });
    work.push(Work::Event(Event::Tasks(tasks::Event::Propose { reply_to: ReplyTo::new(token), proposal })));
    Queue::uncapped()
}

/// The same change commits a party's or a task's owner-approved set (engine.md, 17.6).
pub(crate) fn accept(
    core: &mut Core,
    env: &Env<Limits>,
    routed: &mut Routed,
    person: u64,
    request: Token,
    key: (people::Proposer, u32),
    running: people::Running,
) -> Result<Option<u32>, people::Refusal> {
    let set = core.settings_proposals.get(&key).expect("pending settings set").clone();
    let room = route_writes(&env.limits)
        .expect("settings decision room")
        .saturating_sub(1)
        .saturating_sub(people::outputs_bound(&env.limits.people));
    let change = s::application::proposal_change(core, set, running)?;
    if has_application(&change.items) {
        preflight(core, env, &change, Some(key))?;
        s::application::start(core, routed, request, person, change, room, Some(key));
        return Ok(None);
    }
    let revision = decide(core, env, routed, s::Decider::Party(person), Some(key), change, room)?;
    Ok(Some(revision))
}

/// Check every new task notification's number before changing a revision or its defaults.
pub(crate) fn withdrawal_room(core: &Core, except: Option<(people::Proposer, u32)>) -> bool {
    let mut messages = 0_u64;
    for (&key, _) in &core.settings_proposals {
        if Some(key) != except
            && let people::Proposer::Task(_) = key.0
        {
            messages = messages.checked_add(1).expect("bounded pending settings");
        }
    }
    core.counters.deployment().messages.checked_add(messages).is_some()
}
