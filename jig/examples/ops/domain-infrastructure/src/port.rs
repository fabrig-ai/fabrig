//! Infrastructure's engine port (jig/docs/design/domain/connectors.md, 2).
use crate::translate::{
    admission, holdings, infrastructure_added, infrastructure_description, infrastructure_executor, infrastructure_key,
    infrastructure_outcome, infrastructure_spec, resource_role,
};
use crate::{
    ClientDocument, ClientScope, CloseOutcome, DocumentMissing, Domain, Effect, Event, Hold, Limits, Out,
    ProcedureSignal, Request, Resource, StepDecision, fire_system, step_system,
};
use alloc::boxed::Box;
use jig_domain_contract::{self as jig, effects, lifecycle, ops, parts, procedures, requests, resources};
use ops::party::{ActStatus, Again};
use requests::ApplicationScope;
use skein_lib::{Duration, Env, List, Map, Queue, Token};

#[derive(Debug)]
pub(crate) struct State {
    number: u16,
    deployment: [u8; 16],
    reason: Box<[u8]>,
    grace: Duration,
    typed: Map<Token, Effect>,
    procedure_effects: Map<u64, (u64, Effect)>,
    effect_tasks: Map<Token, u64>,
    offers: Map<u64, (u64, u32)>,
    documents: Map<Token, ClientDocument>,
}

impl State {
    pub(crate) fn new() -> Self {
        Self {
            number: 0,
            deployment: [0; 16],
            reason: Box::new([]),
            grace: Duration::from_secs(0),
            typed: Map::uncapped(),
            procedure_effects: Map::uncapped(),
            effect_tasks: Map::uncapped(),
            offers: Map::uncapped(),
            documents: Map::uncapped(),
        }
    }
}

impl Domain {
    /// Give the connector its number and the deployment's close policy.
    #[must_use]
    pub fn with_contract(
        mut self,
        number: jig::Connector,
        deployment: [u8; 16],
        reason: Box<[u8]>,
        grace: Duration,
    ) -> Self {
        self.port.number = number.0;
        self.port.deployment = deployment;
        self.port.reason = reason;
        self.port.grace = grace;
        self
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
        Event::Ask(ask) => ask_engine(domain, env, ask, &mut system, out),
        Event::Released(released) => match released {
            jig::Released::Make { entry } => step_system(domain, env, Event::MakeEntry { entry }, &mut system),
            jig::Released::Answered { key, outcome } => {
                let status = match outcome.act_status() {
                    ActStatus::Closed => CloseOutcome::Closed,
                    ActStatus::Again => CloseOutcome::Again,
                    ActStatus::NotOffered => CloseOutcome::NotOffered,
                    ActStatus::NoOwner => CloseOutcome::NoOwner,
                    ActStatus::Ended => CloseOutcome::Ended,
                    ActStatus::Refused => CloseOutcome::Refused,
                };
                step_system(domain, env, Event::Closed { key, outcome: status }, &mut system);
            }
        },
        Event::StageEffect { owner, effect } => {
            domain.port.typed.insert(owner, effect).expect("one staged connector input per owner");
        }
        Event::EffectOwner { task, owner } => {
            let (step, effect) = domain.port.procedure_effects.remove(&task).expect("pending procedure effect");
            domain.port.effect_tasks.insert(owner, task).expect("one task per procedure effect owner");
            domain.port.typed.insert(owner, effect).expect("one payload per owner");
            say_effect(
                out,
                effects::Up::Start { owner, origin: effects::EffectOrigin::Procedure { task, step, entry: None } },
            );
        }
        system_event @ (Event::ProcedureAdmission { .. }
        | Event::ResourceChanged { .. }
        | Event::ObservedServiceChanged { .. }
        | Event::Closed { .. }
        | Event::SendCloses
        | Event::ProcedureDelegateOutcome { .. }
        | Event::DocumentPrepare { .. }
        | Event::DocumentForget { .. }
        | Event::Document { .. }
        | Event::ReadDocument { .. }
        | Event::ApplicationPrepare { .. }
        | Event::ApplicationDrop { .. }
        | Event::Application { .. }
        | Event::Close { .. }
        | Event::Release { .. }
        | Event::Names { .. }
        | Event::Unnamed { .. }
        | Event::StartProcedure { .. }
        | Event::Procedure { .. }
        | Event::Describe { .. }
        | Event::DescribeProposal { .. }
        | Event::KeepProposal { .. }
        | Event::DropProposal { .. }
        | Event::Keep { .. }
        | Event::Drop { .. }
        | Event::Make { .. }
        | Event::MakeEntry { .. }
        | Event::ResolveEntry { .. }
        | Event::Restore { .. }
        | Event::ReadAfresh
        | Event::Restart
        | Event::System(_)) => step_system(domain, env, system_event, &mut system),
    }
    say_system(domain, system, out);
}

