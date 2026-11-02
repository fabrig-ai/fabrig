//! Each variant fixes its destination. Replace connector arms when copying;
//! retain the journal marks and engine's permanent host link (domain/root.md, 5).
use crate::domain::{Work, child, held, now, write};
use crate::translate;
use crate::{Domain, Event, Key, Limits, Output, Record, Released, Write};
use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_core as core;
use jig_domain_core::effects::connector;
use jig_domain_fleet as fleet;
use jig_domain_host as host;
use jig_domain_hub_link::{Event as LinkEvent, Procedure};
use jig_domain_tasks as tasks;
use jig_ops_domain_connectors::Connector;
use jig_ops_domain_infrastructure as infrastructure;
use jig_ops_domain_observability as observability;
use skein_lib::{Decision, Env, List, Queue, ReplyTo, Token};

pub(crate) fn work(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision<Write, Output>,
    item: Work,
    work: &mut Queue<Work>,
) {
    match item {
        Work::Event(event) => event_in(domain, env, decision, event, work),
        Work::Core(event) => {
            let requests = core::step(&mut domain.core, &env.with(env.limits.core), event);
            core_out(domain, env, decision, requests, work);
        }
        Work::Effect { owner, origin, effect } => effect_start(domain, env, decision, owner, origin, effect, work),
        Work::Finish => {}
        Work::ResumeFleet => {
            let requests = core::resume_fleet(&mut domain.core, &env.with(env.limits.core));
            core_out(domain, env, decision, requests, work);
            let mut hub_out = Queue::uncapped();
            host::resume(&mut domain.host, &env.with(env.limits.host), &mut hub_out);
            crate::link::host_out(domain, env, decision, &mut hub_out, work);
        }
        Work::Infrastructure(event) => {
            let mut out = Queue::uncapped();
            infrastructure::step(&mut domain.infrastructure, &env.with(env.limits.infrastructure), event, &mut out);
            for _ in 0..out.len() {
                let request = out.pop().expect("connector output count");
                connector_out(domain, env, decision, ConnectorRequest::Infrastructure(request), work);
            }
        }
        Work::Observability(event) => {
            let mut out = Queue::uncapped();
            observability::step(&mut domain.observability, &env.with(env.limits.observability), event, &mut out);
            for _ in 0..out.len() {
                let request = out.pop().expect("connector output count");
                connector_out(domain, env, decision, ConnectorRequest::Observability(request), work);
            }
        }
        Work::Host(event) => {
            let mut out = Queue::uncapped();
            host::step(&mut domain.host, &env.with(env.limits.host), event, &mut out);
            crate::link::host_out(domain, env, decision, &mut out, work);
        }
        Work::Start(action) => crate::restart::route(domain, env, decision, action, work),
        Work::StartDone(phase) => crate::restart::done(domain, env, decision, phase, work),
    }
}

fn effect_start(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision<Write, Output>,
    owner: Token,
    origin: core::EffectOrigin,
    effect: infrastructure::Effect,
    work: &mut Queue<Work>,
) {
    let mut requests = core::step(
        &mut domain.core,
        &env.with(env.limits.core),
        core::Event::EffectStart {
            owner,
            connector: jig_ops_domain_connectors::number(Connector::Infrastructure),
            origin,
        },
    );
    let mut effect = Some(effect);
    // The core selects description or refusal; the typed input moves once.
    for _ in 0..requests.len() {
        match requests.pop().expect("effect start outputs") {
            core::Request::Ask { connector, ask: core::Ask::Effect(connector::Ask::Describe { owner: asked }) } => {
                assert!(
                    asked == owner && connector == jig_ops_domain_connectors::number(Connector::Infrastructure),
                    "effect input owner"
                );
                work.push(Work::Infrastructure(infrastructure::Event::Describe {
                    token: owner,
                    effect: effect.take().expect("one description request"),
                }));
            }
            request @ (core::Request::Write(_)
            | core::Request::Held(_)
            | core::Request::Now(_)
            | core::Request::Ask { .. }
            | core::Request::ProjectMade { .. }) => {
                let mut one = Queue::uncapped();
                one.push(request);
                core_out(domain, env, decision, one, work);
            }
        }
    }
}

#[expect(clippy::too_many_lines, reason = "the application input vocabulary is routed exhaustively")]
fn event_in(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision<Write, Output>,
    event: Event,
    work: &mut Queue<Work>,
) {
    match event {
        Event::WatchApplication { watcher, sign_in, scope, subject, window } => {
            let token = domain.token();
            let number = crate::client_documents::prepare(domain, token, subject, work);
            work.push(Work::Core(core::Event::Watch {
                watcher,
                sign_in,
                subject: core::documents::Watched::Application { scope, connector: number, subject: token },
                window,
            }));
        }
        Event::ReadApplication { reply, sign_in, scope, subject } => {
            let token = domain.token();
            let number = crate::client_documents::prepare(domain, token, subject, work);
            work.push(Work::Core(core::Event::Read {
                reply,
                sign_in,
                read: core::reads::Read::Application {
                    scope,
                    connector: number,
                    read: token,
                    window: core::documents::Window { rows: env.limits.core.documents.rows, after: None },
                },
            }));
        }
        Event::Application { reply_to, sign_in, key, project, digest, value } => {
            let owner = domain.token();
            work.push(Work::Infrastructure(infrastructure::Event::ApplicationPrepare { owner, value }));
            work.push(Work::Core(core::Event::Application {
                reply_to,
                sign_in,
                key,
                scope: jig_domain_people::ApplicationScope::Project(project),
                connector: jig_ops_domain_connectors::number(Connector::Infrastructure),
                digest,
                owner,
                carried: None,
            }));
        }
        Event::Core(event) => work.push(Work::Core(event)),
        Event::Infrastructure(event) => work.push(Work::Infrastructure(event)),
        Event::Observability(event) => work.push(Work::Observability(event)),
        Event::Host(event) => work.push(Work::Host(event)),
        Event::Restart { pending } => {
            domain.start.state = crate::domain::StartState::Starting;
            let request = domain.core.start_begin(pending, env.wall);
            work.push(Work::Start(request));
        }
        Event::Store(store) => crate::restart::store(domain, env, decision, store, work),
        Event::Released(released) => match released {
            Released::Core(event) => work.push(Work::Core(event)),
            Released::Fleet(event) => work.push(Work::Core(core::Event::Fleet(event))),
            Released::Host(event) => work.push(Work::Host(event)),
            Released::Infrastructure(event) => work.push(Work::Infrastructure(event)),
            Released::Observability(event) => work.push(Work::Observability(event)),
            Released::Start(continuation) => {
                if !domain.core.stopping() {
                    domain.start.waiting = false;
                    let mut out = Queue::uncapped();
                    let action = domain.core.start_resume(&env.with(env.limits.core), continuation, &mut out);
                    core_out(domain, env, decision, out, work);
                    work.push(Work::Start(action));
                }
            }
            Released::Procedure { task, step: _, connector, code: _ } => {
                match jig_ops_domain_connectors::connector(connector) {
                    Some(Connector::Infrastructure) => {
                        work.push(Work::Infrastructure(infrastructure::Event::Procedure {
                            task,
                            signal: infrastructure::ProcedureSignal::Step,
                        }));
                    }
                    Some(Connector::Observability) | None => {}
                }
            }
        },
        Event::ResultRead { payload, end } => {
            crate::link::route(domain, env, decision, LinkEvent::ResultRead { payload, end }, false, work);
        }
        Event::CallAnswered { to, call } => {
            crate::link::route(domain, env, decision, LinkEvent::SettledCall { to, call }, false, work);
        }
        Event::ConnectorRead { to, task, name: _, tool, read } => {
            crate::assemble::connector_read(domain, env, to, task, tool, read, work);
        }
        Event::Effect { to, key, effect, deadline, proposal } => {
            let owner = domain.core.effect_owner();
            let origin = match proposal {
                Some(reason) => core::EffectOrigin::Propose { to, key, reason, as_holder: false },
                None => core::EffectOrigin::Call { to, key, deadline },
            };
            work.push(Work::Effect { owner, origin, effect });
        }
        Event::Timer(timer) => {
            let requests = core::fire(&mut domain.core, &env.with(env.limits.core), timer);
            core_out(domain, env, decision, requests, work);
        }
        Event::TurnTimer => crate::link::fire(domain, env, decision, work),
        Event::InfrastructureTimer => {
            let mut out = Queue::uncapped();
            infrastructure::fire(&mut domain.infrastructure, &env.with(env.limits.infrastructure), &mut out);
            for _ in 0..out.len() {
                let request = out.pop().expect("connector output count");
                connector_out(domain, env, decision, ConnectorRequest::Infrastructure(request), work);
            }
        }
        Event::ObservabilityTimer => {
            let mut out = Queue::uncapped();
            observability::fire(&mut domain.observability, &env.with(env.limits.observability), &mut out);
            for _ in 0..out.len() {
                let request = out.pop().expect("connector output count");
                connector_out(domain, env, decision, ConnectorRequest::Observability(request), work);
            }
        }
        Event::Agent(report) => work.push(Work::Host(host::Event::Agent(report))),
        Event::AgentContent { owner, content } => {
            crate::link::agent_content(domain, env, decision, work, owner, content);
        }
    }
}

