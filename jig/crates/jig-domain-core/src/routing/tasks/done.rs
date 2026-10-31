//! Successful task continuations and their named-call terminals
//! (jig/docs/design/domain/engine.md, 4.4).

use crate::flight::Flight;
use crate::routing::children::append_people;
use crate::routing::runs::call_decision;
use crate::{Ask, CallPart, Core, GoalRoute, Limits, PersonProposalRoute, PersonTaskRoute, Request, RoutedCall};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Env, Queue, ReplyTo, Token};

pub(super) fn done(
    core: &mut Core,
    env: &Env<Limits>,
    reply_to: ReplyTo,
    pending: &mut Queue<tasks::Request>,
    out: &mut Queue<Request>,
) {
    let token = reply_to.into_token();
    match core.land_task(token) {
        Some(flight) => complete(core, env, flight, pending, out),
        None => {
            if let Some(attempt) = core.claiming.remove(&token.raw()) {
                core.claim_done(token.raw(), attempt, out);
            }
        }
    }
}

pub(super) fn complete(
    core: &mut Core,
    env: &Env<Limits>,
    flight: Flight,
    pending: &mut Queue<tasks::Request>,
    out: &mut Queue<Request>,
) {
    match flight {
        Flight::Internal(crate::flight::Internal::Deletion { project }) => {
            crate::deletion::compound_done(core, project);
        }
        Flight::PersonTask { request, route } => {
            let outcome = match route {
                PersonTaskRoute::Compound { project, operations, first_message } => {
                    people::Outcome::Compound { project, operations, first_message }
                }
                PersonTaskRoute::Take(task) => people::Outcome::PersonTaken { task },
                PersonTaskRoute::PoolSet { project, person } => people::Outcome::PoolSet { project, person },
                PersonTaskRoute::HandBack(task) => people::Outcome::PersonHandedBack { task },
                PersonTaskRoute::Answer(task) => people::Outcome::PersonAnswered { task },
                PersonTaskRoute::Stop(task) => people::Outcome::Stopped { task },
                PersonTaskRoute::Cancel(task) => people::Outcome::Cancelled { task },
                PersonTaskRoute::Release(task) => people::Outcome::Released { task },
                PersonTaskRoute::Amended { task, released } => people::Outcome::Amended { task, released },
                PersonTaskRoute::Lift(_) => return,
            };
            append_people(core, env, people::Event::Decided { request, outcome }, out);
        }
        Flight::Moving { request, task } => {
            append_people(core, env, people::Event::Decided { request, outcome: people::Outcome::Moved { task } }, out);
        }
        Flight::ProcedureProposal { owner, route } => {
            let event = crate::effects::procedure_proposed(core, out, owner, route, true);
            super::follow(core, env, event, pending);
        }
        Flight::Goal { request, route: GoalRoute::Accepting { proposer, proposal, by } } => {
            let token = core.fly_goal(request, GoalRoute::Deciding { proposer, proposal, by });
            super::follow(
                core,
                env,
                tasks::Event::DecidePersonProposal {
                    reply_to: ReplyTo::new(token),
                    proposer,
                    proposal,
                    by: tasks::Party::Person(by),
                    message: None,
                    decision: tasks::ProposalDecision::Accept,
                    revision: None,
                },
                pending,
            );
        }
        Flight::PersonProposal(PersonProposalRoute::Accepting { request, person, proposer, proposal, message }) => {
            let token = core.fly_person_proposal(PersonProposalRoute::Deciding {
                request,
                proposer,
                proposal,
                by: person,
                moved: None,
            });
            super::follow(
                core,
                env,
                tasks::Event::DecideProposal {
                    reply_to: ReplyTo::new(token),
                    proposer,
                    proposal,
                    message: Some(message),
                    by: tasks::Party::Person(person),
                    decision: tasks::ProposalDecision::Accept,
                },
                pending,
            );
        }
        Flight::Call { request, route, effect } => call_done(core, env, request, route, effect, pending, out),
        Flight::Goal { .. }
        | Flight::PersonProposal(_)
        | Flight::Made { .. }
        | Flight::PersonEscalation(_)
        | Flight::Internal(_)
        | Flight::Delegating { .. }
        | Flight::Saying { .. }
        | Flight::Party { .. }
        | Flight::PartyPart { .. }
        | Flight::EndedMark { .. }
        | Flight::Key { .. }
        | Flight::Application { .. }
        | Flight::Watch { .. }
        | Flight::ResultRead(_)
        | Flight::CreatingInputs { .. }
        | Flight::Creating { .. }
        | Flight::AwaitingResources(_)
        | Flight::CreatingDelegates { .. }
        | Flight::EffectStaging
        | Flight::Effect { .. }
        | Flight::Note(_)
        | Flight::Workspace { .. }
        | Flight::ProcedureCreation { .. }
        | Flight::SettingsOffering { .. }
        | Flight::Retired => {}
    }
}

fn call_done(
    core: &mut Core,
    env: &Env<Limits>,
    request: Token,
    route: RoutedCall,
    effect: Option<(u16, Token)>,
    pending: &mut Queue<tasks::Request>,
    out: &mut Queue<Request>,
) {
    let (key, part) = match route {
        RoutedCall::Propose { key, proposal } => {
            if let Some((connector, owner)) = effect {
                out.push(Request::Ask {
                    connector,
                    ask: Ask::Effect(crate::connector::Ask::KeepProposal {
                        owner,
                        proposal,
                        task: key.task,
                        project: core.tasks.task(key.task).expect("proposing task exists").project,
                    }),
                });
            }
            let tasks::ProposalState::Pending { holder, .. } =
                core.tasks.proposal(key.task, proposal).expect("admitted proposal").state
            else {
                return;
            };
            (key, CallPart::Proposed { proposal, holder })
        }
        RoutedCall::Introduce(key) => (key, CallPart::Introduced),
        RoutedCall::Control { key, target, revised } => {
            let part = if revised {
                let revision = core.tasks.task(target).expect("controlled task exists").revisions.spec;
                CallPart::Revised { task: target, revision }
            } else {
                CallPart::Controlled
            };
            (key, part)
        }
        RoutedCall::Accepting { key, proposer, proposal, message } => {
            let token = core.fly_call(request, RoutedCall::Decide { key, proposal });
            super::follow(
                core,
                env,
                tasks::Event::DecideProposal {
                    reply_to: ReplyTo::new(token),
                    proposer,
                    proposal,
                    message: Some(message),
                    by: tasks::Party::Task(key.task),
                    decision: tasks::ProposalDecision::Accept,
                },
                pending,
            );
            return;
        }
        RoutedCall::Subscribe { key, subscription } => {
            for &connector in core.connectors.as_ref() {
                out.push(Request::Ask { connector, ask: Ask::SubscriptionDone { request, key, subscription } });
            }
            return;
        }
        RoutedCall::Unsubscribe(key) => {
            for &connector in core.connectors.as_ref() {
                out.push(Request::Ask { connector, ask: Ask::UnsubscriptionDone { request, key } });
            }
            return;
        }
        RoutedCall::Message(_)
        | RoutedCall::Decide { .. }
        | RoutedCall::Withdraw { .. }
        | RoutedCall::Escalation { .. } => return,
    };
    call_decision(core, out, ReplyTo::new(request), key, part);
}
