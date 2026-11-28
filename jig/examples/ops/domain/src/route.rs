//! Each variant fixes its destination. Replace connector arms when copying;
//! retain the journal marks and engine's permanent host link (domain/root.md, 5).
use crate::domain::{Work, child, held, now, write};
use crate::{Domain, Event, Key, Limits, Output, Record, Released, Store, Write};
use alloc::boxed::Box;
use jig_domain_contract as jig;
use jig_domain_core as core;
use jig_domain_core::engine::{self, In, Out, Peer, application, store};
use jig_domain_fleet as fleet;
use jig_domain_host as host;
use jig_domain_hub_link::Event as LinkEvent;
use jig_domain_tasks as tasks;
use jig_ops_domain_connectors::Connector;
use jig_ops_domain_infrastructure as infrastructure;
use jig_ops_domain_observability as observability;
use skein_lib::{Decision, Env, Queue, ReplyTo, Token};

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
            let mut out = Queue::uncapped();
            engine::step(&mut domain.engine, &env.with(env.limits.core), In::Core(event), &mut out);
            engine_out(domain, env, decision, out, work);
        }
        Work::Engine(input) => {
            let mut out = Queue::uncapped();
            engine::step(&mut domain.engine, &env.with(env.limits.core), input, &mut out);
            engine_out(domain, env, decision, out, work);
        }
        Work::Finish => {}
        Work::ResumeFleet => {
            let requests = core::resume_fleet(domain.engine.core_mut(), &env.with(env.limits.core));
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
                infrastructure_out(domain, decision, request, work);
            }
        }
        Work::Observability(event) => {
            let mut out = Queue::uncapped();
            observability::step(&mut domain.observability, &env.with(env.limits.observability), event, &mut out);
            for _ in 0..out.len() {
                let request = out.pop().expect("connector output count");
                observability_out(domain, decision, request, work);
            }
        }
        Work::Host(event) => {
            let mut out = Queue::uncapped();
            host::step(&mut domain.host, &env.with(env.limits.host), event, &mut out);
            crate::link::host_out(domain, env, decision, &mut out, work);
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
            let number = crate::client_documents::prepare(token, subject, work);
            work.push(Work::Core(core::Event::Watch {
                watcher,
                sign_in,
                subject: core::documents::Watched::Application { scope, connector: number, subject: token },
                window,
            }));
        }
        Event::ReadApplication { reply, sign_in, scope, subject } => {
            let token = domain.token();
            let number = crate::client_documents::prepare(token, subject, work);
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
        Event::Core(core::Event::PublicRead { reply, .. }) => {
            held(decision, Output::Core(core::Held::PublicRead { reply, answer: core::PublicAnswer::Busy }));
        }
        Event::Core(event) => work.push(Work::Core(event)),
        Event::Infrastructure(event) => work.push(Work::Infrastructure(event)),
        Event::Observability(event) => work.push(Work::Observability(event)),
        Event::Host(event) => work.push(Work::Host(event)),
        Event::Restart { pending } => {
            work.push(Work::Engine(In::Store(store::In::Opened { pending })));
        }
        Event::Store(store) => match store {
            Store::Engine(answer) => work.push(Work::Engine(In::Store(answer))),
            Store::Loaded(event) => work.push(Work::Core(event)),
            Store::Committed { .. } | Store::Failed { .. } => unreachable!("store terminals belong to the journal"),
        },
        Event::Released(released) => match released {
            Released::Core(event) => work.push(Work::Core(event)),
            Released::Fleet(event) => work.push(Work::Core(core::Event::Fleet(event))),
            Released::Host(event) => work.push(Work::Host(event)),
            Released::Infrastructure(event) => work.push(Work::Infrastructure(event)),
            Released::Observability(event) => work.push(Work::Observability(event)),
            Released::Resume(resume) => work.push(Work::Engine(In::Resume(resume))),
            Released::Connector(number, released) => match jig_ops_domain_connectors::connector(number.0) {
                Some(Connector::Infrastructure) => {
                    work.push(Work::Infrastructure(infrastructure::Event::Released(released)));
                }
                Some(Connector::Observability) => {
                    work.push(Work::Observability(observability::Event::Released(released)));
                }
                None => unreachable!("configured connector"),
            },
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
            let owner = domain.engine.core_mut().effect_owner();
            let origin = match proposal {
                Some(reason) => core::EffectOrigin::Propose { to, key, reason, as_holder: false },
                None => core::EffectOrigin::Call { to, key, deadline },
            };
            work.push(Work::Infrastructure(infrastructure::Event::StageEffect { owner, effect }));
            work.push(Work::Core(core::Event::EffectStart {
                owner,
                connector: jig_ops_domain_connectors::number(Connector::Infrastructure),
                origin,
            }));
        }
        Event::Timer(timer) => {
            let mut out = Queue::uncapped();
            engine::fire(&mut domain.engine, &env.with(env.limits.core), timer, &mut out);
            engine_out(domain, env, decision, out, work);
        }
        Event::TurnTimer => crate::link::fire(domain, env, decision, work),
        Event::InfrastructureTimer => {
            let mut out = Queue::uncapped();
            infrastructure::fire(&mut domain.infrastructure, &env.with(env.limits.infrastructure), &mut out);
            for _ in 0..out.len() {
                let request = out.pop().expect("connector output count");
                infrastructure_out(domain, decision, request, work);
            }
        }
        Event::ObservabilityTimer => {
            let mut out = Queue::uncapped();
            observability::fire(&mut domain.observability, &env.with(env.limits.observability), &mut out);
            for _ in 0..out.len() {
                let request = out.pop().expect("connector output count");
                observability_out(domain, decision, request, work);
            }
        }
        Event::Agent(report) => work.push(Work::Host(host::Event::Agent(report))),
        Event::AgentContent { owner, content } => {
            crate::link::agent_content(domain, env, decision, work, owner, content);
        }
    }
}