pub(crate) fn core_out(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision<Write, Output>,
    requests: Queue<core::Request>,
    work: &mut Queue<Work>,
) {
    let mut out = requests;
    for _ in 0..out.len() {
        match out.pop().expect("core request count") {
            core::Request::Write(core::Write::DropTranscript { task }) => {
                write(domain, decision, Write::DropTranscript { task });
            }
            core::Request::Write(core::Write::CopySettingsRevision(number)) => {
                write(domain, decision, Write::CopySettingsRevision(number));
            }
            core::Request::Write(core::Write::Save(record)) => {
                write(domain, decision, Write::Save(Record::Core(record)));
            }
            core::Request::Write(core::Write::Erase(key)) => write(domain, decision, Write::Erase(Key::Core(key))),
            core::Request::Held(value) => core_held(domain, env, decision, *value, work),
            core::Request::Now(value) => core_now(domain, env, decision, *value, work),
            core::Request::Ask { connector, ask } => core_ask(domain, env, decision, connector, ask, work),
            core::Request::ProjectMade { project } => held(decision, Output::ProjectMade { project }),
        }
    }
}
#[expect(clippy::too_many_lines, reason = "one dispatcher preserves the full held output vocabulary")]
fn core_held(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision<Write, Output>,
    value: core::Held,
    work: &mut Queue<Work>,
) {
    let value = match jig_domain_hub_link::down(&mut domain.hub_link, value) {
        Ok((channel, value)) => {
            assert!(channel == Token::new(0), "ops engine's permanent link");
            child(decision, Released::Host(host::Event::Down(value)));
            return;
        }
        Err(value) => value,
    };
    match value {
        core::Held::Load(load) => held(decision, Output::CoreLoad(*load)),
        core::Held::ApplicationDelivery { .. } => unreachable!("document assembly owns this synchronous output"),
        core::Held::ApplicationWatchEnded { watcher, connector, subject, why } => {
            crate::client_documents::forget(domain, connector, subject, work);
            held(decision, Output::Core(core::Held::WatchEnded { watcher, why }));
        }
        core::Held::Applied { to, connector, token } => {
            assert!(
                connector == jig_ops_domain_connectors::number(Connector::Infrastructure),
                "application request owner"
            );
            let outcome = domain
                .infrastructure
                .application_outcome(token)
                .expect("kept people token has its retained connector outcome");
            held(decision, Output::Application { to, outcome });
        }
        core::Held::Acted { to, outcome } => {
            let token = to.into_token();
            let key = domain.hub_link.take_close_reply(token).expect("infrastructure act reply");
            let translated = match outcome.act_status() {
                jig_domain_people::ActStatus::Closed => infrastructure::CloseOutcome::Closed,
                jig_domain_people::ActStatus::Again => infrastructure::CloseOutcome::Again,
                jig_domain_people::ActStatus::NotOffered => infrastructure::CloseOutcome::NotOffered,
                jig_domain_people::ActStatus::NoOwner => infrastructure::CloseOutcome::NoOwner,
                jig_domain_people::ActStatus::Ended => infrastructure::CloseOutcome::Ended,
                jig_domain_people::ActStatus::Refused => infrastructure::CloseOutcome::Refused,
            };
            // Keep the complete answer as a report fact, including saved
            // refusals; the connector owns retention and retry decisions.
            held(decision, Output::Core(core::Held::Acted { to: ReplyTo::new(token), outcome }));
            child(decision, Released::Infrastructure(infrastructure::Event::Closed { key, outcome: translated }));
        }
        core::Held::MakeEffect { connector, entry } => match jig_ops_domain_connectors::connector(connector) {
            Some(Connector::Infrastructure) => {
                child(decision, Released::Infrastructure(infrastructure::Event::MakeEntry { entry }));
            }
            Some(Connector::Observability) => {
                child(decision, Released::Observability(observability::Event::MakeEntry { entry }));
            }
            None => {}
        },
        core::Held::Assign { channel, run, attempt, .. } => {
            crate::link::route(domain, env, decision, LinkEvent::Assign { channel, run, attempt }, true, work);
        }
        core::Held::SettledCall { to, key: _, call } => {
            crate::link::route(domain, env, decision, LinkEvent::SettledCall { to, call }, false, work);
        }
        core::Held::Relayed { channel, run, attempt, call, answer } => crate::link::route(
            domain,
            env,
            decision,
            LinkEvent::Relayed { channel, run, attempt, call, answer },
            true,
            work,
        ),
        core::Held::Inbound { channel, run, attempt, message } => {
            crate::link::route(
                domain,
                env,
                decision,
                LinkEvent::Inbound { channel, run, attempt, message },
                true,
                work,
            );
        }
        core::Held::Relay { task, attempt, word, .. } => {
            let name = Token::new(word.number);
            let words = domain.hub_link.retain(jig_domain_hub_link::Payload::InboxWord(word));
            child(
                decision,
                Released::Fleet(fleet::Event::Inbound {
                    run: Token::new(task),
                    attempt: Token::new(attempt),
                    message: fleet::Message { name, sender: Token::new(0), words },
                }),
            );
        }
        core::Held::TaskTurnKept { task: run, attempt, turn } => {
            child(
                decision,
                Released::Fleet(fleet::Event::TurnKept { run: Token::new(run), attempt: Token::new(attempt), turn }),
            );
        }
        core::Held::TurnBusy { channel, run, attempt, turn } if channel == Token::new(0) => {
            crate::link::route(
                domain,
                env,
                decision,
                LinkEvent::TurnBusy { run, attempt, turn, retained: domain.host.has_turn(run, attempt, turn) },
                false,
                work,
            );
        }
        core::Held::StopRun { task, attempt } => child(
            decision,
            Released::Fleet(fleet::Event::Cancel { run: Token::new(task), attempt: Token::new(attempt) }),
        ),
        core::Held::TaskTerminalAcknowledged { task, attempt } => child(
            decision,
            Released::Fleet(fleet::Event::Acknowledge { run: Token::new(task), attempt: Token::new(attempt) }),
        ),
        other @ (core::Held::CallAnswer { .. }
        | core::Held::ReadLoad { .. }
        | core::Held::Read { .. }
        | core::Held::Delivery { .. }
        | core::Held::WatchEnded { .. }
        | core::Held::PeopleReply { .. }
        | core::Held::CarriedUnused { .. }
        | core::Held::Result { .. }
        | core::Held::Drained
        | core::Held::Grant { .. }
        | core::Held::AcknowledgeTurn { .. }
        | core::Held::Cancel { .. }
        | core::Held::Acknowledge { .. }
        | core::Held::Sources { .. }
        | core::Held::CheckSource { .. }
        | core::Held::SignOutSource { .. }
        | core::Held::SourceSignedOut { .. }
        | core::Held::Refuse { .. }
        | core::Held::TurnBusy { .. }
        | core::Held::NotesLoad { .. }
        | core::Held::NotesWritten { .. }
        | core::Held::NotesDeleted { .. }) => held(decision, Output::Core(other)),
    }
}

