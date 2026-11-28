//! Observability's engine port (jig/docs/design/domain/connectors.md, 2).
use crate::translate::{
    news_class, observability_description, observability_key, observability_outcome, observability_spec, requirement,
    service, verdict, watch_inputs,
};
use crate::{
    Alert, ClientDocument, ClientScope, DocumentMissing, Domain, Effect, Event, Limits, Out, Request, WatchDecision,
    fire_system, step_system,
};
use alloc::boxed::Box;
use jig_domain_contract::{self as jig, effects, lifecycle, ops, parts, procedures, requests, topics};
use requests::ApplicationScope;
use skein_lib::{Env, List, Map, Queue, Token};

/// Deployment policy for the two requirements this connector judges.
#[derive(Clone, Copy, Debug)]
pub struct Requirements {
    pub healthy_replica_requirement: u16,
    pub low_load_requirement: u16,
    pub scale_down_load_percent: u8,
    pub scale_down_load_seconds: u64,
    pub freshness_seconds: u64,
}

#[derive(Debug)]
pub(crate) struct State {
    number: u16,
    requirements: Requirements,
    triage: Map<u16, Box<[procedures::Delegate]>>,
    lead: Map<u16, Box<[procedures::Delegate]>>,
    offers: Map<u64, (u64, u32)>,
    last_question: u64,
    staged_effects: Map<Token, Effect>,
    documents: Map<Token, ClientDocument>,
}

impl State {
    pub(crate) fn new() -> Self {
        Self {
            number: 0,
            requirements: Requirements {
                healthy_replica_requirement: 1,
                low_load_requirement: 2,
                scale_down_load_percent: 20,
                scale_down_load_seconds: 1800,
                freshness_seconds: 60,
            },
            triage: Map::uncapped(),
            lead: Map::uncapped(),
            offers: Map::uncapped(),
            last_question: 0,
            staged_effects: Map::uncapped(),
            documents: Map::uncapped(),
        }
    }
}

impl Domain {
    /// Give the connector its assigned number and requirement policy.
    #[must_use]
    pub fn with_contract(mut self, number: jig::Connector, requirements: Requirements) -> Self {
        self.port.number = number.0;
        self.port.requirements = requirements;
        self
    }

    /// Keep a decoded template with the owner of the procedure that uses it.
    pub fn prepare_triage(&mut self, number: u16, delegate: Box<[procedures::Delegate]>) {
        self.port.triage.insert(number, delegate).expect("configured triage templates fit");
    }

    /// Keep a decoded lead template with its procedure owner.
    pub fn prepare_lead(&mut self, number: u16, delegate: Box<[procedures::Delegate]>) {
        self.port.lead.insert(number, delegate).expect("configured lead templates fit");
    }

    /// Move the prepared document into the engine-paced delivery.
    pub fn take_document(&mut self, subject: Token) -> Option<ClientDocument> {
        self.port.documents.remove(&subject)
    }
}

