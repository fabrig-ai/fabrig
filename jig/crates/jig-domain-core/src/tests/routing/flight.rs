//! Task replies preserve their caller and choose one continuation.
use crate::{
    routing::flight::{Flight, Internal},
    tests::*,
};

fn outputs(core: &mut Core, limits: &Limits, request: tasks::Request) -> Queue<Request> {
    let mut child = Queue::uncapped();
    child.push(request);
    let mut out = Queue::uncapped();
    crate::routing::start_task_outputs(
        core,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits },
        child,
        &mut out,
    );
    out
}

fn pending(core: &mut Core, limits: &Limits) -> Token {
    let identity = core.people.identity(1).unwrap().clone();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.people };
    let mut out = Queue::uncapped();
    people::step(
        &mut core.people,
        &env,
        people::Event::SignedIn {
            held: None,
            reply_to: ReplyTo::new(Token::new(76)),
            person: 1,
            sign_in: 9,
            identity,
            kind: people::Kind::Person,
        },
        &mut out,
    );
    people::step(
        &mut core.people,
        &env,
        people::Event::Ask {
            models: Box::new([]),
            reply_to: ReplyTo::new(Token::new(77)),
            sign_in: 9,
            key: [1; 16],
            ask: people::Ask::Stop { project: 1, task: 1 },
        },
        &mut out,
    );
    while let Some(row) = out.pop() {
        if let people::Request::Route { request, .. } = row {
            return core.fly(Flight::Party { request });
        }
    }
    panic!("one pending party request")
}

fn outcome(out: &Queue<Request>) -> people::Outcome {
    for row in out {
        if let Request::Held(held) = row
            && let Held::PeopleReply { to, reply: people::Reply::Outcome(outcome) } = &**held
        {
            assert_eq!(*to, ReplyTo::new(Token::new(77)));
            return outcome.clone();
        }
    }
    panic!("one party outcome: {out:?}")
}

fn key() -> CallKey {
    CallKey { task: 1, attempt: 1, completion: 1, position: 0 }
}

#[test]
fn an_unknown_done_is_ignored_and_an_unknown_refusal_returns_the_payload() {
    let (mut core, limits) = core();
    let token = Token::new(u64::MAX - 10);
    assert!(outputs(&mut core, &limits, tasks::Request::Done { reply_to: ReplyTo::new(token) }).is_empty());
    let problem = tasks::Problem::new(Some(1), tasks::Refusal::Unknown);
    let mut out = outputs(
        &mut core,
        &limits,
        tasks::Request::Refused { reply_to: ReplyTo::new(token), problem: problem.clone() },
    );
    let Some(Request::Now(now)) = out.pop() else { panic!("payload refusal") };
    let Now::RefusedPayload { request, problem: found } = *now else { panic!("unknown payload refused") };
    assert_eq!(request, token);
    assert_eq!(found, problem);
    assert!(out.is_empty());
    assert!(core.flights.is_empty());
}

#[test]
fn a_stale_reply_cannot_continue_the_flight_that_reuses_its_slot() {
    let (mut core, limits) = core();
    let stale = core.internal(Internal::Admission);
    assert!(outputs(&mut core, &limits, tasks::Request::Done { reply_to: ReplyTo::new(stale) }).is_empty());
    let current = core.fly_call(Token::new(99), RoutedCall::Introduce(key()));
    assert_ne!(stale, current);
    assert!(outputs(&mut core, &limits, tasks::Request::Done { reply_to: ReplyTo::new(stale) }).is_empty());
    assert_eq!(core.flights.len(), 1);
    let mut out = outputs(&mut core, &limits, tasks::Request::Done { reply_to: ReplyTo::new(current) });
    let Some(Request::Held(held)) = out.pop() else { panic!("named answer") };
    let Held::CallAnswer { to, key: named, part } = *held else { panic!("call continuation") };
    assert_eq!(to, ReplyTo::new(Token::new(99)));
    assert_eq!(named, key());
    assert_eq!(part, CallPart::Introduced);
    assert!(core.flights.is_empty());
}