#[expect(clippy::too_many_lines, reason = "one exhaustive dispatcher translates this child boundary")]
fn core_ask(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision<Write, Output>,
    number: u16,
    ask: core::Ask,
    work: &mut Queue<Work>,
) {
    let destination = jig_ops_domain_connectors::connector(number);
    match (destination, ask) {
        (Some(Connector::Infrastructure | Connector::Observability) | None, core::Ask::ReleaseProject { project }) => {
            work.push(Work::Core(core::Event::ProjectReleased { connector: number, project }));
        }
        (
            Some(Connector::Infrastructure | Connector::Observability) | None,
            core::Ask::StepMade { task, step, first: Some(first), .. },
        ) => {
            crate::steps::made(number, task, step, Some(first), work);
        }
        (
            Some(Connector::Infrastructure | Connector::Observability) | None,
            core::Ask::StepRefused { task, step, .. },
        ) => crate::steps::made(number, task, step, None, work),
        (Some(Connector::Infrastructure | Connector::Observability) | None, core::Ask::DropDocument { subject }) => {
            crate::client_documents::forget(domain, number, subject, work);
        }
        (
            Some(Connector::Infrastructure | Connector::Observability) | None,
            core::Ask::Document { watcher, scope, subject, party, window: _ },
        ) => {
            crate::client_documents::ask_document(domain, number, watcher, scope, subject, party, work);
        }
        (
            Some(Connector::Infrastructure | Connector::Observability) | None,
            core::Ask::Read { reply, scope, read, party, window: _ },
        ) => {
            crate::client_documents::read(domain, env, decision, number, reply, scope, read, party, work);
        }
        (
            Some(Connector::Infrastructure | Connector::Observability) | None,
            core::Ask::Application { request, token, owner, person, scope, role, retention },
        ) => {
            let (jig_domain_people::ApplicationScope::Project(project), Some(role)) = (scope, role) else {
                work.push(Work::Core(core::Event::People(jig_domain_people::Event::Decided {
                    request,
                    outcome: jig_domain_people::Outcome::Refused(jig_domain_people::Refusal::NotOffered),
                })));
                return;
            };
            assert!(number == jig_ops_domain_connectors::number(Connector::Infrastructure), "application owner");
            work.push(Work::Infrastructure(infrastructure::Event::Application {
                request,
                token,
                owner,
                person,
                project,
                role: role.number(),
                retention,
            }));
        }
        (Some(Connector::Infrastructure), core::Ask::DropApplication { owner }) => {
            work.push(Work::Infrastructure(infrastructure::Event::ApplicationDrop { owner }));
        }
        (
            Some(Connector::Infrastructure | Connector::Observability) | None,
            core::Ask::ApplicationPartAnswered { .. },
        ) => unreachable!("infrastructure requests ask for no core part"),
        (Some(Connector::Infrastructure | Connector::Observability) | None, core::Ask::Effect(ask)) => {
            effect_ask(domain, env, number, ask, work);
        }
        (
            Some(Connector::Infrastructure | Connector::Observability) | None,
            core::Ask::TaskHoldings { request, executor, spec, .. },
        ) => work.push(Work::Core(core::Event::Holdings {
            refused: None,
            request,
            connector: number,
            holdings: connector_holdings(env, number, executor, &spec),
        })),
        (
            Some(Connector::Infrastructure | Connector::Observability) | None,
            core::Ask::DelegateHoldings { request, members, .. },
        ) => {
            let mut holdings = List::with_capacity(u32::try_from(members.len()).expect("bounded batch"));
            let mut valid = true;
            for member in members {
                match connector_holdings(env, number, member.executor, &member.spec) {
                    Some(value) => holdings.push(value).expect("one holdings row"),
                    None => valid = false,
                }
            }
            let holdings = if valid { Some(holdings.into_boxed()) } else { None };
            work.push(Work::Core(core::Event::DelegateHoldings {
                refused: None,
                request,
                connector: number,
                holdings,
            }));
        }
        (
            Some(Connector::Infrastructure | Connector::Observability) | None,
            core::Ask::ProcedureHoldings { task, step, members },
        ) => {
            let mut holdings = List::with_capacity(u32::try_from(members.len()).expect("bounded batch"));
            let mut valid = true;
            for member in members {
                match connector_holdings(env, number, member.executor, &member.spec) {
                    Some(value) => holdings.push(value).expect("one holdings row"),
                    None => valid = false,
                }
            }
            let holdings = if valid { Some(holdings.into_boxed()) } else { None };
            work.push(Work::Core(core::Event::ProcedureHoldings {
                refused: None,
                task,
                step,
                connector: number,
                holdings,
            }));
        }
        (
            Some(Connector::Infrastructure | Connector::Observability) | None,
            core::Ask::StartProcedure { context, step },
        ) => {
            let task = context.task;
            let code = match context.executor {
                tasks::Executor::Procedure { code, .. } => code,
                tasks::Executor::Agent { .. } | tasks::Executor::Person(_) => {
                    unreachable!("core-selected procedure executor")
                }
            };
            domain.hub_link.keep_procedure(task, Procedure { step, connector: number, code });
            match destination {
                Some(Connector::Infrastructure) => {
                    match infrastructure::procedure(
                        &env.limits.infrastructure.vocabulary,
                        code,
                        &translate::infrastructure_spec(&context.spec),
                        domain.core.counters.deployment().id,
                    ) {
                        Some(procedure) => {
                            work.push(Work::Infrastructure(infrastructure::Event::StartProcedure {
                                task,
                                project: context.project,
                                chat: domain.core.tasks.served_chat(task),
                                pending: context.pending,
                                added: translate::infrastructure_added(context.added),
                                procedure,
                            }));
                        }
                        None => work.push(Work::Core(core::Event::ProcedureStep {
                            task,
                            step,
                            connector: number,
                            code,
                            decision: core::ProcedureStep::new(core::StepEnd::Hold(tasks::Hold::Procedure)),
                        })),
                    }
                    child(decision, Released::Procedure { task, step, connector: number, code });
                }
                Some(Connector::Observability) => {
                    match observability::watch_spec(
                        &translate::observability_spec(&context.spec),
                        &env.limits.observability.vocabulary,
                    ) {
                        Some(spec) => {
                            work.push(Work::Observability(observability::Event::StartWatch {
                                task,
                                project: context.project,
                                services: Box::new([spec.service]),
                                template: spec.template,
                            }));
                            if let Some(row) = domain.core.tasks.task(task) {
                                for subscription in &row.subscriptions {
                                    match subscription.kind {
                                        tasks::SubscriptionKind::Topic { connector, topic } => {
                                            if connector == number && topic == task {
                                                work.push(Work::Observability(observability::Event::LinkWatch {
                                                    task,
                                                    subscription: subscription.number,
                                                }));
                                            }
                                        }
                                        tasks::SubscriptionKind::Task { .. }
                                        | tasks::SubscriptionKind::Timer { .. } => {}
                                    }
                                }
                            }
                            let messages = observability::watch_messages(
                                translate::watch_inputs(context.inbox),
                                &env.limits.observability,
                            );
                            work.push(Work::Observability(observability::Event::WatchStep {
                                task,
                                step,
                                batch: context.last_message,
                                messages,
                            }));
                        }
                        None => work.push(Work::Core(core::Event::ProcedureStep {
                            task,
                            step,
                            connector: number,
                            code,
                            decision: core::ProcedureStep::new(core::StepEnd::Hold(tasks::Hold::Procedure)),
                        })),
                    }
                }
                None => {}
            }
        }
        (Some(Connector::Infrastructure), core::Ask::ClaimDone { task, attempt, kinds, version }) => {
            crate::link::route(domain, env, decision, LinkEvent::Start { task, attempt, kinds, version }, true, work);
        }
        (Some(Connector::Infrastructure), core::Ask::Close { task, .. }) => {
            work.push(Work::Infrastructure(infrastructure::Event::Close { task }));
        }
        (Some(Connector::Observability), core::Ask::Close { task, .. }) => {
            work.push(Work::Observability(observability::Event::Close { task }));
        }
        (Some(Connector::Infrastructure), core::Ask::Release { task, .. }) => {
            work.push(Work::Infrastructure(infrastructure::Event::Release { task }));
        }
        (Some(Connector::Observability), core::Ask::Release { task, .. }) => {
            work.push(Work::Observability(observability::Event::Release { task }));
        }
        (
            Some(Connector::Infrastructure | Connector::Observability) | None,
            core::Ask::Gather { section, .. } | core::Ask::CutTo { section, .. },
        ) => {
            work.push(Work::Core(core::Event::EffectConnector(connector::Event::SectionReady { section, size: None })));
        }
        (Some(Connector::Infrastructure | Connector::Observability) | None, core::Ask::ProjectGoal { feed }) => {
            // These two connectors have no goal projection. Their closing
            // hand-off has no pending writes to settle.
            if feed.closing {
                work.push(Work::Core(core::Event::ProjectionSettled { goal: feed.goal.number, connector: number }));
            }
        }
        (Some(Connector::Observability), core::Ask::SubscriptionDone { request: _, key, subscription }) => {
            work.push(Work::Observability(observability::Event::LinkWatch { task: key.task, subscription }));
        }
        (Some(Connector::Infrastructure), core::Ask::SubscriptionDone { request, key, subscription }) => {
            work.push(Work::Core(core::Event::NamedAnswer {
                to: ReplyTo::new(request),
                key,
                part: core::CallPart::Subscribed { subscription },
            }));
        }
        (Some(Connector::Infrastructure), core::Ask::UnsubscriptionDone { request, key }) => {
            work.push(Work::Core(core::Event::NamedAnswer {
                to: ReplyTo::new(request),
                key,
                part: core::CallPart::Unsubscribed,
            }));
        }
        (
            Some(Connector::Observability) | None,
            core::Ask::DropApplication { .. } | core::Ask::ClaimDone { .. } | core::Ask::UnsubscriptionDone { .. },
        )
        | (None, core::Ask::Close { .. } | core::Ask::Release { .. } | core::Ask::SubscriptionDone { .. })
        | (
            Some(Connector::Infrastructure | Connector::Observability) | None,
            core::Ask::StepMade { first: None, .. }
            | core::Ask::ForgetProjection { .. }
            | core::Ask::Adopt { .. }
            | core::Ask::Hold { .. }
            | core::Ask::EndTopic { .. }
            | core::Ask::Drop { .. }
            | core::Ask::Lost { .. }
            | core::Ask::DropSubscription { .. }
            | core::Ask::RepairRefused { .. }
            | core::Ask::DelegateRefused { .. },
        ) => {}
    }
}