/// Step the connector at its engine and system boundary.
pub fn step(domain: &mut Domain, env: &Env<Limits>, event: Event, out: &mut Queue<Out>) {
    let mut system = Queue::uncapped();
    match event {
        Event::Ask(ask) => match ask {
            jig::Ask::Lifecycle(ask) => match ask {
                lifecycle::Ask::Restore => out.push(Out::LoadRestore),
                lifecycle::Ask::CatchUp => step_system(domain, env, Event::ReadAfresh, &mut system),
                lifecycle::Ask::ReleaseProject { project } => {
                    out.push(Out::Up(jig::Up::Lifecycle(lifecycle::Up::Released { project })));
                }
                // Watch state belongs to its task. No separate adoptions or project settings are kept.
                lifecycle::Ask::Adopt { .. } => {}
            },
            jig::Ask::Effects(ask) => effect(domain, env, ask, &mut system, out),
            jig::Ask::Topics(ask) => match ask {
                topics::Ask::Subscribed { key, subscription, .. } => {
                    step_system(domain, env, Event::LinkWatch { task: key.task, subscription }, &mut system);
                }
                topics::Ask::Ended { task, subscription } => {
                    crate::domain::end_topic(domain, task, subscription, &mut system);
                }
                topics::Ask::Unsubscribed { .. } | topics::Ask::Refused { .. } => {}
            },
            jig::Ask::Parts(ask) => documents(domain, env, ask, out),
            jig::Ask::Procedures(ask) => procedure(domain, env, ask, &mut system, out),
            jig::Ask::Resources(_) | jig::Ask::Projections(_) | jig::Ask::Settings(_) | jig::Ask::Requests(_) => {}
        },
        Event::RestoreDone => {
            out.push(Out::Up(jig::Up::Lifecycle(lifecycle::Up::Done { stage: lifecycle::RestartStage::Restored })));
        }
        Event::Released(released) => match released {
            jig::Released::Make { entry } => step_system(domain, env, Event::MakeEntry { entry }, &mut system),
            jig::Released::Answered { .. } => {}
        },
        Event::StageEffect { owner, effect } => {
            domain.port.staged_effects.insert(owner, effect).expect("one staged effect per owner");
        }
        Event::WakeReleased { task } => out.push(Out::Up(jig::Up::Procedures(procedures::Up::Wake { task }))),
        system_event @ (Event::DocumentPrepare { .. }
        | Event::DocumentForget { .. }
        | Event::Document { .. }
        | Event::ReadDocument { .. }
        | Event::Close { .. }
        | Event::Release { .. }
        | Event::Judge { .. }
        | Event::Read { .. }
        | Event::Subscribe { .. }
        | Event::Unsubscribe { .. }
        | Event::LinkWatch { .. }
        | Event::StartWatch { .. }
        | Event::StopWatch { .. }
        | Event::WakeWatch { .. }
        | Event::WatchStep { .. }
        | Event::ProcedureDelegateOutcome { .. }
        | Event::Describe { .. }
        | Event::Keep { .. }
        | Event::Drop { .. }
        | Event::Make { .. }
        | Event::MakeEntry { .. }
        | Event::Restore { .. }
        | Event::ReadAfresh
        | Event::Restart
        | Event::System(_)) => step_system(domain, env, system_event, &mut system),
    }
    say_system(domain, env, system, out);
}

/// Fire the connector's due timer at its port.
pub fn fire(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Out>) {
    let mut system = Queue::uncapped();
    fire_system(domain, env, &mut system);
    say_system(domain, env, system, out);
}

fn effect(
    domain: &mut Domain,
    env: &Env<Limits>,
    ask: effects::Ask,
    system: &mut Queue<Request>,
    out: &mut Queue<Out>,
) {
    let event = match ask {
        effects::Ask::Describe { owner } => {
            let Some(effect) = domain.port.staged_effects.remove(&owner) else {
                say_effect(out, effects::Up::Refused { owner });
                return;
            };
            let mut description = Queue::uncapped();
            step_system(domain, env, Event::Describe { token: owner, effect }, &mut description);
            if let Some(Request::Refused { .. }) = description.iter().next() {
                say_effect(out, effects::Up::Refused { owner });
            } else {
                say_system(domain, env, description, out);
            }
            return;
        }
        effects::Ask::DescribeProposal { owner, .. } => {
            say_effect(out, effects::Up::Refused { owner });
            return;
        }
        effects::Ask::Judge { owner, judge, state, resources } => {
            // The judgement is synchronous. Its original correlation never leaves the connector.
            let (verdict, at) = if let (Some(service), Some(requirement)) =
                (service(&resources), requirement(judge.requirement, &domain.port.requirements))
            {
                domain.port.last_question = domain.port.last_question.checked_add(1).expect("question number fits");
                let token = Token::new(domain.port.last_question);
                let mut judged = Queue::uncapped();
                step_system(
                    domain,
                    env,
                    Event::Judge { token, requirement, service, freshness: domain.port.requirements.freshness_seconds },
                    &mut judged,
                );
                let Some(Request::Verdict { token: answered, verdict: result }) = judged.pop() else {
                    unreachable!("the system answers a question before asking for fresh facts");
                };
                assert_eq!(answered, token, "the answer names this question");
                for _ in 0..judged.len() {
                    system.push(judged.pop().expect("remaining outputs"));
                }
                verdict(result, env.wall)
            } else {
                (effects::Verdict::Refuse, env.wall)
            };
            say_effect(out, effects::Up::Verdict { owner, judge, state, verdict, at, guarded: false });
            return;
        }
        effects::Ask::Keep { owner, entry, key, project, .. } => {
            Event::Keep { token: owner, entry, key: observability_key(key), project }
        }
        effects::Ask::Drop { owner, .. } => {
            domain.port.staged_effects.remove(&owner);
            Event::Drop { token: owner }
        }
        effects::Ask::Close { task, .. } => Event::Close { task },
        effects::Ask::Release { task, .. } => Event::Release { task },
        effects::Ask::Resolve { entry, again } => {
            if let Some(key) = crate::domain::resolve(domain, env, entry, again, system) {
                say_system(domain, env, core::mem::replace(system, Queue::uncapped()), out);
                say_effect(
                    out,
                    effects::Up::Outbox { entry, task: key.task, outcome: effects::OutboxOutcome::NotMade },
                );
            }
            return;
        }
        effects::Ask::KeepProposal { .. } | effects::Ask::DropProposal { .. } => return,
    };
    step_system(domain, env, event, system);
}

