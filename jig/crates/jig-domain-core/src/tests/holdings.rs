//! A member's symbols and connector refusals (domain/tasks.md, 6.1 and 6.2).
use super::{
    fill::{ask, filled_core, goal, signed_in},
    *,
};

const OWNER: u64 = 1;
const CHILD: u64 = 2;
const HOLDER: u64 = 7;
const FIRST_CONNECTOR: u16 = 1;
const SYMBOL_CONNECTOR: u16 = 2;
const UNKNOWN_CONNECTOR: u16 = 3;
const OWN_RESOURCE: u16 = 1;
const SECOND_RESOURCE: u16 = 2;
// The fixture configures two connectors, each asked once per member.
const CONNECTORS: usize = 2;

fn party_core() -> (Core, Env<Limits>) {
    let (mut core, env) = filled_core(0);
    signed_in(&mut core, &env);
    (core, env)
}

fn symbolic_goal(connector: u16) -> people::Ask {
    let people::Ask::Create { project, mut batch } = goal() else { panic!("goal creation") };
    batch[0].symbols = Box::new([people::Symbol { connector, symbol: OWN_RESOURCE }]);
    people::Ask::Create { project, batch }
}

fn holdings_request(mut out: Queue<Request>) -> Token {
    let mut asked = 0_usize;
    let mut request = None;
    while let Some(next) = out.pop() {
        if let Request::Ask { ask: Ask::TaskHoldings { request: token, .. }, .. } = next {
            request = Some(token);
            asked = asked.checked_add(1).unwrap();
        }
    }
    assert_eq!(asked, CONNECTORS);
    request.expect("a holdings request")
}

fn person_refusal(mut out: Queue<Request>, which: people::LimitKind) {
    let mut reply = None;
    while let Some(next) = out.pop() {
        match next {
            Request::Held(held) => {
                if let Held::PeopleReply { reply: value, .. } = *held {
                    assert!(reply.replace(value).is_none(), "one terminal answer");
                }
            }
            Request::Ask { .. } => panic!("a refusal asks no connector"),
            Request::Write(Write::Save(Record::Tasks(tasks::Stored::Live(_)))) => panic!("a refusal makes no task"),
            Request::ProjectMade { .. } | Request::Write(_) | Request::Now(_) => {}
        }
    }
    assert_eq!(reply, Some(people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Limit { which }))));
}

#[test]
fn a_members_symbols_reach_only_their_connector_with_whether_it_is_tracked() {
    let (mut core, env) = party_core();
    let requests = symbol_asks(ask(&mut core, &env, symbolic_goal(SYMBOL_CONNECTOR)));
    assert_eq!(
        requests.as_ref(),
        &[(FIRST_CONNECTOR, Box::from([]), true, OWNER), (SYMBOL_CONNECTOR, Box::from([OWN_RESOURCE]), true, OWNER),]
    );
}

#[test]
fn a_symbol_naming_no_connector_refuses_the_batch_before_any_ask() {
    let (mut core, env) = party_core();
    person_refusal(ask(&mut core, &env, symbolic_goal(UNKNOWN_CONNECTOR)), people::LimitKind::HoldKind);
    assert!(core.tasks.task(OWNER).is_none());
}

fn refused_person(refused: Option<HoldingsRefusal>, which: people::LimitKind) {
    let (mut core, env) = party_core();
    let request = holdings_request(ask(&mut core, &env, symbolic_goal(SYMBOL_CONNECTOR)));
    drop(step(&mut core, &env, Event::Holdings { request, connector: SYMBOL_CONNECTOR, holdings: None, refused }));
    person_refusal(
        step(
            &mut core,
            &env,
            Event::Holdings { request, connector: FIRST_CONNECTOR, holdings: Some(Box::new([])), refused: None },
        ),
        which,
    );
    assert!(core.tasks.task(OWNER).is_none());
}

#[test]
fn a_connector_refusing_a_symbol_refuses_the_persons_batch_and_makes_nothing() {
    refused_person(Some(HoldingsRefusal::Symbol { symbol: OWN_RESOURCE }), people::LimitKind::HoldKind);
}

#[test]
fn a_connector_naming_a_holder_refuses_the_persons_batch_without_naming_it() {
    refused_person(Some(HoldingsRefusal::Held { holder: HOLDER }), people::LimitKind::HoldTaken);
}

#[test]
fn an_unnamed_holdings_failure_keeps_the_persons_holds_refusal() {
    refused_person(None, people::LimitKind::Holds);
}

