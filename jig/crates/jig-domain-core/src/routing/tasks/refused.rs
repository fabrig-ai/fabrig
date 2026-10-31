//! Refused task continuations and their party or named-call answers
//! (jig/docs/design/domain/engine.md, 4.4).

use crate::flight::{Flight, Internal};
use crate::routing::children::append_people;
use crate::routing::refusals::{person_message_refusal, person_move_refusal, person_problem_refusal};
use crate::routing::runs::call_decision;
use crate::{Ask, CallPart, Core, Limits, Now, PersonProposalRoute, PersonTaskRoute, Request, RoutedCall};
use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Env, Queue, ReplyTo, Token};

pub(super) fn procedure_refused(
    core: &mut Core,
    task: u64,
    step: u64,
    problem: tasks::Problem,
    out: &mut Queue<Request>,
) {
    super::super::procedure_steps::append_problem(core, task, step, problem, out);
}

pub(super) fn compound_refused(
    core: &mut Core,
    env: &Env<Limits>,
    reply_to: ReplyTo,
    at: u32,
    problem: tasks::Problem,
    out: &mut Queue<Request>,
) {
    let request = reply_to.into_token();
    let Some(Flight::PersonTask { request, route: PersonTaskRoute::Compound { .. } }) = core.land(request) else {
        return;
    };
    let outcome = person_problem_refusal(core, problem);
    let outcome = if let people::Outcome::Refused(why) = outcome {
        people::Outcome::CompoundRefused { at, why }
    } else {
        outcome
    };
    append_people(core, env, people::Event::Decided { request, outcome }, out);
}

pub(super) fn refused(
    core: &mut Core,
    env: &Env<Limits>,
    reply_to: ReplyTo,
    problem: tasks::Problem,
    pending: &mut Queue<tasks::Request>,
    out: &mut Queue<Request>,
) {
    let token = reply_to.into_token();
    if let Some(flight) = core.land(token) {
        refuse_flight(core, env, token, flight, problem, pending, out);
        return;
    }
    if let Some((key, _stubs, _given)) = core.delegating.remove(&token) {
        call_decision(core, out, ReplyTo::new(token), key, CallPart::DelegationRefused(problem));
        return;
    }
    if core.saying.remove(&token).is_some() {
        append_people(
            core,
            env,
            people::Event::Decided { request: token, outcome: person_message_refusal(core, problem) },
            out,
        );
        return;
    }
    if core.claiming.remove(&token.raw()).is_some() {
        core.restore_replaced_proof(token.raw());
        out.push(Request::Now(Box::new(Now::DropAssignment { task: token.raw() })));
        let mut follow = Queue::uncapped();
        crate::routing::step::step_tasks(core, env, tasks::Event::PreparationFailed { task: token.raw() }, &mut follow);
        for _ in 0..follow.len() {
            pending.push(follow.pop().expect("failed preparation output count"));
        }
        return;
    }
    out.push(Request::Now(Box::new(Now::RefusedPayload { request: token, problem })));
}

fn refuse_flight(
    core: &mut Core,
    env: &Env<Limits>,
    token: Token,
    flight: Flight,
    problem: tasks::Problem,
    pending: &mut Queue<tasks::Request>,
    out: &mut Queue<Request>,
) {
    match flight {
        Flight::Goal { request, .. } | Flight::PersonTask { request, .. } | Flight::Made { request, .. } => {
            if match &core.settings_offering {
                Some((token, ..)) => *token == request,
                None => false,
            } {
                drop(core.settings_offering.take());
            }
            let outcome = person_problem_refusal(core, problem);
            append_people(core, env, people::Event::Decided { request, outcome }, out);
        }
        Flight::Moving { request, .. } => {
            let outcome = person_move_refusal(core, problem);
            append_people(core, env, people::Event::Decided { request, outcome }, out);
        }
        Flight::PersonProposal(route) => {
            let request = match route {
                PersonProposalRoute::Deciding { request, .. } | PersonProposalRoute::Accepting { request, .. } => {
                    request
                }
            };
            let outcome = person_move_refusal(core, problem);
            append_people(core, env, people::Event::Decided { request, outcome }, out);
        }
        Flight::ProcedureProposal { owner, route } => {
            let event = crate::effects::procedure_proposed(core, out, owner, route, false);
            super::follow(core, env, event, pending);
        }
        Flight::Call { request, route, effect } => {
            if let Some((connector, owner)) = effect {
                out.push(Request::Ask {
                    connector,
                    ask: Ask::Effect(crate::connector::Ask::Drop {
                        owner,
                        answer: authority::Answer::Refuse,
                        findings: Box::new([]),
                        why: crate::connector::EffectRefusal::Proposal,
                    }),
                });
            }
            let (key, part) = match route {
                RoutedCall::Message(key) | RoutedCall::Introduce(key) => (key, CallPart::MessageRefused(problem)),
                RoutedCall::Control { key, target, .. } => {
                    let part = if let tasks::Refusal::HoldOwner { hold } = problem.why {
                        let row = core.tasks.placed_hold(hold).expect("owner refusal names retained hold");
                        CallPart::HoldOwner {
                            task: target,
                            hold,
                            revision: row.revision,
                            owner: crate::roles::hold_owner(row.owner),
                        }
                    } else {
                        CallPart::ControlRefused(problem)
                    };
                    (key, part)
                }
                RoutedCall::Escalation { key, .. } => (key, CallPart::EscalationRefused(problem)),
                RoutedCall::Propose { key, .. }
                | RoutedCall::Decide { key, .. }
                | RoutedCall::Withdraw { key, .. }
                | RoutedCall::Accepting { key, .. } => (key, CallPart::ProposalRefused(problem)),
                RoutedCall::Subscribe { key, .. } | RoutedCall::Unsubscribe(key) => {
                    call_decision(core, out, ReplyTo::new(request), key, CallPart::SubscriptionRefused(problem));
                    for &connector in core.connectors.as_ref() {
                        out.push(Request::Ask { connector, ask: Ask::DropSubscription { request } });
                    }
                    return;
                }
            };
            call_decision(core, out, ReplyTo::new(request), key, part);
        }
        Flight::Internal(Internal::Repair) => {
            for &connector in core.connectors.as_ref() {
                out.push(Request::Ask { connector, ask: Ask::RepairRefused { repair: problem.task } });
            }
        }
        Flight::Internal(Internal::Delegate | Internal::Acknowledge) => {
            for &connector in core.connectors.as_ref() {
                out.push(Request::Ask { connector, ask: Ask::DelegateRefused { task: problem.task } });
            }
        }
        Flight::Internal(Internal::Admission) => {
            out.push(Request::Now(Box::new(Now::RefusedPayload { request: token, problem })));
        }
        Flight::Retired | Flight::PersonEscalation(_) => {}
    }
}