fn procedure(
    domain: &mut Domain,
    env: &Env<Limits>,
    ask: procedures::Ask,
    system: &mut Queue<Request>,
    out: &mut Queue<Out>,
) {
    match ask {
        procedures::Ask::Start { context, step, .. } => {
            let task = context.task;
            let code = match context.executor {
                ops::Executor::Procedure { code, .. } => code,
                ops::Executor::Job { .. } => return,
                ops::Executor::Agent { .. } | ops::Executor::Person(_) => unreachable!("core-selected procedure"),
            };
            let Some(spec) = crate::watch_spec(&observability_spec(&context.spec), &env.limits.vocabulary) else {
                say_step(
                    out,
                    task,
                    step,
                    code,
                    procedures::ProcedureStep::new(procedures::StepEnd::Hold(ops::Hold::Procedure)),
                );
                return;
            };
            domain.port.offers.insert(task, (step, code)).expect("one offered procedure step per task");
            step_system(
                domain,
                env,
                Event::StartWatch {
                    task,
                    project: context.project,
                    services: Box::new([spec.service]),
                    template: spec.template,
                },
                system,
            );
            let messages = crate::watch_messages(watch_inputs(context.inbox), &env.limits);
            step_system(domain, env, Event::WatchStep { task, step, batch: context.last_message, messages }, system);
        }
        procedures::Ask::StepMade { task, step, first: Some(child), .. } => {
            step_system(domain, env, Event::ProcedureDelegateOutcome { task, step, child: Some(child) }, system);
        }
        procedures::Ask::StepRefused { task, step, .. } => {
            step_system(domain, env, Event::ProcedureDelegateOutcome { task, step, child: None }, system);
        }
        procedures::Ask::StepMade { first: None, .. } | procedures::Ask::WaitsRefused { .. } => {}
    }
}

fn document_scope(scope: ApplicationScope) -> Option<ClientScope> {
    match scope {
        ApplicationScope::Project(project) => Some(ClientScope::Project(project)),
        ApplicationScope::Deployment => Some(ClientScope::Deployment),
        ApplicationScope::Own => None,
    }
}