fn delegate(core: &Core) -> Delegate {
    let mut member = new_row(core, CHILD, tasks::Funder::Task(OWNER), None);
    member.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    Delegate {
        executor: member.executor,
        spec: member.spec,
        contract: member.contract,
        authority: member.authority,
        deadline: member.deadline,
        llm: None,
        symbolic_grants: Box::new([]),
        dependencies: Box::new([]),
        wake: member.wake,
        tracked: None,
        label: member.label,
        flags: member.flags,
        budget: people::Budget::Named,
    }
}

fn delegate_core() -> (Core, Env<Limits>) {
    let (mut core, env) = party_core();
    make_task(&mut core, &env.limits, OWNER);
    core.counters.deployment.tasks = OWNER;
    core.proofs.insert(OWNER, RunProof::claimed(OWNER, 1, 1, None, fleet::HostKind::Worker)).unwrap();
    (core, env)
}

fn refused_delegate(refused: Option<HoldingsRefusal>, expected: tasks::Problem) {
    let (mut core, env) = delegate_core();
    let batch = Box::new([delegate(&core)]);
    let key = CallKey { task: OWNER, attempt: 1, completion: 1, position: 0 };
    let mut out = step(
        &mut core,
        &env,
        Event::DelegateValidated { to: ReplyTo::new(Token::new(7300)), key, batch, stubs: Box::new([]) },
    );
    let mut token = None;
    let mut asks = 0_usize;
    while let Some(next) = out.pop() {
        if let Request::Ask { ask: Ask::DelegateHoldings { request, .. }, .. } = next {
            token = Some(request);
            asks = asks.checked_add(1).unwrap();
        }
    }
    assert_eq!(asks, CONNECTORS);
    let request = token.expect("delegation asks for holdings");
    drop(step(
        &mut core,
        &env,
        Event::DelegateHoldings { request, connector: SYMBOL_CONNECTOR, holdings: None, refused },
    ));
    let mut out = step(
        &mut core,
        &env,
        Event::DelegateHoldings {
            request,
            connector: FIRST_CONNECTOR,
            holdings: Some(Box::new([Box::new([])])),
            refused: None,
        },
    );
    let mut part = None;
    while let Some(next) = out.pop() {
        if let Request::Held(held) = next
            && let Held::CallAnswer { part: value, .. } = *held
        {
            assert!(part.replace(value).is_none(), "one terminal answer");
        }
    }
    assert_eq!(part, Some(CallPart::DelegationRefused(expected)));
    assert!(core.tasks.task(CHILD).is_none());
    assert!(core.tasks.task(OWNER).unwrap().delegates.is_empty());
}

#[test]
fn a_connector_naming_a_holder_refuses_a_delegate_batch_naming_it() {
    refused_delegate(
        Some(HoldingsRefusal::Held { holder: HOLDER }),
        tasks::Problem::naming(Some(CHILD), tasks::Refusal::HoldTaken, &[HOLDER]),
    );
}

#[test]
fn a_connector_refusing_a_symbol_names_the_delegate_member() {
    refused_delegate(
        Some(HoldingsRefusal::Symbol { symbol: OWN_RESOURCE }),
        tasks::Problem::new(Some(CHILD), tasks::Refusal::HoldKind),
    );
}

#[test]
fn an_unnamed_holdings_failure_keeps_the_delegate_holds_refusal() {
    refused_delegate(None, tasks::Problem::new(None, tasks::Refusal::Holds));
}

fn refused_procedure(refused: Option<HoldingsRefusal>, expected: tasks::Problem) {
    let (mut core, env) = party_core();
    let mut parent = new_row(&core, OWNER, tasks::Funder::Period { project: 1, period: 1 }, None);
    parent.executor = tasks::Executor::Procedure { connector: SYMBOL_CONNECTOR, code: 1 };
    send_task(
        &mut core,
        &env.limits,
        tasks::Event::Make {
            reply_to: ReplyTo::new(Token::new(7301)),
            creator: tasks::Party::Person(OWNER),
            batch: Box::new([parent]),
        },
    );
    core.counters.deployment.tasks = OWNER;
    let batch = Box::new([delegate(&core)]);
    let (_, _, step_number) = core.tasks.procedure_due(OWNER).expect("due procedure");
    let mut out = step(
        &mut core,
        &env,
        Event::ProcedureStep {
            task: OWNER,
            step: step_number,
            connector: SYMBOL_CONNECTOR,
            code: 1,
            decision: ProcedureStep { batch, ..ProcedureStep::new(StepEnd::Wait) },
        },
    );
    let mut asks = 0_usize;
    while let Some(next) = out.pop() {
        if let Request::Ask { ask: Ask::ProcedureHoldings { .. }, .. } = next {
            asks = asks.checked_add(1).unwrap();
        }
    }
    assert_eq!(asks, CONNECTORS);
    drop(step(
        &mut core,
        &env,
        Event::ProcedureHoldings {
            task: OWNER,
            step: step_number,
            connector: SYMBOL_CONNECTOR,
            holdings: None,
            refused,
        },
    ));
    let mut out = step(
        &mut core,
        &env,
        Event::ProcedureHoldings {
            task: OWNER,
            step: step_number,
            connector: FIRST_CONNECTOR,
            holdings: Some(Box::new([Box::new([])])),
            refused: None,
        },
    );
    let mut refusal = None;
    while let Some(next) = out.pop() {
        if let Request::Ask { connector: SYMBOL_CONNECTOR, ask: Ask::StepRefused { task, step, why } } = next {
            assert_eq!((task, step), (OWNER, step_number));
            assert!(refusal.replace(why).is_none(), "one step refusal");
        }
    }
    assert_eq!(refusal, Some(StepRefusal::Refused(expected)));
    assert!(core.tasks.task(CHILD).is_none());
    assert!(core.tasks.task(OWNER).unwrap().delegates.is_empty());
}

