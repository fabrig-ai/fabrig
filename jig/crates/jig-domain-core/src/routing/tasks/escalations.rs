//! Escalation choices and held-recipient routing
//! (jig/docs/design/domain/engine.md, 4.4).

use crate::flight::Flight;
use crate::routing::children::append_people;
use crate::routing::runs::call_decision;
use crate::{CallPart, Core, CoreRecord, EscalationDecisionRecord, Limits, Now, Record, Request, RoutedCall, Write};
use alloc::boxed::Box;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Env, Queue, ReplyTo};

pub(super) fn escalation_decided(
    core: &mut Core,
    env: &Env<Limits>,
    reply_to: ReplyTo,
    task: u64,
    revision: u64,
    outcome: tasks::EscalationOutcome,
    out: &mut Queue<Request>,
) {
    let token = reply_to.into_token();
    let flight = match core.land(token) {
        Some(Flight::PersonEscalation(flight)) => flight,
        Some(Flight::Call {
            request,
            route: RoutedCall::Escalation { key, task: named, revision: current, passed_up },
            ..
        }) => {
            assert!(task == named && revision == current, "exact task escalation terminal");
            call_decision(
                core,
                out,
                ReplyTo::new(request),
                key,
                match outcome {
                    tasks::EscalationOutcome::Passed { .. } if passed_up => CallPart::PassedUp { task },
                    tasks::EscalationOutcome::Released
                    | tasks::EscalationOutcome::Rejected
                    | tasks::EscalationOutcome::Passed { .. }
                    | tasks::EscalationOutcome::Stale
                    | tasks::EscalationOutcome::NoFurther
                    | tasks::EscalationOutcome::Limit => CallPart::EscalationDecided { task, revision, outcome },
                },
            );
            return;
        }
        _ => return,
    };
    assert!(flight.task == task && flight.revision == revision, "exact held choice terminal");
    let choice = match outcome {
        tasks::EscalationOutcome::Released => people::EscalationChoice::Released,
        tasks::EscalationOutcome::Rejected => people::EscalationChoice::Rejected,
        tasks::EscalationOutcome::Passed { .. } => people::EscalationChoice::Passed,
        tasks::EscalationOutcome::NoFurther => {
            append_people(
                core,
                env,
                people::Event::Decided {
                    request: flight.request,
                    outcome: people::Outcome::Refused(people::Refusal::NoFurther),
                },
                out,
            );
            return;
        }
        tasks::EscalationOutcome::Limit => {
            append_people(
                core,
                env,
                people::Event::Decided {
                    request: flight.request,
                    outcome: people::Outcome::Refused(people::Refusal::Limit { which: people::LimitKind::Words }),
                },
                out,
            );
            return;
        }
        tasks::EscalationOutcome::Stale => {
            out.push(Request::Now(Box::new(Now::HistoricalEscalation {
                request: flight.request,
                person: flight.person,
                project: flight.project,
                task,
                revision,
            })));
            return;
        }
    };
    out.push(Request::Write(Write::Save(Record::Core(CoreRecord::EscalationDecision(EscalationDecisionRecord {
        project: flight.project,
        requester: flight.requester,
        task,
        revision,
        by: flight.person,
        at: env.wall,
        decision: flight.decision,
    })))));
    append_people(
        core,
        env,
        people::Event::Decided {
            request: flight.request,
            outcome: people::Outcome::EscalationDecided { task, revision, by: flight.person, at: env.wall, choice },
        },
        out,
    );
}

pub(super) fn escalation_needed(
    core: &mut Core,
    env: &Env<Limits>,
    context: Box<tasks::EscalationContext>,
    pending: &mut Queue<tasks::Request>,
    out: &mut Queue<Request>,
) {
    let mut follow = Queue::uncapped();
    if !core.route_waiting_escalation(env, &context, &mut follow) {
        out.push(Request::Now(Box::new(Now::RestoreRefused)));
        return;
    }
    for _ in 0..follow.len() {
        pending.push(follow.pop().expect("escalation output count"));
    }
}

pub(super) fn escalation_stalled(
    core: &mut Core,
    env: &Env<Limits>,
    task: u64,
    revision: u64,
    holder: tasks::EscalationHolder,
    pending: &mut Queue<tasks::Request>,
    out: &mut Queue<Request>,
) {
    let Some(context) = core.tasks.escalation(task) else { return };
    let tasks::Escalation::Waiting { revision: current, holder: old, .. } = context.escalation else {
        return;
    };
    if current != revision || old != holder {
        return;
    }
    let role = core.people.role(context.requester, context.project);
    let Some(next) = core.escalation_recipient_after(&env.limits, &context, role, Some(holder)) else {
        return;
    };
    let Some(entry) = crate::fresh(&mut core.counters, crate::Family::Message) else {
        out.push(Request::Now(Box::new(Now::RestoreRefused)));
        return;
    };
    let mut follow = Queue::uncapped();
    crate::routing::step::step_tasks(
        core,
        env,
        tasks::Event::RoutedEscalation { task, revision, holder: next, entry },
        &mut follow,
    );
    for _ in 0..follow.len() {
        pending.push(follow.pop().expect("stalled escalation output count"));
    }
}