fn documents(domain: &mut Domain, env: &Env<Limits>, ask: parts::Ask, out: &mut Queue<Out>) {
    let up = match ask {
        parts::Ask::Document { watcher, scope, subject, party, .. } => {
            let document = match document_scope(scope) {
                Some(scope) => crate::client_documents::make(domain, &env.limits, scope, subject, party),
                None => Err(DocumentMissing::Unknown),
            };
            let outcome = match document {
                Ok(value) => {
                    assert!(
                        domain.port.documents.insert(subject, value).expect("due document has room").is_none(),
                        "the prior document was taken"
                    );
                    parts::Made::Ready
                }
                Err(DocumentMissing::Unknown) => parts::Made::Gone { why: parts::End::Gone },
                Err(DocumentMissing::NotAllowed) => parts::Made::Gone { why: parts::End::NotAllowed },
            };
            parts::Up::Made { watcher, outcome }
        }
        parts::Ask::Plan(_)
        | parts::Ask::Abandon { .. }
        | parts::Ask::Gather { .. }
        | parts::Ask::Cut { .. }
        | parts::Ask::Drop { .. } => return,
        parts::Ask::DropDocument { subject } => {
            domain.client_subjects.remove(&subject);
            domain.port.documents.remove(&subject);
            return;
        }
        parts::Ask::Read { reply, scope, read, party, .. } => {
            let answer = match document_scope(scope) {
                Some(scope) => crate::client_documents::make(domain, &env.limits, scope, read, party),
                None => Err(DocumentMissing::Unknown),
            };
            domain.client_subjects.remove(&read);
            out.push(Out::ClientRead { reply, answer });
            return;
        }
        parts::Ask::Lines(ask) => {
            let mut parts = List::uncapped();
            for _row in &ask.rows {
                for _kind in &ask.kinds {
                    parts.push(parts::MadePart::Failed(parts::PartFailure::Unreadable)).expect("parts fit the ask");
                }
            }
            parts::Up::Lines { token: ask.token, parts }
        }
        parts::Ask::Public { reply, .. } => parts::Up::Public { reply, answer: None },
    };
    out.push(Out::Up(jig::Up::Parts(up)));
}

fn say_effect(out: &mut Queue<Out>, up: effects::Up) {
    out.push(Out::Up(jig::Up::Effects(up)));
}

fn say_step(out: &mut Queue<Out>, task: u64, step: u64, code: u32, decision: procedures::ProcedureStep) {
    out.push(Out::Up(jig::Up::Procedures(procedures::Up::Step { task, step, code, decision })));
}

fn watch_step(domain: &mut Domain, out: &mut Queue<Out>, task: u64, offered: Option<u64>, chosen: WatchDecision) {
    let Some((step, code)) = domain.port.offers.remove(&task) else { return };
    if let Some(offered) = offered {
        assert_eq!(offered, step, "the owner answers its offered step");
    }
    let Some(watch) = domain.watches.get(&task) else { return };
    let decision = match chosen {
        WatchDecision::Triage { alerts, .. } => {
            delegation(triage_batch(domain.port.triage.get(&watch.template).cloned(), &alerts))
        }
        WatchDecision::Lead { .. } => delegation(domain.port.lead.get(&watch.template).cloned()),
        WatchDecision::Wait => procedures::ProcedureStep::new(procedures::StepEnd::Wait),
        WatchDecision::Hold => procedures::ProcedureStep::new(procedures::StepEnd::Hold(ops::Hold::Procedure)),
    };
    say_step(out, task, step, code, decision);
}

fn delegation(batch: Option<Box<[procedures::Delegate]>>) -> procedures::ProcedureStep {
    match batch {
        Some(batch) => procedures::ProcedureStep { batch, ..procedures::ProcedureStep::new(procedures::StepEnd::Wait) },
        None => procedures::ProcedureStep::new(procedures::StepEnd::Hold(ops::Hold::Procedure)),
    }
}

/// Add the alert IDs to the already-decoded triage delegate.
#[must_use]
pub fn triage_batch(batch: Option<Box<[procedures::Delegate]>>, alerts: &[u64]) -> Option<Box<[procedures::Delegate]>> {
    let mut batch = batch?;
    if batch.len() != 1 {
        return None;
    }
    for member in &mut batch {
        let mut room = member.spec.words.len().checked_add(b"\nAlerts:".len())?;
        for alert in alerts {
            room = room.checked_add(1)?.checked_add(skein_lib::Decimal::of(*alert).as_bytes().len())?;
        }
        let mut words = skein_lib::Writer::new(room);
        words.put(&member.spec.words).ok()?;
        words.put(b"\nAlerts:").ok()?;
        for alert in alerts {
            words.put(b" ").ok()?;
            words.put(skein_lib::Decimal::of(*alert).as_bytes()).ok()?;
        }
        member.spec.words = words.finish();
    }
    Some(batch)
}

fn news(out: &mut Queue<Out>, subscribers: Box<[crate::Classed]>, words: &[u8]) {
    for subscriber in subscribers {
        out.push(Out::Up(jig::Up::Topics(topics::Up::News {
            task: subscriber.task,
            subscription: subscriber.subscription,
            topic: subscriber.task,
            class: news_class(subscriber.class),
            words: Box::from(words),
        })));
    }
}