fn engine_out(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision<Write, Output>,
    mut out: Queue<Out>,
    work: &mut Queue<Work>,
) {
    for _ in 0..out.len() {
        match out.pop().expect("engine output count") {
            Out::Core(request) => core_request(domain, env, decision, request, work),
            Out::Write(write) => core_request(domain, env, decision, core::Request::Write(write), work),
            Out::Now(Peer::Application(report)) => match report {
                application::Out::Numbered { commits } => domain.restored_commits = Some(commits),
                application::Out::Start { phase } => {
                    domain.start = crate::domain::StartState::Reported(phase);
                    #[cfg(test)]
                    if let Some(phase) = phase {
                        domain.start_phases.push(phase).expect("the start's finite phase list");
                    }
                }
                application::Out::Refused { phase, failure } => {
                    domain.start_failure = Some((phase, failure));
                    domain.journal.stop();
                }
            },
            Out::Now(Peer::Store(_) | Peer::Connector(..) | Peer::Resume(_)) | Out::Held(Peer::Application(_)) => {
                unreachable!("engine port output mark")
            }
            Out::Held(Peer::Resume(resume)) => child(decision, Released::Resume(resume)),
            Out::Ask(number, ask) => match jig_ops_domain_connectors::connector(number.0) {
                Some(Connector::Infrastructure) => work.push(Work::Infrastructure(infrastructure::Event::Ask(ask))),
                Some(Connector::Observability) => work.push(Work::Observability(observability::Event::Ask(ask))),
                None => unreachable!("configured connector"),
            },
            Out::Held(Peer::Connector(connector, released)) => {
                child(decision, Released::Connector(connector, released));
            }
            Out::Held(Peer::Store(load)) => held(decision, Output::Store(load)),
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
    let mut out = Queue::with_capacity(requests.len());
    engine::append(requests, &mut out);
    engine_out(domain, env, decision, out, work);
}

pub(crate) fn core_request(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision<Write, Output>,
    request: core::Request,
    work: &mut Queue<Work>,
) {
    match request {
        core::Request::Write(core::Write::Wires) => write(domain, decision, Write::Wires),
        core::Request::Write(core::Write::DropHistory { task }) => {
            write(domain, decision, Write::DropHistory { task });
        }
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
    if let core::Held::PublicRead { .. } = value {
        return;
    }
    if let core::Held::PublicWait { read } = value {
        child(decision, Released::Core(core::Event::PublicDurable { read }));
        return;
    }
    if let core::Held::Store(_) = value {
        unreachable!("store loads cross the engine port (jig/docs/design/domain/engine/vocabulary.md, 2.4)")
    }

    if let core::Held::ApplicationDelivery { watcher, connector, subject } = value {
        let document = crate::client_documents::take(domain, connector, subject);
        held(decision, Output::ApplicationDocument { watcher, document });
        return;
    }
    if let core::Held::ApplicationWatchEnded { watcher, connector, subject, why } = value {
        crate::client_documents::forget(connector, subject, work);
        held(decision, Output::Core(core::Held::WatchEnded { watcher, why }));
        return;
    }
    if let core::Held::Applied { to, connector, token } = value {
        assert!(connector == jig_ops_domain_connectors::number(Connector::Infrastructure), "application request owner");
        let outcome = domain
            .infrastructure
            .application_outcome(token)
            .expect("kept people token has its retained connector outcome");
        held(decision, Output::Application { to, outcome });
        return;
    }
    if let core::Held::Assign { channel, run, attempt, .. } = value {
        crate::link::route(domain, env, decision, LinkEvent::Assign { channel, run, attempt }, true, work);
        return;
    }
    if let core::Held::SettledCall { to, key: _, call } = value {
        crate::link::route(domain, env, decision, LinkEvent::SettledCall { to, call }, false, work);
        return;
    }
    if let core::Held::Relayed { channel, run, attempt, call, answer } = value {
        crate::link::route(
            domain,
            env,
            decision,
            LinkEvent::Relayed { channel, run, attempt, call, answer },
            true,
            work,
        );
        return;
    }
    if let core::Held::Inbound { channel, run, attempt, message } = value {
        crate::link::route(domain, env, decision, LinkEvent::Inbound { channel, run, attempt, message }, true, work);
        return;
    }
    if let core::Held::Relay { task, attempt, word, .. } = value {
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
        return;
    }
    if let core::Held::TaskPartKept { key } = value {
        child(
            decision,
            Released::Fleet(fleet::Event::PartKept {
                run: Token::new(key.task),
                attempt: Token::new(key.attempt),
                part: key.number,
            }),
        );
        return;
    }
    if let core::Held::TaskTurnKept { task: run, attempt, turn } = value {
        child(
            decision,
            Released::Fleet(fleet::Event::TurnKept { run: Token::new(run), attempt: Token::new(attempt), turn }),
        );
        return;
    }
    if let core::Held::TurnBusy { channel, run, attempt, turn } = value
        && channel == Token::new(0)
    {
        crate::link::route(
            domain,
            env,
            decision,
            LinkEvent::TurnBusy { run, attempt, turn, retained: domain.host.has_turn(run, attempt, turn) },
            false,
            work,
        );
        return;
    }
    if let core::Held::StopRun { task, attempt } = value {
        child(decision, Released::Fleet(fleet::Event::Cancel { run: Token::new(task), attempt: Token::new(attempt) }));
        return;
    }
    if let core::Held::TaskTerminalAcknowledged { task, attempt } = value {
        child(
            decision,
            Released::Fleet(fleet::Event::Acknowledge { run: Token::new(task), attempt: Token::new(attempt) }),
        );
        return;
    }
    if let core::Held::CallAnswer { .. }
    | core::Held::Read { .. }
    | core::Held::Delivery { .. }
    | core::Held::WatchEnded { .. }
    | core::Held::PeopleReply { .. }
    | core::Held::CarriedUnused { .. }
    | core::Held::NeedArrived { .. }
    | core::Held::Result { .. }
    | core::Held::Drained
    | core::Held::Restarting { .. }
    | core::Held::Grant { .. }
    | core::Held::AcknowledgeTurn { .. }
    | core::Held::AcknowledgePart { .. }
    | core::Held::PartBusy { .. }
    | core::Held::Cancel { .. }
    | core::Held::Acknowledge { .. }
    | core::Held::ReadUsage { .. }
    | core::Held::ReadList { .. }
    | core::Held::Sources { .. }
    | core::Held::PauseWorker { .. }
    | core::Held::CheckSource { .. }
    | core::Held::SignOutSource { .. }
    | core::Held::SourceSignedOut { .. }
    | core::Held::Refuse { .. }
    | core::Held::TurnBusy { .. }
    | core::Held::NotesWritten { .. }
    | core::Held::NotesDeleted { .. }
    | core::Held::Acted { .. } = value
    {
        held(decision, Output::Core(value));
        return;
    }
    unreachable!("connector releases cross the engine port: {value:?}");
}

// The remaining core asks are outside the connectors' port.
fn core_ask(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision<Write, Output>,
    number: u16,
    ask: core::Ask,
    work: &mut Queue<Work>,
) {
    let infrastructure = jig_ops_domain_connectors::connector(number) == Some(Connector::Infrastructure);
    if let core::Ask::ClaimDone { task, attempt, kinds, version } = ask {
        if infrastructure {
            crate::link::route(domain, env, decision, LinkEvent::Start { task, attempt, kinds, version }, true, work);
        }
        return;
    }
    if let core::Ask::JobClaimed { task, attempt, kind, spec, deadline, result, grants, workspace } = ask {
        if infrastructure {
            crate::link::route(
                domain,
                env,
                decision,
                LinkEvent::StartJob {
                    job: jig_domain_hub_link::JobStart {
                        task,
                        attempt,
                        workstream: task,
                        kind,
                        spec,
                        deadline,
                        result,
                        grants,
                        workspace,
                    },
                },
                true,
                work,
            );
        }
        return;
    }
    // Ops supplies no connector brief sections.
    if let core::Ask::Gather { .. }
    | core::Ask::CutTo { .. }
    | core::Ask::Drop { .. }
    | core::Ask::Providers(_)
    | core::Ask::Workers(_)
    | core::Ask::Catalogue { .. } = ask
    {
        return;
    }
    unreachable!("connector asks cross the engine port: {ask:?}");
}

fn infrastructure_out(
    domain: &mut Domain,
    decision: &mut Decision<Write, Output>,
    request: infrastructure::Out,
    work: &mut Queue<Work>,
) {
    use infrastructure::Out as I;
    let number = jig_ops_domain_connectors::number(Connector::Infrastructure);
    match request {
        I::LoadRestore => held(decision, Output::ConnectorLoad(crate::Range::Infrastructure)),
        I::Up(up) => work.push(Work::Engine(In::Connector(jig::Connector(number), up))),
        I::Save { record } => write(domain, decision, Write::Save(Record::Infrastructure(record))),
        I::Erase { key } => write(domain, decision, Write::Erase(Key::Infrastructure(key))),
        I::System(request) => held(decision, Output::Infrastructure(request)),
        I::AfterCommit(event) => child(decision, Released::Infrastructure(*event)),
        I::Continue(event) => work.push(Work::Infrastructure(*event)),
        I::Changed { resource } => work.push(Work::Infrastructure(infrastructure::Event::ResourceChanged { resource })),
        I::EffectOwner { task } => {
            let owner = domain.engine.core_mut().effect_owner();
            work.push(Work::Infrastructure(infrastructure::Event::EffectOwner { task, owner }));
        }
        I::ClientRead { reply, answer } => {
            held(decision, Output::ApplicationRead { reply, answer: crate::client_documents::infrastructure(answer) });
        }
    }
}

fn observability_out(
    domain: &mut Domain,
    decision: &mut Decision<Write, Output>,
    request: observability::Out,
    work: &mut Queue<Work>,
) {
    use observability::Out as O;
    let number = jig_ops_domain_connectors::number(Connector::Observability);
    match request {
        O::LoadRestore => held(decision, Output::ConnectorLoad(crate::Range::Observability)),
        O::Up(up) => work.push(Work::Engine(In::Connector(jig::Connector(number), up))),
        O::Save { record } => write(domain, decision, Write::Save(Record::Observability(record))),
        O::Erase { key } => write(domain, decision, Write::Erase(Key::Observability(key))),
        O::AfterCommit(event) => child(decision, Released::Observability(*event)),
        O::Continue(event) => work.push(Work::Observability(*event)),
        O::ClientRead { reply, answer } => {
            held(decision, Output::ApplicationRead { reply, answer: crate::client_documents::observability(answer) });
        }
        O::System(request @ observability::SystemRequest::Read { .. }) => now(domain, Output::Observability(request)),
        O::System(
            request @ (observability::SystemRequest::Facts { .. }
            | observability::SystemRequest::Silence { .. }
            | observability::SystemRequest::FindSilence { .. }),
        ) => held(decision, Output::Observability(request)),
        O::Answer { token, bytes } => {
            if let Some(call) = domain.hub_link.take_read_call(token) {
                crate::assemble::read_answer(domain, call, false, bytes, work);
            } else {
                now(domain, Output::Read { owner: token, bytes });
            }
        }
        O::ReadRefused { token } => {
            if let Some(call) = domain.hub_link.take_read_call(token) {
                crate::assemble::read_answer(domain, call, true, Box::from(&b"unavailable"[..]), work);
            }
        }
        O::WatchInterest { task, subscription, topic } => {
            work.push(Work::Core(core::Event::Tasks(tasks::Event::SubscribeTopic {
                reply_to: ReplyTo::new(Token::new(subscription)),
                task,
                subscription: tasks::Subscription {
                    number: subscription,
                    kind: tasks::SubscriptionKind::Topic { connector: number, topic },
                },
            })));
        }
        O::AssociationRefused { task, step, reason } => {
            held(decision, Output::IncidentAssociationRefused { task, step, reason });
        }
        O::Changed { service } => work.push(Work::Infrastructure(infrastructure::Event::ObservedServiceChanged {
            environment: service.environment,
            name: service.name,
        })),
    }
}

#[expect(clippy::too_many_lines, reason = "one exhaustive dispatcher routes the core's immediate outputs")]
fn core_now(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision<Write, Output>,
    value: core::Now,
    work: &mut Queue<Work>,
) {
    match value {
        core::Now::PartBusy { key } => {
            work.push(Work::Core(core::Event::Fleet(fleet::Event::PartBusy {
                run: Token::new(key.task),
                attempt: Token::new(key.attempt),
                part: key.number,
            })));
        }
        core::Now::Ask { channel, run, attempt, number, bytes: _ } => {
            work.push(Work::Core(core::Event::Fleet(jig_domain_fleet::Event::Asked {
                channel,
                run,
                attempt,
                number,
                answer: jig_domain_link::AskAnswer::Refused,
            })));
        }
        core::Now::Link(value) => {
            crate::link::route(domain, env, decision, jig_domain_hub_link::now(value), false, work);
        }
        core::Now::Undelivered { message, .. } => drop(domain.hub_link.take_payload(message.words)),
        core::Now::Activate { context } => {
            work.push(Work::Core(core::Event::Activate { context, ready: domain.ready() }));
        }
        core::Now::BriefCorePlanned { task, sections, .. } => {
            let budget = env.limits.core.brief.brief_bytes;
            let deadline = env.now.saturating_add(env.limits.policy.brief_timeout);
            work.push(Work::Core(jig_domain_hub_link::brief_plan(task, sections, budget, deadline)));
        }
        core::Now::CompleteBrief { brief, order } => {
            domain.hub_link.keep_brief(brief.raw(), order);
            work.push(Work::Core(core::Event::BriefAssembled { task: brief.raw(), failure: None }));
        }
        core::Now::WorkspaceRequest { task, attempt, .. } => {
            work.push(Work::Core(core::Event::WorkspacePrepared {
                task,
                attempt,
                holds: Box::new([]),
                workspace: None,
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
                parts: Box::new([]),
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
                answered: domain.engine.core().settled_calls(task, attempt),
            };
            if crate::assemble::assignment(domain, env, &assignment).is_some() {
                domain.hub_link.keep_assignment(assignment);
                work.push(Work::Core(core::Event::ClaimPrepared { task, attempt, budget, writes: Box::new([]) }));
            } else {
                work.push(Work::Core(core::Event::ClaimRefused { task }));
            }
        }
        core::Now::BriefFailed { .. } => {}
        core::Now::RunPreparationFailed { task } | core::Now::DropAssignment { task } => {
            drop(domain.hub_link.take_assignment(task));
            drop(domain.hub_link.take_brief(task));
        }
        core::Now::RestoreRefused => {
            assert!(domain.engine.core().start_failure().is_some(), "a refused row records its start failure");
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
        | core::Now::TaskRead { .. }
        | core::Now::ProposalRead { .. }
        | core::Now::SettingsAnswer { .. }
        | core::Now::PersonAnswer { .. }
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