#[test]
fn person_task_move_and_make_replies_return_the_original_keyed_outcome() {
    for kind in 0_u8..3 {
        let (mut core, limits) = core();
        let request = pending(&mut core, &limits);
        let (flight, expected) = match kind {
            0 => (
                Flight::PersonTask { request, route: PersonTaskRoute::Take(4) },
                people::Outcome::PersonTaken { task: 4 },
            ),
            1 => (Flight::Moving { request, task: 5 }, people::Outcome::Moved { task: 5 }),
            _ => (
                Flight::Made { request, route: crate::routing::Made { first: 6, given: None, stubs: Box::new([]) } },
                people::Outcome::Created { first: 6, last: 7, given: None },
            ),
        };
        let token = core.fly(flight);
        let reply = if kind == 2 {
            tasks::Request::Made { reply_to: ReplyTo::new(token), tasks: Box::new([6, 7]) }
        } else {
            tasks::Request::Done { reply_to: ReplyTo::new(token) }
        };
        assert_eq!(outcome(&outputs(&mut core, &limits, reply)), expected);
        assert!(core.flights.is_empty());
    }
}

#[test]
fn person_origin_and_task_origin_proposals_choose_their_own_party_reply() {
    for task_origin in [false, true] {
        let (mut core, limits) = core();
        let request = pending(&mut core, &limits);
        let (token, reply, proposer) = if task_origin {
            let token = core.fly_person_proposal(PersonProposalRoute::Deciding {
                request,
                proposer: 1,
                proposal: 3,
                by: 1,
                moved: None,
            });
            (token, true, Proposer::Task(1))
        } else {
            let token = core.fly_goal(request, GoalRoute::Deciding { proposer: 1, proposal: 3, by: 1 });
            (token, false, Proposer::Party(1))
        };
        let reply = if reply {
            tasks::Request::ProposalDecided {
                reply_to: ReplyTo::new(token),
                proposer: 1,
                number: 3,
                outcome: tasks::ProposalOutcome::Accepted,
            }
        } else {
            tasks::Request::PersonProposalDecided {
                reply_to: Some(ReplyTo::new(token)),
                proposer: 1,
                number: 3,
                outcome: tasks::ProposalOutcome::Accepted,
            }
        };
        assert_eq!(
            outcome(&outputs(&mut core, &limits, reply)),
            people::Outcome::ProposalDecided {
                proposer,
                proposal: 3,
                by: 1,
                at: Wall::EPOCH,
                choice: people::ProposalChoice::Accepted,
                repeated: false,
            }
        );
        assert!(core.flights.is_empty());
    }
}

#[test]
fn an_effect_proposal_refusal_drops_the_connector_owner_before_answering_the_call() {
    let (mut core, limits) = core();
    let token = core.fly(Flight::Call {
        request: Token::new(99),
        route: RoutedCall::Propose { key: key(), proposal: 3 },
        effect: Some((1, Token::new(88))),
    });
    let problem = tasks::Problem::new(Some(1), tasks::Refusal::Unknown);
    let mut out = outputs(
        &mut core,
        &limits,
        tasks::Request::Refused { reply_to: ReplyTo::new(token), problem: problem.clone() },
    );
    let Some(Request::Ask { connector: 1, ask: Ask::Effect(connector::Ask::Drop { owner, .. }) }) = out.pop() else {
        panic!("effect dropped first")
    };
    assert_eq!(owner, Token::new(88));
    let Some(Request::Held(held)) = out.pop() else { panic!("call refused next") };
    let Held::CallAnswer { to, part, .. } = *held else { panic!("named proposal refusal") };
    assert_eq!(to, ReplyTo::new(Token::new(99)));
    assert_eq!(part, CallPart::ProposalRefused(problem));
    assert!(out.is_empty());
    assert!(core.flights.is_empty());
}

#[test]
fn a_procedure_proposal_refusal_uses_its_owner_and_retires_the_followup() {
    let (mut core, limits) = core();
    let token = core.fly(Flight::ProcedureProposal {
        owner: Token::new(88),
        route: crate::effects::ProcedureProposal { task: 1, step: 2, connector: 1, proposal: 3 },
    });
    let mut out = outputs(
        &mut core,
        &limits,
        tasks::Request::Refused {
            reply_to: ReplyTo::new(token),
            problem: tasks::Problem::new(Some(1), tasks::Refusal::Unknown),
        },
    );
    let Some(Request::Ask { connector: 1, ask: Ask::Effect(connector::Ask::Drop { owner, .. }) }) = out.pop() else {
        panic!("procedure owner refused")
    };
    assert_eq!(owner, Token::new(88));
    assert!(core.flights.is_empty());
}