/// Fire the connector's due timer at its port.
pub fn fire(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Out>) {
    let mut system = Queue::uncapped();
    fire_system(domain, env, &mut system);
    say_system(domain, system, out);
}

fn ask_engine(
    domain: &mut Domain,
    env: &Env<Limits>,
    ask: jig::Ask,
    system: &mut Queue<Request>,
    out: &mut Queue<Out>,
) {
    match ask {
        jig::Ask::Lifecycle(ask) => match ask {
            lifecycle::Ask::CatchUp => step_system(domain, env, Event::ReadAfresh, system),
            lifecycle::Ask::ReleaseProject { project } => {
                out.push(Out::Up(jig::Up::Lifecycle(lifecycle::Up::Released { project })));
            }
            // Task reliance is released with the task; there are no separate adoptions or project settings.
            lifecycle::Ask::Adopt { .. } => {}
        },
        jig::Ask::Resources(ask) => resources(domain, env, ask, system, out),
        jig::Ask::Effects(ask) => effects(domain, env, ask, system, out),
        jig::Ask::Procedures(ask) => procedure(domain, env, ask, system, out),
        jig::Ask::Parts(ask) => documents(domain, env, ask, out),
        jig::Ask::Requests(ask) => match ask {
            requests::Ask::Decide { request, token, owner, scope, role, retention, .. } => {
                if let (ApplicationScope::Project(project), Some(role)) = (scope, role) {
                    step_system(
                        domain,
                        env,
                        Event::Application {
                            request,
                            token,
                            owner,
                            person: 0,
                            project,
                            role: role.number(),
                            retention,
                        },
                        system,
                    );
                } else {
                    step_system(domain, env, Event::ApplicationDrop { owner }, system);
                    out.push(Out::Up(jig::Up::Requests(requests::Up::Decided { request, outcome: Err(Again::Busy) })));
                }
            }
            requests::Ask::Drop { owner } => step_system(domain, env, Event::ApplicationDrop { owner }, system),
            requests::Ask::PartAnswered { .. } => {}
        },
        jig::Ask::Topics(_) | jig::Ask::Projections(_) | jig::Ask::Settings(_) => {}
    }
}

fn effects(
    domain: &mut Domain,
    env: &Env<Limits>,
    ask: effects::Ask,
    system: &mut Queue<Request>,
    out: &mut Queue<Out>,
) {
    let event = match ask {
        effects::Ask::Describe { owner } => {
            let Some(effect) = domain.port.typed.remove(&owner) else {
                say_effect(out, effects::Up::Refused { owner });
                return;
            };
            Event::Describe { token: owner, effect }
        }
        effects::Ask::DescribeProposal { owner, proposal } => {
            Event::DescribeProposal { token: owner, number: proposal }
        }
        effects::Ask::KeepProposal { owner, proposal, task, .. } => {
            domain.port.effect_tasks.remove(&owner);
            Event::KeepProposal { token: owner, number: proposal, task }
        }
        effects::Ask::DropProposal { proposal } => Event::DropProposal { number: proposal },
        effects::Ask::Keep { owner, entry, key, project, .. } => {
            domain.port.effect_tasks.remove(&owner);
            Event::Keep { token: owner, entry, key: infrastructure_key(key), project }
        }
        effects::Ask::Drop { owner, answer, .. } => {
            domain.port.typed.remove(&owner);
            if let Some(task) = domain.port.effect_tasks.remove(&owner) {
                step_system(domain, env, Event::ProcedureAdmission { task, answer: admission(answer) }, system);
            }
            Event::Drop { token: owner }
        }
        effects::Ask::Judge { owner, judge, state, .. } => {
            say_effect(
                out,
                effects::Up::Verdict {
                    owner,
                    judge,
                    state,
                    at: env.wall,
                    guarded: false,
                    verdict: effects::Verdict::Refuse,
                },
            );
            return;
        }
        effects::Ask::Resolve { entry, again } => Event::ResolveEntry { entry, repeat: again },
        effects::Ask::Close { task, .. } => Event::Close { task },
        effects::Ask::Release { task, .. } => Event::Release { task },
    };
    step_system(domain, env, event, system);
}