#[expect(clippy::too_many_lines, reason = "one exhaustive dispatcher translates this child boundary")]
fn effect_ask(domain: &mut Domain, env: &Env<Limits>, number: u16, ask: connector::Ask, work: &mut Queue<Work>) {
    let destination = jig_ops_domain_connectors::connector(number);
    match (destination, ask) {
        (Some(Connector::Infrastructure | Connector::Observability) | None, connector::Ask::Describe { owner }) => {
            work.push(Work::Core(core::Event::EffectConnector(connector::Event::DescribeRefused { owner })));
        }
        (
            Some(Connector::Infrastructure | Connector::Observability) | None,
            connector::Ask::DescribeProposal { owner, proposal },
        ) => {
            work.push(Work::Infrastructure(infrastructure::Event::DescribeProposal { token: owner, number: proposal }));
        }
        (
            Some(Connector::Infrastructure | Connector::Observability) | None,
            connector::Ask::KeepProposal { owner, proposal, task, project: _ },
        ) => {
            let _removed = domain.hub_link.take_effect_procedure(owner);
            work.push(Work::Infrastructure(infrastructure::Event::KeepProposal {
                token: owner,
                number: proposal,
                task,
            }));
        }
        (
            Some(Connector::Infrastructure | Connector::Observability) | None,
            connector::Ask::DropProposal { proposal },
        ) => {
            work.push(Work::Infrastructure(infrastructure::Event::DropProposal { number: proposal }));
        }
        (
            Some(Connector::Infrastructure | Connector::Observability) | None,
            connector::Ask::Keep { owner, entry, key, .. },
        ) => {
            let _removed = domain.hub_link.take_effect_procedure(owner);
            match destination {
                Some(Connector::Infrastructure) => {
                    work.push(Work::Infrastructure(infrastructure::Event::Keep {
                        token: owner,
                        entry,
                        key: translate::infrastructure_key(key),
                        project: match domain.core.tasks.task(key.task) {
                            Some(task) => Some(task.project),
                            None => None,
                        },
                    }));
                }
                Some(Connector::Observability) => {
                    work.push(Work::Observability(observability::Event::Keep {
                        token: owner,
                        entry,
                        key: translate::observability_key(key),
                        project: match domain.core.tasks.task(key.task) {
                            Some(task) => Some(task.project),
                            None => None,
                        },
                    }));
                }
                None => {}
            }
        }
        (
            Some(Connector::Infrastructure | Connector::Observability) | None,
            connector::Ask::Drop { owner, answer, .. },
        ) => {
            if let Some(task) = domain.hub_link.take_effect_procedure(owner) {
                work.push(Work::Infrastructure(infrastructure::Event::ProcedureAdmission {
                    task,
                    answer: translate::admission(answer),
                }));
            }
            match destination {
                Some(Connector::Infrastructure) => {
                    work.push(Work::Infrastructure(infrastructure::Event::Drop { token: owner }));
                }
                Some(Connector::Observability) => {
                    work.push(Work::Observability(observability::Event::Drop { token: owner }));
                }
                None => {}
            }
        }
        (
            Some(Connector::Infrastructure | Connector::Observability) | None,
            connector::Ask::Judge { owner, judge, state, resources },
        ) => {
            let token = domain.token();
            match translate::service(&resources) {
                Some(service) => match translate::requirement(judge.requirement, &env.limits.policy) {
                    Some(requirement) => {
                        domain.hub_link.keep_judge(token, (owner, judge, state));
                        work.push(Work::Observability(observability::Event::Judge {
                            token,
                            requirement,
                            service,
                            freshness: env.limits.policy.verdict_freshness_seconds,
                        }));
                    }
                    None => work.push(Work::Core(core::Event::EffectConnector(connector::Event::Verdict {
                        owner,
                        judge,
                        verdict: authority::Verdict::Refuse,
                        at: env.wall,
                        guarded: false,
                        state,
                    }))),
                },
                None => work.push(Work::Core(core::Event::EffectConnector(connector::Event::Verdict {
                    owner,
                    judge,
                    verdict: authority::Verdict::Refuse,
                    at: env.wall,
                    guarded: false,
                    state,
                }))),
            }
        }
        (Some(Connector::Infrastructure), connector::Ask::ResolveUncertain { entry, again }) => {
            work.push(Work::Infrastructure(infrastructure::Event::ResolveEntry { entry, repeat: again }));
        }
        (Some(Connector::Infrastructure), connector::Ask::Make { entry }) => {
            work.push(Work::Infrastructure(infrastructure::Event::MakeEntry { entry }));
        }
        (Some(Connector::Observability), connector::Ask::Make { entry }) => {
            work.push(Work::Observability(observability::Event::MakeEntry { entry }));
        }
        (Some(Connector::Infrastructure), connector::Ask::ReadAfresh) => {
            work.push(Work::Infrastructure(infrastructure::Event::ReadAfresh));
        }
        (Some(Connector::Observability), connector::Ask::ReadAfresh) => {
            work.push(Work::Observability(observability::Event::ReadAfresh));
        }
        (Some(Connector::Infrastructure), connector::Ask::SettleOutbox) => {
            work.push(Work::Infrastructure(infrastructure::Event::Restart));
        }
        (Some(Connector::Observability), connector::Ask::SettleOutbox) => {
            work.push(Work::Observability(observability::Event::Restart));
        }
        (Some(Connector::Observability) | None, connector::Ask::ResolveUncertain { .. })
        | (None, connector::Ask::Make { .. } | connector::Ask::ReadAfresh | connector::Ask::SettleOutbox)
        | (
            Some(Connector::Infrastructure | Connector::Observability) | None,
            connector::Ask::Names { .. }
            | connector::Ask::Unname { .. }
            | connector::Ask::Hold { .. }
            | connector::Ask::ReleaseHold { .. }
            | connector::Ask::Writer { .. }
            | connector::Ask::ProcedureMade { .. }
            | connector::Ask::ProcedureActivate { .. }
            | connector::Ask::ProcedureMessage { .. }
            | connector::Ask::ProcedureClose { .. }
            | connector::Ask::Project { .. }
            | connector::Ask::Release { .. }
            | connector::Ask::Subscribe { .. }
            | connector::Ask::Unsubscribe { .. }
            | connector::Ask::Read { .. }
            | connector::Ask::Gather { .. }
            | connector::Ask::CutTo { .. }
            | connector::Ask::Take { .. }
            | connector::Ask::Prepare { .. }
            | connector::Ask::Left { .. }
            | connector::Ask::Adopt { .. }
            | connector::Ask::Restore { .. },
        ) => {}
    }
}