#[test]
fn person_and_task_escalations_choose_their_own_terminal() {
    let (mut core, limits) = core();
    let request = pending(&mut core, &limits);
    let token = core.fly(Flight::PersonEscalation(crate::routing::PersonEscalation {
        request,
        requester: 1,
        person: 1,
        project: 1,
        task: 4,
        revision: 2,
        decision: people::EscalationDecision::Release,
    }));
    let out = outputs(
        &mut core,
        &limits,
        tasks::Request::EscalationDecided {
            reply_to: ReplyTo::new(token),
            task: 4,
            revision: 2,
            outcome: tasks::EscalationOutcome::NoFurther,
        },
    );
    assert_eq!(outcome(&out), people::Outcome::Refused(people::Refusal::NoFurther));
    let token =
        core.fly_call(Token::new(99), RoutedCall::Escalation { key: key(), task: 4, revision: 2, passed_up: true });
    let mut out = outputs(
        &mut core,
        &limits,
        tasks::Request::EscalationDecided {
            reply_to: ReplyTo::new(token),
            task: 4,
            revision: 2,
            outcome: tasks::EscalationOutcome::Passed { revision: 3 },
        },
    );
    let Some(Request::Held(held)) = out.pop() else { panic!("task escalation answer") };
    let Held::CallAnswer { to, part, .. } = *held else { panic!("task escalation continuation") };
    assert_eq!(to, ReplyTo::new(Token::new(99)));
    assert_eq!(part, CallPart::PassedUp { task: 4 });
    assert!(core.flights.is_empty());
}

#[test]
fn internal_repair_and_delegate_refusals_notify_each_connector_in_order() {
    for repair in [false, true] {
        let (mut core, limits) = core();
        let token = core.internal(if repair { Internal::Repair } else { Internal::Delegate });
        let mut out = outputs(
            &mut core,
            &limits,
            tasks::Request::Refused {
                reply_to: ReplyTo::new(token),
                problem: tasks::Problem::new(Some(4), tasks::Refusal::Unknown),
            },
        );
        for connector in core.connectors.as_ref() {
            let Some(Request::Ask { connector: named, ask }) = out.pop() else { panic!("connector refusal") };
            assert_eq!(named, *connector);
            if repair {
                let Ask::RepairRefused { repair } = ask else { panic!("repair refused") };
                assert_eq!(repair, Some(4));
            } else {
                let Ask::DelegateRefused { task } = ask else { panic!("delegate refused") };
                assert_eq!(task, Some(4));
            }
        }
        assert!(out.is_empty());
        assert!(core.flights.is_empty());
    }
}

#[test]
fn an_ended_load_resumes_its_party_and_a_stale_load_leaves_the_next_read_pending() {
    let (mut core, limits) = core();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let request = pending(&mut core, &limits);
    let stale = core.fly(Flight::ResultRead(crate::routing::run::results::ResultRead {
        why: crate::routing::run::results::ReadFor::DeploymentClose { request, project: 1 },
        ids: Box::new([12]),
        at: 0,
        results: List::with_capacity(0),
    }));
    let mut out = step(&mut core, &env, Event::EndedTaskLoaded { request: stale, row: None, busy: true });
    let Some(Request::Held(held)) = out.pop() else { panic!("load answer") };
    let Held::PeopleReply { to, reply } = *held else { panic!("party load answer") };
    assert_eq!(to, ReplyTo::new(Token::new(77)));
    assert_eq!(reply, people::Reply::Again(people::Again::Busy));
    let current = core.fly(Flight::ResultRead(crate::routing::run::results::ResultRead {
        why: crate::routing::run::results::ReadFor::Preparation { task: 1, last_message: 0 },
        ids: Box::new([13]),
        at: 0,
        results: List::with_capacity(0),
    }));
    assert_ne!(stale, current);
    assert!(step(&mut core, &env, Event::EndedTaskLoaded { request: stale, row: None, busy: true }).is_empty());
    assert!(core.flights.get(skein_lib::Id::from_token(current)).is_some());
    drop(core.take_result_read(current));
    assert!(core.flights.is_empty());
}

#[test]
fn a_refused_staged_effect_releases_its_owner_and_late_descriptions_leave_the_next_owner_untouched() {
    let (mut core, limits) = core();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let owner = core.effect_owner();
    assert!(!core.quiescent(true));
    let mut out = step(
        &mut core,
        &env,
        Event::EffectStart { owner, connector: 1, origin: EffectOrigin::Projection { goal: 99, entry: None } },
    );
    let Some(Request::Ask { connector: 1, ask: Ask::Effect(connector::Ask::Drop { owner: dropped, .. }) }) = out.pop()
    else {
        panic!("staged effect dropped")
    };
    assert_eq!(dropped, owner);
    assert!(core.flights.is_empty());
    let current = core.effect_owner();
    assert_ne!(owner, current);
    assert!(step(&mut core, &env, Event::EffectConnector(connector::Event::DescribeBusy { owner })).is_empty());
    assert!(core.flights.get(skein_lib::Id::from_token(current)).is_some());
    drop(core.land_kind(current, flight::Kind::EffectStaging));
    assert!(core.flights.is_empty());
}