fn resources(
    domain: &mut Domain,
    env: &Env<Limits>,
    ask: resources::Ask,
    system: &mut Queue<Request>,
    out: &mut Queue<Out>,
) {
    let number = domain.port.number;
    let up = match ask {
        resources::Ask::Holdings { request, executor, spec, .. } => {
            resources::Up::Holdings { request, holdings: task_holdings(env, number, executor, &spec), refused: None }
        }
        resources::Ask::DelegateHoldings { request, members, .. } => {
            resources::Up::DelegateHoldings { request, holdings: batch_holdings(env, number, members), refused: None }
        }
        resources::Ask::ProcedureHoldings { task, step, members } => resources::Up::ProcedureHoldings {
            task,
            step,
            holdings: batch_holdings(env, number, members),
            refused: None,
        },
        resources::Ask::Unname { task } => {
            step_system(domain, env, Event::Unnamed { task }, system);
            return;
        }
        resources::Ask::Hold { .. }
        | resources::Ask::Lost { .. }
        | resources::Ask::RepairRefused { .. }
        | resources::Ask::DelegateRefused { .. } => return,
    };
    out.push(Out::Up(jig::Up::Resources(up)));
}

fn task_holdings(
    env: &Env<Limits>,
    number: u16,
    executor: ops::Executor,
    spec: &ops::Spec,
) -> Option<Box<[resources::Holding]>> {
    holdings(
        number,
        crate::holdings(&env.limits.vocabulary, number, infrastructure_executor(executor), &infrastructure_spec(spec)),
    )
}

fn batch_holdings(
    env: &Env<Limits>,
    number: u16,
    members: Box<[resources::HoldingsNeed]>,
) -> Option<Box<[Box<[resources::Holding]>]>> {
    let mut rows = List::with_capacity(u32::try_from(members.len()).expect("bounded batch"));
    for member in members {
        rows.push(task_holdings(env, number, member.executor, &member.spec)?).expect("one holdings row");
    }
    Some(rows.into_boxed())
}