#[derive(Debug)]
pub(crate) enum ConnectorRequest {
    Infrastructure(infrastructure::Request),
    Observability(observability::Request),
}

#[expect(clippy::too_many_lines, reason = "one exhaustive dispatcher translates both connector vocabularies")]
pub(crate) fn connector_out(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision<Write, Output>,
    request: ConnectorRequest,
    work: &mut Queue<Work>,
) {
    let number = match &request {
        ConnectorRequest::Infrastructure(..) => jig_ops_domain_connectors::number(Connector::Infrastructure),
        ConnectorRequest::Observability(..) => jig_ops_domain_connectors::number(Connector::Observability),
    };
    match request {
        ConnectorRequest::Infrastructure(infrastructure::Request::Document { watcher, subject, document }) => {
            let document = match document {
                Ok(document) => Ok(crate::ClientDocument::Infrastructure(Box::new(document))),
                Err(why) => Err(match why {
                    infrastructure::DocumentMissing::Unknown => jig_domain_views::End::Gone,
                    infrastructure::DocumentMissing::NotAllowed => jig_domain_views::End::NotAllowed,
                }),
            };
            crate::client_documents::document(domain, env, decision, number, watcher, subject, document, work);
        }
        ConnectorRequest::Infrastructure(infrastructure::Request::ReadAnswered { .. })
        | ConnectorRequest::Observability(observability::Request::ReadAnswered { .. }) => {
            unreachable!("client read completes in its synchronous route")
        }
        ConnectorRequest::Infrastructure(infrastructure::Request::Marked { subjects })
        | ConnectorRequest::Observability(observability::Request::Marked { subjects }) => {
            work.push(Work::Core(core::Event::Marked { connector: number, subjects }));
        }
        ConnectorRequest::Infrastructure(infrastructure::Request::ApplicationDone { request, token }) => {
            work.push(Work::Core(core::Event::ApplicationFinished { request, outcome: Ok(token) }));
        }
        ConnectorRequest::Infrastructure(infrastructure::Request::ApplicationBusy { request }) => {
            work.push(Work::Core(core::Event::ApplicationFinished {
                request,
                outcome: Err(jig_domain_people::Again::Busy),
            }));
        }
        ConnectorRequest::Infrastructure(infrastructure::Request::Closed { task })
        | ConnectorRequest::Observability(observability::Request::Closed { task }) => {
            work.push(Work::Core(core::Event::EffectConnector(connector::Event::Closed { task, connector: number })));
        }
        ConnectorRequest::Infrastructure(infrastructure::Request::Released { task })
        | ConnectorRequest::Observability(observability::Request::Released { task }) => {
            work.push(Work::Core(core::Event::Tasks(tasks::Event::Settled { task })));
        }
        ConnectorRequest::Infrastructure(infrastructure::Request::Save { record }) => {
            write(domain, decision, Write::Save(Record::Infrastructure(record)));
        }
        ConnectorRequest::Infrastructure(infrastructure::Request::Erase { key }) => {
            write(domain, decision, Write::Erase(Key::Infrastructure(key)));
        }
        ConnectorRequest::Infrastructure(infrastructure::Request::System(request)) => {
            held(decision, Output::Infrastructure(request));
        }
        ConnectorRequest::Infrastructure(infrastructure::Request::Described {
            token,
            description,
            waits_for_dependencies,
        }) => {
            work.push(Work::Core(core::Event::EffectConnector(connector::Event::Described {
                owner: token,
                description: Box::new(translate::infrastructure_description(
                    number,
                    description,
                    waits_for_dependencies,
                )),
            })));
        }
        ConnectorRequest::Infrastructure(infrastructure::Request::Refused { token })
        | ConnectorRequest::Observability(observability::Request::Refused { token }) => {
            if let Some(call) = domain.hub_link.take_read_call(token) {
                crate::assemble::read_answer(domain, call, true, Box::from(&b"unavailable"[..]), work);
            } else {
                work.push(Work::Core(core::Event::EffectConnector(connector::Event::DescribeRefused { owner: token })));
            }
        }
        ConnectorRequest::Infrastructure(infrastructure::Request::Make { key }) => {
            child(decision, Released::Infrastructure(infrastructure::Event::Make { key }));
        }
        ConnectorRequest::Infrastructure(infrastructure::Request::Outcome { entry, key, outcome }) => {
            work.push(Work::Core(core::Event::EffectConnector(connector::Event::Outbox {
                connector: number,
                entry,
                task: key.task,
                outcome: translate::infrastructure_outcome(outcome),
            })));
            if let Some(event) = infrastructure::procedure_outcome(key, outcome) {
                work.push(Work::Infrastructure(event));
            }
        }
        ConnectorRequest::Infrastructure(infrastructure::Request::Step { task, decision: action }) => {
            let Some(Procedure { step, connector, code }) = domain.hub_link.take_procedure(task) else { return };
            match action {
                infrastructure::StepDecision::Effect(effect) => {
                    let owner = domain.core.effect_owner();
                    domain.hub_link.keep_effect_procedure(owner, task);
                    work.push(Work::Effect {
                        owner,
                        origin: core::EffectOrigin::Procedure { task, step, entry: None },
                        effect,
                    });
                }
                infrastructure::StepDecision::Wait { .. } => work.push(Work::Core(core::Event::ProcedureStep {
                    task,
                    step,
                    connector,
                    code,
                    decision: core::ProcedureStep::new(core::StepEnd::Wait),
                })),
                infrastructure::StepDecision::Finish => work.push(Work::Core(core::Event::ProcedureStep {
                    task,
                    step,
                    connector,
                    code,
                    decision: core::ProcedureStep::new(core::StepEnd::Result(tasks::TaskResult::Report {
                        words: Box::new([]),
                    })),
                })),
                infrastructure::StepDecision::FinishReport { words } => {
                    work.push(Work::Core(core::Event::ProcedureStep {
                        task,
                        step,
                        connector,
                        code,
                        decision: core::ProcedureStep::new(core::StepEnd::Result(tasks::TaskResult::Report { words })),
                    }));
                }
                infrastructure::StepDecision::Hold { .. } => work.push(Work::Core(core::Event::ProcedureStep {
                    task,
                    step,
                    connector,
                    code,
                    decision: core::ProcedureStep::new(core::StepEnd::Hold(tasks::Hold::Effects)),
                })),
            }
        }
        ConnectorRequest::Infrastructure(infrastructure::Request::Drift { task, resource }) => {
            work.push(Work::Core(core::Event::EffectConnector(connector::Event::Drift {
                task,
                resource: tasks::Name { connector: number, path: resource.segments() },
            })));
        }
        ConnectorRequest::Infrastructure(infrastructure::Request::Slots { pool, quota, used: _ }) => {
            work.push(Work::Core(core::Event::EffectConnector(connector::Event::PoolSlots {
                name: tasks::Name { connector: number, path: infrastructure::Resource::Pool(pool).segments() },
                slots: quota,
            })));
        }
        ConnectorRequest::Infrastructure(infrastructure::Request::Wake { task }) => {
            work.push(Work::Core(core::Event::Tasks(tasks::Event::WakeProcedure { task })));
        }
        ConnectorRequest::Infrastructure(infrastructure::Request::Changed { resource }) => {
            work.push(Work::Infrastructure(infrastructure::Event::ResourceChanged { resource }));
        }
        ConnectorRequest::Infrastructure(infrastructure::Request::Named { project, resources, .. }) => {
            for named in resources {
                let hold = match named.hold {
                    infrastructure::Hold::None => connector::HoldKind::Shared,
                    infrastructure::Hold::ExclusiveWait => connector::HoldKind::Exclusive { wait: true },
                    infrastructure::Hold::PooledWait => connector::HoldKind::Pooled { slots: 0, wait: true },
                };
                work.push(Work::Core(core::Event::EffectConnector(connector::Event::Resource {
                    project,
                    role: translate::resource_role(named.role),
                    name: tasks::Name { connector: number, path: named.resource.segments() },
                    hold,
                })));
            }
        }
        ConnectorRequest::Infrastructure(infrastructure::Request::Close { key, project, chat }) => {
            let token = domain.token();
            domain.hub_link.keep_close_reply(token, key);
            let mut grace = None;
            for setting in &domain.core.current.offered() {
                let number = match setting {
                    core::settings::Setting::Template(number) => *number,
                    core::settings::Setting::Charter(_)
                    | core::settings::Setting::Policy(_)
                    | core::settings::Setting::Rules => continue,
                };
                let template = domain.core.current.template(number).expect("offered template");
                if template.flags.conversation {
                    grace = template.grace;
                    break;
                }
            }
            work.push(Work::Core(core::Event::Act {
                reply_to: ReplyTo::new(token),
                connector: number,
                key,
                act: Box::new(jig_domain_people::Act::Close {
                    project,
                    task: chat,
                    grace,
                    reason: domain.environment_closed_reason.clone(),
                }),
                carried: None,
            }));
        }
        ConnectorRequest::Infrastructure(infrastructure::Request::Restarted { stage }) => {
            let step = match stage {
                infrastructure::RestartStage::ReadAfresh => core::start::Phase::ReadAfresh { connector: number },
                infrastructure::RestartStage::Outbox => core::start::Phase::SettleOutbox { connector: number },
            };
            work.push(Work::StartDone(step));
        }
        ConnectorRequest::Observability(observability::Request::Document { watcher, subject, document }) => {
            let document = match document {
                Ok(document) => Ok(crate::ClientDocument::Observability(Box::new(document))),
                Err(why) => Err(match why {
                    observability::DocumentMissing::Unknown => jig_domain_views::End::Gone,
                    observability::DocumentMissing::NotAllowed => jig_domain_views::End::NotAllowed,
                }),
            };
            crate::client_documents::document(domain, env, decision, number, watcher, subject, document, work);
        }
        ConnectorRequest::Observability(observability::Request::Save { record }) => {
            write(domain, decision, Write::Save(Record::Observability(record)));
        }
        ConnectorRequest::Observability(observability::Request::Erase { key }) => {
            write(domain, decision, Write::Erase(Key::Observability(key)));
        }
        ConnectorRequest::Observability(observability::Request::System(
            request @ observability::SystemRequest::Read { .. },
        )) => {
            now(domain, Output::Observability(request));
        }
        ConnectorRequest::Observability(observability::Request::System(
            request @ (observability::SystemRequest::Facts { .. }
            | observability::SystemRequest::Silence { .. }
            | observability::SystemRequest::FindSilence { .. }),
        )) => held(decision, Output::Observability(request)),
        ConnectorRequest::Observability(observability::Request::Answer { token, bytes }) => {
            if let Some(call) = domain.hub_link.take_read_call(token) {
                crate::assemble::read_answer(domain, call, false, bytes, work);
            } else {
                now(domain, Output::Read { owner: token, bytes });
            }
        }
        ConnectorRequest::Observability(observability::Request::Verdict { token, verdict }) => {
            let Some((owner, judge, state)) = domain.hub_link.take_judge(token) else { return };
            let (verdict, at) = translate::verdict(verdict, env.wall);
            work.push(Work::Core(core::Event::EffectConnector(connector::Event::Verdict {
                owner,
                judge,
                verdict,
                at,
                guarded: false,
                state,
            })));
        }
        ConnectorRequest::Observability(observability::Request::Described {
            token,
            effect,
            waits_for_dependencies,
        }) => {
            work.push(Work::Core(core::Event::EffectConnector(connector::Event::Described {
                owner: token,
                description: Box::new(translate::observability_description(
                    number,
                    effect,
                    waits_for_dependencies,
                    &env.limits.observability.vocabulary,
                )),
            })));
        }
        ConnectorRequest::Observability(observability::Request::Make { key }) => {
            child(decision, Released::Observability(observability::Event::Make { key }));
        }
        ConnectorRequest::Observability(observability::Request::Outcome { entry, key, outcome }) => {
            work.push(Work::Core(core::Event::EffectConnector(connector::Event::Outbox {
                connector: number,
                entry,
                task: key.task,
                outcome: translate::observability_outcome(outcome),
            })));
        }
        ConnectorRequest::Observability(observability::Request::News { alert, subscribers }) => {
            for subscriber in subscribers {
                let class = translate::news_class(subscriber.class);
                if let Some(event) = domain.core.connector_news(
                    subscriber.task,
                    subscriber.subscription,
                    class,
                    translate::alert_words(&alert),
                    env.wall,
                ) {
                    work.push(Work::Core(core::Event::Tasks(event)));
                }
            }
        }
        ConnectorRequest::Observability(observability::Request::Health { service, healthy, subscribers }) => {
            for subscriber in subscribers {
                let class = translate::news_class(subscriber.class);
                if let Some(event) = domain.core.connector_news(
                    subscriber.task,
                    subscriber.subscription,
                    class,
                    translate::health_words(&service, healthy),
                    env.wall,
                ) {
                    work.push(Work::Core(core::Event::Tasks(event)));
                }
            }
        }
        ConnectorRequest::Observability(observability::Request::WatchInterest { task, subscription, topic }) => {
            work.push(Work::Core(core::Event::Tasks(tasks::Event::SubscribeTopic {
                reply_to: ReplyTo::new(Token::new(subscription)),
                task,
                subscription: tasks::Subscription {
                    number: subscription,
                    kind: tasks::SubscriptionKind::Topic { connector: number, topic },
                },
            })));
        }
        ConnectorRequest::Observability(observability::Request::WatchDecision { task, step, decision: chosen }) => {
            let Some(Procedure { step: offered, connector, code }) = domain.hub_link.take_procedure(task) else {
                return;
            };
            assert_eq!(offered, step, "owner answers its offered step");
            let action = match chosen {
                observability::WatchDecision::Triage { delegate, alerts } => {
                    match translate::triage(&delegate, &alerts, &env.limits.policy.delegate) {
                        Some(batch) => core::ProcedureStep { batch, ..core::ProcedureStep::new(core::StepEnd::Wait) },
                        None => core::ProcedureStep::new(core::StepEnd::Hold(tasks::Hold::Procedure)),
                    }
                }
                observability::WatchDecision::Lead { delegate } => {
                    match translate::lead(&delegate, &env.limits.policy.delegate) {
                        Some(batch) => core::ProcedureStep { batch, ..core::ProcedureStep::new(core::StepEnd::Wait) },
                        None => core::ProcedureStep::new(core::StepEnd::Hold(tasks::Hold::Procedure)),
                    }
                }
                observability::WatchDecision::Wait => core::ProcedureStep::new(core::StepEnd::Wait),
                observability::WatchDecision::Hold => {
                    core::ProcedureStep::new(core::StepEnd::Hold(tasks::Hold::Procedure))
                }
            };
            work.push(Work::Core(core::Event::ProcedureStep { task, step, connector, code, decision: action }));
        }
        ConnectorRequest::Observability(observability::Request::WakeProcedure { task }) => {
            child(decision, Released::Core(core::Event::Tasks(tasks::Event::WakeProcedure { task })));
        }
        ConnectorRequest::Observability(observability::Request::AssociationRefused { task, step, reason }) => {
            held(decision, Output::IncidentAssociationRefused { task, step, reason });
        }
        ConnectorRequest::Observability(observability::Request::Triage { watch, delegate, alerts, .. }) => {
            let Some(Procedure { step, connector, code }) = domain.hub_link.take_procedure(watch) else { return };
            let action = match translate::triage(&delegate, &alerts, &env.limits.policy.delegate) {
                Some(batch) => core::ProcedureStep { batch, ..core::ProcedureStep::new(core::StepEnd::Wait) },
                None => core::ProcedureStep::new(core::StepEnd::Hold(tasks::Hold::Procedure)),
            };
            work.push(Work::Core(core::Event::ProcedureStep { task: watch, step, connector, code, decision: action }));
        }
        ConnectorRequest::Observability(observability::Request::Changed { service }) => {
            work.push(Work::Infrastructure(infrastructure::Event::ObservedServiceChanged {
                environment: service.environment,
                name: service.name,
            }));
        }
        ConnectorRequest::Observability(observability::Request::Restarted { stage }) => {
            let step = match stage {
                observability::RestartStage::ReadAfresh => core::start::Phase::ReadAfresh { connector: number },
                observability::RestartStage::Outbox => core::start::Phase::SettleOutbox { connector: number },
            };
            work.push(Work::StartDone(step));
        }
    }
}
#[expect(clippy::too_many_lines, reason = "one exhaustive dispatcher translates this child boundary")]
fn core_now(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision<Write, Output>,
    value: core::Now,
    work: &mut Queue<Work>,
) {
    match value {
        core::Now::Relayed { channel, run, attempt, call, answer } => crate::link::route(
            domain,
            env,
            decision,
            LinkEvent::Relayed { channel, run, attempt, call, answer },
            false,
            work,
        ),
        core::Now::Undelivered { message, .. } => {
            drop(domain.hub_link.take_payload(message.words));
        }
        core::Now::Activate { context } => {
            work.push(Work::Core(core::Event::Activate { context, ready: domain.ready() }));
        }
        core::Now::PrepareAgent { context, load_transcript } => {
            let transcript_waiter = if load_transcript { Some(Token::new(context.task)) } else { None };
            work.push(Work::Core(core::Event::PreparedAgent { context, transcript_waiter, busy: false }));
        }
        value @ core::Now::StartPreparation { .. } => {
            now(domain, Output::CoreLoad(value));
        }
        core::Now::BriefCorePlanned { task, sections, .. } => {
            work.push(Work::Core(core::Event::Brief(jig_domain_brief::GatherEvent::Plan {
                brief: Token::new(task),
                budget: env.limits.core.brief.brief_bytes,
                deadline: env.now.saturating_add(env.limits.policy.brief_timeout),
                sections: sections.into_boxed(),
            })));
        }
        core::Now::CompleteBrief { brief, order } => {
            domain.hub_link.keep_brief(brief.raw(), order);
            work.push(Work::Core(core::Event::BriefAssembled { task: brief.raw(), ready: true }));
        }
        core::Now::WorkspaceRequest { task, attempt, .. } => {
            work.push(Work::Core(core::Event::WorkspacePrepared {
                task,
                attempt,
                credentials: Box::new([]),
                writes: Some(Box::new([])),
            }));
        }
        core::Now::RunPrepared {
            live,
            task,
            attempt,
            conversation,
            charter,
            run,
            messages,
            transcript,
            grants,
            ..
        } => {
            let budget = run.budget;
            let brief = domain.hub_link.take_brief(task).expect("completed brief parts");
            let assignment = jig_domain_hub_link::Assignment {
                charter,
                live,
                task,
                attempt,
                conversation,
                run,
                brief,
                messages,
                transcript,
                grants,
                answered: domain.core.settled_calls(task, attempt),
            };
            if crate::assemble::assignment(domain, env, &assignment).is_some() {
                domain.hub_link.keep_assignment(assignment);
                work.push(Work::Core(core::Event::ClaimPrepared { task, attempt, budget, writes: Box::new([]) }));
            } else {
                work.push(Work::Core(core::Event::ClaimRefused { task }));
            }
        }
        core::Now::RunPreparationFailed { task } | core::Now::DropAssignment { task } => {
            drop(domain.hub_link.take_assignment(task));
            drop(domain.hub_link.take_brief(task));
        }
        core::Now::DropPayload { payload } => {
            crate::link::route(domain, env, decision, LinkEvent::DropPayload { payload }, false, work);
        }
        core::Now::TurnPayload { run, attempt, turn, body } => {
            crate::link::route(domain, env, decision, LinkEvent::TurnPayload { run, attempt, turn, body }, false, work);
        }
        core::Now::AnswerPayload { run, attempt, payload } => {
            crate::link::route(domain, env, decision, LinkEvent::AnswerPayload { run, attempt, payload }, false, work);
        }
        core::Now::AcceptedTurn { payload, task, attempt, turn, accepted } => crate::link::route(
            domain,
            env,
            decision,
            LinkEvent::AcceptedTurn { payload, task, attempt, turn, accepted },
            false,
            work,
        ),
        core::Now::RefusedPayload { request, problem } => {
            crate::link::route(domain, env, decision, LinkEvent::RefusedPayload { request, problem }, false, work);
        }
        load @ (core::Now::SeenConversation { .. }
        | core::Now::LoadEndedRead { .. }
        | core::Now::EndedProposal { .. }
        | core::Now::DecidedProposal { .. }
        | core::Now::EndedEscalation { .. }
        | core::Now::EndedPersonTask { .. }
        | core::Now::EndedTask { .. }
        | core::Now::PartyInputs { .. }) => {
            now(domain, Output::CoreLoad(load));
        }

        core::Now::RestoreRefused => {
            assert!(domain.core.start_failure().is_some(), "a refused row records its start failure");
        }
        core::Now::RelayBusy { channel, run, attempt, call } => {
            crate::link::route(
                domain,
                env,
                decision,
                LinkEvent::RelayBusy { channel, run, attempt, call },
                false,
                work,
            );
        }
        core::Now::DropCall { call } => {
            crate::link::route(domain, env, decision, LinkEvent::DropCall { call }, false, work);
        }
        other @ (core::Now::Call { .. }
        | core::Now::EffectAnswer { .. }
        | core::Now::SettingsAnswer { .. }
        | core::Now::NoteBusy { .. }
        | core::Now::SettledCallRefused { .. }
        | core::Now::CallBusy { .. }
        | core::Now::SignInRefused { .. }
        | core::Now::WatchRefused { .. }
        | core::Now::Account(_)
        | core::Now::RunText { .. }
        | core::Now::NotesIndexed { .. }
        | core::Now::NotesRecalled { .. }
        | core::Now::NotesRefused { .. }) => now(domain, Output::Now(other)),
    }
}

fn connector_holdings(
    env: &Env<Limits>,
    number: u16,
    executor: tasks::Executor,
    spec: &tasks::Spec,
) -> Option<Box<[tasks::Holding]>> {
    match jig_ops_domain_connectors::connector(number) {
        Some(Connector::Infrastructure) => translate::holdings(
            number,
            infrastructure::holdings(
                &env.limits.infrastructure.vocabulary,
                number,
                translate::infrastructure_executor(executor),
                &translate::infrastructure_spec(spec),
            ),
        ),
        Some(Connector::Observability) | None => Some(Box::new([])),
    }
}