#[test]
fn an_application_part_uses_the_people_childs_request_when_its_core_handle_differs() {
    let (mut core, limits) = core();
    let waiting = pending(&mut core, &limits);
    let staged = core.effect_owner();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut out = step(
        &mut core,
        &env,
        Event::Application {
            reply_to: ReplyTo::new(Token::new(91)),
            sign_in: 9,
            key: [2; 16],
            scope: people::ApplicationScope::Own,
            connector: 1,
            digest: [3; 32],
            owner: Token::new(92),
            carried: None,
        },
    );
    let mut admitted = None;
    while let Some(row) = out.pop() {
        if let Request::Ask { ask: Ask::Application { request, token, .. }, .. } = row {
            admitted = Some((request, token));
        }
    }
    let (request, token) = admitted.expect("application admitted");
    for through in [0, 1] {
        let seen = people::Seen::Results { through };
        let mut part =
            step(&mut core, &env, Event::ApplicationPart { request, ask: Box::new(people::Ask::MarkSeen { seen }) });
        let Some(Request::Ask { connector: 1, ask: Ask::ApplicationPartAnswered { request: found, outcome } }) =
            part.pop()
        else {
            panic!("application part answer")
        };
        assert_eq!(found, request);
        assert_eq!(outcome, people::Outcome::MarkedSeen { seen });
        assert!(part.is_empty());
    }
    let mut out = step(&mut core, &env, Event::ApplicationFinished { request, outcome: Ok(token) });
    assert!(out.pop().is_some());
    assert!(core.application_mut(request).is_none());
    assert!(
        step(
            &mut core,
            &env,
            Event::ApplicationPart {
                request,
                ask: Box::new(people::Ask::MarkSeen { seen: people::Seen::Results { through: 2 } }),
            }
        )
        .is_empty()
    );
    drop(core.land_kind(waiting, flight::Kind::Party));
    drop(core.land_kind(staged, flight::Kind::EffectStaging));
    assert!(core.flights.is_empty());
}

#[test]
fn an_adoption_part_names_its_parent_until_its_own_answer_arrives() {
    let (mut core, limits) = core();
    let waiting = pending(&mut core, &limits);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut out = step(
        &mut core,
        &env,
        Event::Application {
            reply_to: ReplyTo::new(Token::new(91)),
            sign_in: 9,
            key: [2; 16],
            scope: people::ApplicationScope::Project(1),
            connector: 1,
            digest: [3; 32],
            owner: Token::new(92),
            carried: None,
        },
    );
    let mut admitted = None;
    while let Some(row) = out.pop() {
        if let Request::Ask { ask: Ask::Application { request, token, .. }, .. } = row {
            admitted = Some((request, token));
        }
    }
    let (application, token) = admitted.expect("application admitted");
    let mut out = step(
        &mut core,
        &env,
        Event::ApplicationPart {
            request: application,
            ask: Box::new(people::Ask::Adopt {
                project: 1,
                adoption: people::Adoption {
                    resource: people::ResourceName { connector: 1, path: Box::new([Box::from(&b"resource"[..])]) },
                    role: people::ResourceRole::Context,
                    options: Box::new([]),
                },
            }),
        },
    );
    let first = out.pop();
    let Some(Request::Ask { ask: Ask::Adopt { request, .. }, .. }) = first else {
        panic!("adoption part: {first:?}; {out:?}")
    };
    assert_ne!(request, application);
    assert_eq!(core.application_part_request(request), Some(application));
    assert_eq!(core.application_part_request(application), None);
    assert_eq!(core.application_part_request(waiting), None);
    let mut out = step(
        &mut core,
        &env,
        Event::People(people::Event::Decided { request, outcome: people::Outcome::Adopted { project: 1 } }),
    );
    let Some(Request::Ask { ask: Ask::ApplicationPartAnswered { request: parent, outcome }, .. }) = out.pop() else {
        panic!("part answer names its application")
    };
    assert_eq!(parent, application);
    assert_eq!(outcome, people::Outcome::Adopted { project: 1 });
    assert_eq!(core.application_part_request(request), None);
    drop(step(&mut core, &env, Event::ApplicationFinished { request: application, outcome: Ok(token) }));
    let current = core.effect_owner();
    assert_ne!(current, request);
    assert_eq!(core.application_part_request(request), None);
    drop(core.land_kind(current, crate::flight::Kind::EffectStaging));
    drop(core.land_kind(waiting, crate::flight::Kind::Party));
    assert!(core.flights.is_empty());
}