fn procedure(
    domain: &mut Domain,
    env: &Env<Limits>,
    ask: procedures::Ask,
    system: &mut Queue<Request>,
    out: &mut Queue<Out>,
) {
    match ask {
        procedures::Ask::Start { context, step, chat } => {
            let task = context.task;
            let code = match context.executor {
                ops::Executor::Procedure { code, .. } => code,
                ops::Executor::Job { .. } => return,
                ops::Executor::Agent { .. } | ops::Executor::Person(_) => unreachable!("core-selected procedure"),
            };
            let Some(procedure) = crate::procedure(
                &env.limits.vocabulary,
                code,
                &infrastructure_spec(&context.spec),
                domain.port.deployment,
            ) else {
                say_step(out, task, step, code, procedures::StepEnd::Hold(ops::Hold::Procedure));
                return;
            };
            domain.port.offers.insert(task, (step, code)).expect("one offered procedure step per task");
            step_system(
                domain,
                env,
                Event::StartProcedure {
                    task,
                    project: context.project,
                    chat,
                    procedure,
                    pending: context.pending,
                    added: infrastructure_added(context.added),
                },
                system,
            );
            say_system(domain, core::mem::replace(system, Queue::uncapped()), out);
            out.push(Out::AfterCommit(Box::new(Event::Procedure { task, signal: ProcedureSignal::Step })));
        }
        procedures::Ask::StepMade { .. }
        | procedures::Ask::StepRefused { .. }
        | procedures::Ask::WaitsRefused { .. } => {}
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

fn say_step(out: &mut Queue<Out>, task: u64, step: u64, code: u32, end: procedures::StepEnd) {
    out.push(Out::Up(jig::Up::Procedures(procedures::Up::Step {
        task,
        step,
        code,
        decision: procedures::ProcedureStep::new(end),
    })));
}

fn name(number: u16, resource: &Resource) -> resources::Name {
    resources::Name { connector: number, path: resource.segments() }
}

#[expect(clippy::too_many_lines, reason = "one exhaustive system-to-port dispatcher")]
fn say_system(domain: &mut Domain, mut system: Queue<Request>, out: &mut Queue<Out>) {
    let number = domain.port.number;
    for _ in 0..system.len() {
        match system.pop().expect("connector output count") {
            Request::Described { token, description } => say_effect(
                out,
                effects::Up::Described {
                    owner: token,
                    description: Box::new(infrastructure_description(number, description)),
                },
            ),
            Request::Refused { token } => say_effect(out, effects::Up::Refused { owner: token }),
            Request::Outcome { entry, key, outcome } => {
                say_effect(
                    out,
                    effects::Up::Outbox { entry, task: key.task, outcome: infrastructure_outcome(outcome) },
                );
                if let Some(event) = crate::procedure_outcome(key, outcome) {
                    out.push(Out::Continue(Box::new(event)));
                }
            }
            Request::Closed { task } => say_effect(out, effects::Up::Closed { task }),
            Request::Released { task } => say_effect(out, effects::Up::Released { task }),
            Request::Close { key, project, chat } => out.push(Out::Up(jig::Up::Act {
                key,
                act: Box::new(ops::Act::Close {
                    project,
                    task: chat,
                    grace: Some(domain.port.grace),
                    reason: domain.port.reason.clone(),
                }),
                carried: None,
            })),
            Request::Make { key } => out.push(Out::AfterCommit(Box::new(Event::Make { key }))),
            Request::Step { task, decision } => {
                let Some((step, code)) = domain.port.offers.remove(&task) else { continue };
                let end = match decision {
                    StepDecision::Effect(effect) => {
                        domain
                            .port
                            .procedure_effects
                            .insert(task, (step, effect))
                            .expect("one pending effect per procedure");
                        out.push(Out::EffectOwner { task });
                        continue;
                    }
                    StepDecision::Wait { .. } => procedures::StepEnd::Wait,
                    StepDecision::Finish => {
                        procedures::StepEnd::Result(ops::TaskResult::Report { words: Box::new([]) })
                    }
                    StepDecision::FinishReport { words } => {
                        procedures::StepEnd::Result(ops::TaskResult::Report { words })
                    }
                    StepDecision::Hold { .. } => procedures::StepEnd::Hold(ops::Hold::Effects),
                };
                say_step(out, task, step, code, end);
            }
            Request::Named { project, resources, .. } => {
                for named in resources {
                    let hold = match named.hold {
                        Hold::None => resources::HoldKind::Shared,
                        Hold::ExclusiveWait => resources::HoldKind::Exclusive { wait: true },
                        Hold::PooledWait => resources::HoldKind::Pooled { slots: 0, wait: true },
                    };
                    out.push(Out::Up(jig::Up::Resources(resources::Up::Resource {
                        project,
                        name: name(number, &named.resource),
                        hold,
                        role: resource_role(named.role),
                    })));
                }
            }
            Request::Drift { task, resource } => {
                out.push(Out::Up(jig::Up::Resources(resources::Up::Drift { task, resource: name(number, &resource) })));
            }
            Request::Slots { pool, quota, .. } => out.push(Out::Up(jig::Up::Resources(resources::Up::PoolSlots {
                name: name(number, &Resource::Pool(pool)),
                slots: quota,
            }))),
            Request::Wake { task } => out.push(Out::Up(jig::Up::Procedures(procedures::Up::Wake { task }))),
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
            Request::ApplicationDone { request, token } => {
                out.push(Out::Up(jig::Up::Requests(requests::Up::Decided { request, outcome: Ok(token) })));
            }
            Request::ApplicationBusy { request } => {
                out.push(Out::Up(jig::Up::Requests(requests::Up::Decided { request, outcome: Err(Again::Busy) })));
            }
            Request::Marked { subjects } => out.push(Out::Up(jig::Up::Requests(requests::Up::Marked { subjects }))),
            Request::Save { record } => out.push(Out::Save { record }),
            Request::Erase { key } => out.push(Out::Erase { key }),
            Request::System(request) => out.push(Out::System(request)),
            Request::Changed { resource } => out.push(Out::Changed { resource }),
            Request::Document { .. } | Request::ReadAnswered { .. } => unreachable!("documents cross the parts port"),
        }
    }
}