fn say_system(domain: &mut Domain, env: &Env<Limits>, mut system: Queue<Request>, out: &mut Queue<Out>) {
    for _ in 0..system.len() {
        match system.pop().expect("connector output count") {
            Request::Described { token, effect } => say_effect(
                out,
                effects::Up::Described {
                    owner: token,
                    description: Box::new(observability_description(
                        domain.port.number,
                        effect,
                        &env.limits.vocabulary,
                    )),
                },
            ),
            Request::Closed { task } => say_effect(out, effects::Up::Closed { task }),
            Request::Released { task } => say_effect(out, effects::Up::Released { task }),
            Request::Outcome { entry, key, outcome } => {
                say_effect(out, effects::Up::Outbox { entry, task: key.task, outcome: observability_outcome(outcome) });
            }
            Request::Make { key } => out.push(Out::AfterCommit(Box::new(Event::Make { key }))),
            Request::WakeProcedure { task } => out.push(Out::AfterCommit(Box::new(Event::WakeReleased { task }))),
            Request::WatchDecision { task, step, decision } => watch_step(domain, out, task, Some(step), decision),
            Request::Triage { watch, alerts, delegate, .. } => {
                watch_step(domain, out, watch, None, WatchDecision::Triage { alerts, delegate });
            }
            Request::News { alert, subscribers } => news(out, subscribers, &alert_words(&alert)),
            Request::Health { healthy, subscribers, .. } => news(out, subscribers, &health_words(healthy)),
            Request::Marked { subjects } => out.push(Out::Up(jig::Up::Requests(requests::Up::Marked { subjects }))),
            Request::Restarted { stage } => {
                let stage = match stage {
                    crate::RestartStage::ReadAfresh => lifecycle::RestartStage::ReadAfresh,
                    crate::RestartStage::Outbox => lifecycle::RestartStage::Settled,
                };
                out.push(Out::Up(jig::Up::Lifecycle(lifecycle::Up::Done { stage })));
                if stage == lifecycle::RestartStage::ReadAfresh {
                    out.push(Out::Continue(Box::new(Event::Restart)));
                }
            }
            Request::Save { record } => out.push(Out::Save { record }),
            Request::Erase { key } => out.push(Out::Erase { key }),
            Request::System(request) => out.push(Out::System(request)),
            Request::Changed { service } => out.push(Out::Changed { service }),
            Request::WatchInterest { task, subscription, topic } => {
                out.push(Out::WatchInterest { task, subscription, topic });
            }
            Request::AssociationRefused { task, step, reason } => {
                out.push(Out::AssociationRefused { task, step, reason });
            }
            Request::Answer { token, bytes } => out.push(Out::Answer { token, bytes }),
            Request::Refused { token } => out.push(Out::ReadRefused { token }),
            Request::Verdict { .. } => {}
            Request::Document { .. } | Request::ReadAnswered { .. } => unreachable!("documents cross the parts port"),
        }
    }
}

/// Render the alert identity and severity as subscriber words.
#[must_use]
pub fn alert_words(alert: &Alert) -> Box<[u8]> {
    let bytes = b"alert "
        .len()
        .checked_add(skein_lib::Decimal::of(alert.number).as_bytes().len())
        .expect("alert number")
        .checked_add(b" severity ".len())
        .expect("severity heading")
        .checked_add(skein_lib::Decimal::of(u64::from(alert.severity)).as_bytes().len())
        .expect("severity");
    let mut writer = skein_lib::Writer::new(bytes);
    writer.put(b"alert ").expect("alert heading");
    writer.put(skein_lib::Decimal::of(alert.number).as_bytes()).expect("alert identity");
    writer.put(b" severity ").expect("severity heading");
    writer.put(skein_lib::Decimal::of(u64::from(alert.severity)).as_bytes()).expect("severity");
    writer.finish()
}
/// Render a service's health as subscriber words.
#[must_use]
pub fn health_words(healthy: bool) -> Box<[u8]> {
    if healthy { Box::from(&b"healthy"[..]) } else { Box::from(&b"unhealthy"[..]) }
}