#[test]
fn a_connector_naming_a_holder_refuses_a_procedure_batch_naming_it() {
    refused_procedure(
        Some(HoldingsRefusal::Held { holder: HOLDER }),
        tasks::Problem::naming(Some(CHILD), tasks::Refusal::HoldTaken, &[HOLDER]),
    );
}

#[test]
fn a_connector_refusing_a_symbol_names_the_procedure_member() {
    refused_procedure(
        Some(HoldingsRefusal::Symbol { symbol: OWN_RESOURCE }),
        tasks::Problem::new(Some(CHILD), tasks::Refusal::HoldKind),
    );
}

#[test]
fn an_unnamed_holdings_failure_keeps_the_procedures_holds_refusal() {
    refused_procedure(None, tasks::Problem::new(Some(OWNER), tasks::Refusal::Holds));
}

type SymbolAsk = (u16, Box<[u16]>, bool, u64);

fn symbol_asks(mut out: Queue<Request>) -> Box<[SymbolAsk]> {
    let mut asks = List::with_capacity(u32::try_from(CONNECTORS).unwrap());
    while let Some(next) = out.pop() {
        if let Request::Ask { connector, ask: Ask::TaskHoldings { symbols, tracked, number, .. } } = next {
            asks.push((connector, symbols, tracked, number)).unwrap();
        }
    }
    asks.into_boxed()
}

#[test]
fn a_proposals_members_keep_their_symbol_order_and_tracking_separately() {
    let (mut core, env) = party_core();
    let people::Ask::Create { project, mut batch } = symbolic_goal(FIRST_CONNECTOR) else { panic!("goal") };
    let tracked = batch[0].clone();
    let mut untracked = fixtures::member(b"untracked", 100);
    untracked.symbols = Box::new([
        people::Symbol { connector: SYMBOL_CONNECTOR, symbol: SECOND_RESOURCE },
        people::Symbol { connector: SYMBOL_CONNECTOR, symbol: OWN_RESOURCE },
    ]);
    batch = Box::new([tracked, untracked]);
    let mut out = ask(
        &mut core,
        &env,
        people::Ask::Propose { project, action: people::Action::Batch(batch), reason: Box::new([]) },
    );
    let mut request = None;
    let mut first = Queue::uncapped();
    while let Some(next) = out.pop() {
        if let Request::Ask { ask: Ask::TaskHoldings { request: token, .. }, .. } = &next {
            request = Some(*token);
        }
        first.push(next);
    }
    assert_eq!(
        symbol_asks(first).as_ref(),
        &[(FIRST_CONNECTOR, Box::from([OWN_RESOURCE]), true, OWNER), (SYMBOL_CONNECTOR, Box::from([]), true, OWNER),]
    );
    let request = request.expect("the first member asks for holdings");
    drop(step(
        &mut core,
        &env,
        Event::Holdings { request, connector: FIRST_CONNECTOR, holdings: Some(Box::new([])), refused: None },
    ));
    let out = step(
        &mut core,
        &env,
        Event::Holdings { request, connector: SYMBOL_CONNECTOR, holdings: Some(Box::new([])), refused: None },
    );
    assert_eq!(
        symbol_asks(out).as_ref(),
        &[
            (FIRST_CONNECTOR, Box::from([]), false, CHILD),
            (SYMBOL_CONNECTOR, Box::from([SECOND_RESOURCE, OWN_RESOURCE]), false, CHILD),
        ]
    );
}
