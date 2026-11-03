//! Total boundary translations for the testing application's tools, effects
//! and restart script (`domain/root.md`, sections 3 and 11;
//! `domain/engine.md`, sections 6–7).

use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_connector as connector;
use skein_lib::{Decision, Env, List, Queue, Token};

mod brief;
mod workspace;

use crate::{Delivery, Domain, Key, Limits, Record, Work, Write, hold, write};

pub(super) fn route_core(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision<Write, Delivery>,
    requests: Queue<core::Request>,
    work: &mut Queue<Work>,
) {
    let mut requests = requests;
    for _ in 0..requests.len() {
        match requests.pop().expect("core request count") {
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
            core::Request::Held(held) => match *held {
                core::Held::ApplicationDelivery { .. } => {
                    unreachable!("document assembly owns this synchronous output")
                }
                core::Held::ApplicationWatchEnded { watcher, connector: number, subject, why } => {
                    work.push(Work::Connector { number, event: connector::Event::DocumentForget { subject } });
                    hold(decision, Delivery::Core(core::Held::WatchEnded { watcher, why }));
                }
                core::Held::Applied { to, connector: number, token } => {
                    let outcome = crate::numbered(domain, number)
                        .application_outcome(token)
                        .expect("the connector retains every kept people token");
                    hold(decision, Delivery::Application { to, number, outcome });
                }
                core::Held::Assign { channel, run, attempt, .. } => crate::link::held(
                    domain,
                    env.now,
                    decision,
                    jig_domain_hub_link::Event::Assign { channel, run, attempt },
                ),
                core::Held::SettledCall { to, call, .. } => {
                    crate::link::held(domain, env.now, decision, jig_domain_hub_link::Event::SettledCall { to, call });
                }
                core::Held::Relayed { channel, run, attempt, call, answer } => crate::link::held(
                    domain,
                    env.now,
                    decision,
                    jig_domain_hub_link::Event::Relayed { channel, run, attempt, call, answer },
                ),
                core::Held::MakeEffect { connector, entry } => {
                    hold(decision, Delivery::Core(core::Held::MakeEffect { connector, entry }));
                }
                core::Held::Inbound { channel, run, attempt, message } => crate::link::held(
                    domain,
                    env.now,
                    decision,
                    jig_domain_hub_link::Event::Inbound { channel, run, attempt, message },
                ),
                other @ (core::Held::ReadLoad { .. }
                | core::Held::Read { .. }
                | core::Held::Delivery { .. }
                | core::Held::WatchEnded { .. }
                | core::Held::Relay { .. }
                | core::Held::PeopleReply { .. }
                | core::Held::CarriedUnused { .. }
                | core::Held::CallAnswer { .. }
                | core::Held::Result { .. }
                | core::Held::Acknowledge { .. }
                | core::Held::TaskTerminalAcknowledged { .. }
                | core::Held::TaskTurnKept { .. }
                | core::Held::AcknowledgeTurn { .. }
                | core::Held::Drained
                | core::Held::Grant { .. }
                | core::Held::Cancel { .. }
                | core::Held::Sources { .. }
                | core::Held::CheckSource { .. }
                | core::Held::SignOutSource { .. }
                | core::Held::SourceSignedOut { .. }
                | core::Held::Refuse { .. }
                | core::Held::TurnBusy { .. }
                | core::Held::StopRun { .. }
                | core::Held::Load(..)
                | core::Held::NotesLoad { .. }
                | core::Held::NotesWritten { .. }
                | core::Held::Acted { .. }
                | core::Held::NotesDeleted { .. }) => hold(decision, crate::link::down(domain, Delivery::Core(other))),
            },
            core::Request::Now(now) => route_now(domain, env, decision, *now, work),
            core::Request::Ask { connector, ask } => route_ask(domain, env, decision, connector, ask, work),
            core::Request::ProjectMade { project } => hold(decision, Delivery::ProjectMade { project }),
        }
    }
}

#[expect(clippy::too_many_lines, reason = "one dispatcher translates the complete connector ask vocabulary")]
fn route_ask(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision<Write, Delivery>,
    number: u16,
    ask: core::Ask,
    work: &mut Queue<Work>,
) {
    match ask {
        core::Ask::ReleaseProject { project } => {
            if let Some(releases) = &mut domain.scripted_project_releases {
                releases.push((number, project));
            } else {
                work.push(Work::Connector { number, event: connector::Event::ReleaseProject { project } });
            }
        }
        core::Ask::StepMade { task, step, first, revisions } => {
            let _removed = domain.hub_link.take_procedure(task);
            work.push(Work::Connector { number, event: connector::Event::StepMade { task, step, first, revisions } });
        }
        core::Ask::StepRefused { task, step, why } => crate::steps::refused(number, task, step, why, work),

        core::Ask::DropDocument { subject } => {
            if number == 1 || number == 2 {
                work.push(Work::Connector { number, event: connector::Event::DocumentForget { subject } });
            }
        }
        core::Ask::Document { watcher, scope, subject, party, window } => work.push(Work::Connector {
            number,
            event: connector::Event::Document {
                watcher,
                scope: crate::client_documents::scope(scope),
                subject,
                party,
                window: crate::client_documents::window(window),
            },
        }),
        core::Ask::Read { reply, scope, read, party, window } => crate::client_documents::read(
            domain,
            env,
            decision,
            number,
            reply,
            crate::client_documents::scope(scope),
            read,
            party,
            window,
            work,
        ),
        core::Ask::Application { request, token, owner, person, scope, role, retention } => {
            work.push(Work::Connector {
                number,
                event: connector::Event::Application {
                    request,
                    token,
                    owner,
                    person,
                    scope: crate::client_documents::scope(scope),
                    role: crate::client_documents::role_number(role),
                    retention,
                },
            });
        }
        core::Ask::DropApplication { owner } => {
            work.push(Work::Connector { number, event: connector::Event::ApplicationDrop { owner } });
        }
        core::Ask::ApplicationPartAnswered { request, outcome } => work.push(Work::Connector {
            number,
            event: connector::Event::ApplicationCore { request, answer: application_answer(outcome) },
        }),
        core::Ask::Effect(ask) => route_effect_ask(domain, number, ask, work),
        core::Ask::TaskHoldings { request, project, spec, .. } => {
            let holdings = specification_holdings(domain, env, number, project, &spec, work);
            work.push(Work::Core(core::Event::Holdings { refused: None, request, connector: number, holdings }));
        }
        core::Ask::DelegateHoldings { request, members, .. } => {
            let mut holdings: List<Box<[tasks::Holding]>> =
                List::with_capacity(u32::try_from(members.len()).expect("bounded batch"));
            let mut valid = true;
            for member in members {
                match specification_holdings(domain, env, number, member.project, &member.spec, work) {
                    Some(named) => holdings.push(named).expect("batch room"),
                    None => valid = false,
                }
            }
            work.push(Work::Core(core::Event::DelegateHoldings {
                refused: None,
                request,
                connector: number,
                holdings: if valid { Some(holdings.into_boxed()) } else { None },
            }));
        }
        core::Ask::ProcedureHoldings { task, step, members } => {
            let mut holdings: List<Box<[tasks::Holding]>> =
                List::with_capacity(u32::try_from(members.len()).expect("bounded batch"));
            let mut valid = true;
            for member in members {
                match specification_holdings(domain, env, number, member.project, &member.spec, work) {
                    Some(named) => holdings.push(named).expect("batch room"),
                    None => valid = false,
                }
            }
            work.push(Work::Core(core::Event::ProcedureHoldings {
                refused: None,
                task,
                step,
                connector: number,
                holdings: if valid { Some(holdings.into_boxed()) } else { None },
            }));
        }
        core::Ask::StartProcedure { context, step } => {
            let code = match context.executor {
                tasks::Executor::Job { .. } => return,
                tasks::Executor::Procedure { code, .. } => u16::try_from(code).expect("test procedure code"),
                tasks::Executor::Agent { .. } | tasks::Executor::Person(_) => {
                    unreachable!("procedure context has a procedure executor")
                }
            };
            domain.hub_link.keep_procedure(
                context.task,
                jig_domain_hub_link::Procedure { step, connector: number, code: u32::from(code) },
            );
            hold(decision, Delivery::Procedure { task: context.task, step, connector: number, code, context });
        }
        core::Ask::ClaimDone { task, attempt, kinds, version } => {
            if number == 1 {
                crate::link::held(
                    domain,
                    env.now,
                    decision,
                    jig_domain_hub_link::Event::Start { task, attempt, kinds, version },
                );
            }
        }
        core::Ask::JobClaimed { task, attempt, kind, spec, deadline, result, grants, workspace } => {
            if number == 1 {
                drop(domain.workspace_writes.remove(&task));
                crate::link::held(
                    domain,
                    env.now,
                    decision,
                    jig_domain_hub_link::Event::StartJob {
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
                );
            }
        }
        core::Ask::Close { task, .. } => {
            work.push(Work::Connector { number, event: connector::Event::Close { task } });
        }
        core::Ask::Release { task, .. } => {
            work.push(Work::Connector { number, event: connector::Event::Unnamed { task } });
            if number == 1 {
                work.push(Work::Core(core::Event::Tasks(tasks::Event::Settled { task })));
            }
        }
        core::Ask::Adopt { request: _, project, adoption } => {
            let segments = adoption.resource.path;
            let path = path(segments);
            let role = match adoption.role {
                people::ResourceRole::Owned => connector::ResourceRole::Owned,
                people::ResourceRole::Fork => connector::ResourceRole::Participating,
                people::ResourceRole::Context => connector::ResourceRole::Context,
            };
            work.push(Work::Connector { number, event: connector::Event::Adopt { project, resource: path, role } });
        }
        core::Ask::Gather { section, budget } => {
            work.push(Work::Connector {
                number,
                event: connector::Event::Gather { token: section, task: section.raw(), budget },
            });
        }
        core::Ask::CutTo { section, size } => {
            work.push(Work::Connector { number, event: connector::Event::Cut { token: section, size } });
        }
        core::Ask::Drop { section } => {
            work.push(Work::Connector { number, event: connector::Event::HandOver { token: section } });
        }
        core::Ask::Lost { task, attempt } => {
            for resource in domain.core.tasks.lost_writes(task, attempt) {
                if resource.connector == number {
                    let call = connector::SystemRequest::ReadFact {
                        resource: path(resource.path.clone()),
                        observed: env.wall,
                    };
                    hold(decision, Delivery::LostRead { connector: number, task, attempt, resource, call });
                }
            }
        }
        core::Ask::ProjectGoal { feed } => hold(decision, Delivery::Projection { connector: number, feed }),
        core::Ask::EndTopic { task, .. } => crate::topics::sync(domain, number, task, work),
        core::Ask::SubscriptionDone { request, key, subscription } => {
            crate::topics::subscribed(domain, number, request, key, subscription, work);
        }
        core::Ask::UnsubscriptionDone { request, key } => {
            crate::topics::unsubscribed(domain, number, request, key, work);
        }
        core::Ask::DropSubscription { .. }
        | core::Ask::ForgetProjection { .. }
        | core::Ask::Hold { .. }
        | core::Ask::RepairRefused { .. }
        | core::Ask::DelegateRefused { .. } => {}
    }
}

/// Translate the connector's one-based resource IDs. Kinds one and two are
/// respectively exclusive and pooled in this testing application's charter.
fn specification_holdings(
    domain: &mut Domain,
    env: &Env<Limits>,
    number: u16,
    project: u32,
    spec: &tasks::Spec,
    work: &mut Queue<Work>,
) -> Option<Box<[tasks::Holding]>> {
    let mut holdings = List::with_capacity(env.limits.core.tasks.holdings);
    for parameter in &spec.parameters {
        match parameter {
            tasks::Parameter::Resource { connector: owner, resource, .. } => {
                if *owner == number {
                    let described = crate::numbered(domain, number).specification_resource(*resource)?.clone();
                    let role = crate::numbered(domain, number).resource_role(project, &described.path);
                    let role = match role {
                        Some(connector::ResourceRole::Owned) => core::effects::connector::ResourceRole::Owned,
                        Some(connector::ResourceRole::Participating) => {
                            core::effects::connector::ResourceRole::Participant
                        }
                        Some(connector::ResourceRole::Context) => core::effects::connector::ResourceRole::Context,
                        Some(connector::ResourceRole::Unavailable) | None => {
                            core::effects::connector::ResourceRole::Unavailable
                        }
                    };
                    let hold = report_hold(domain, number, &described.path, described.hold);
                    let name = tasks::Name { connector: number, path: name(described.path.clone()).segments };
                    work.push(Work::Core(core::Event::EffectConnector(core::effects::connector::Event::Resource {
                        project,
                        name: name.clone(),
                        role,
                        hold,
                    })));
                    let holding = match described.hold {
                        connector::Hold::Exclusive { .. } => Some(tasks::Holding::Write { resource: name, kind: 1 }),
                        connector::Hold::Pooled { .. } => Some(tasks::Holding::Slot { pool: name, kind: 2 }),
                        connector::Hold::None => None,
                    };
                    if let Some(holding) = holding {
                        holdings.push(holding).ok()?;
                    }
                }
            }
            tasks::Parameter::Number { .. } | tasks::Parameter::Bytes { .. } => {}
        }
    }
    Some(holdings.into_boxed())
}

fn path(segments: Box<[Box<[u8]>]>) -> connector::Path {
    let capacity = u32::try_from(segments.len()).expect("bounded path segments");
    let mut path = List::with_capacity(capacity);
    for segment in segments {
        path.push(segment).expect("path room");
    }
    connector::Path::new(path, u32::MAX).expect("admitted nonempty path")
}

#[expect(clippy::too_many_lines, reason = "one exhaustive root continuation boundary")]
fn route_now(
    domain: &mut Domain,
    env: &Env<Limits>,
    _decision: &mut Decision<Write, Delivery>,
    now: core::Now,
    work: &mut Queue<Work>,
) {
    match now {
        core::Now::RelayBusy { channel, run, attempt, call } => {
            domain.hub_link.forget_call(&call);
            let delivery = crate::link::down(
                domain,
                Delivery::CallBusy { channel, task: run.raw(), attempt: attempt.raw(), name: call },
            );
            assert!(domain.journal.now(delivery).is_ok(), "reserved busy answer room");
        }
        core::Now::CallBusy { to } | core::Now::NoteBusy { to, .. } => crate::stopping::call(domain, to, work),
        core::Now::Relayed { channel, run, attempt, call, answer } => crate::link::route(
            domain,
            env.now,
            jig_domain_hub_link::Event::Relayed { channel, run, attempt, call, answer },
            work,
        ),
        core::Now::DropCall { call } => {
            domain.hub_link.forget_call(&call.name);
            domain.now.push(crate::Request::Now(core::Now::DropCall { call }));
        }
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
            domain.now.push(crate::Request::Now(value));
        }
        core::Now::BriefCorePlanned { task, sections, .. } => brief::plan(env, task, sections, work),
        core::Now::CompleteBrief { brief: token, order } => brief::complete(domain, token, order, work),
        core::Now::WorkspaceRequest { task, attempt, context } => {
            let mut names = List::with_capacity(env.limits.core.tasks.holdings);
            let mut checks = List::with_capacity(env.limits.core.authority.writes);
            for number in [1, 2] {
                let Some(holdings) = specification_holdings(domain, env, number, context.project, &context.spec, work)
                else {
                    work.push(Work::Core(core::Event::WorkspacePrepared {
                        task,
                        attempt,
                        holds: Box::new([]),
                        workspace: None,
                        credentials: Box::new([]),
                        writes: None,
                    }));
                    return;
                };
                for holding in holdings {
                    match holding {
                        tasks::Holding::Write { resource, .. } => {
                            checks
                                .push(authority::Write {
                                    effect: authority::Effect {
                                        connector: resource.connector,
                                        kind: 5,
                                        name: authority::Name { segments: resource.path.clone() },
                                        state: [0; 32],
                                        price: None,
                                        access: authority::EffectAccess::Owned,
                                        additional: Box::new([]),
                                        guards: Box::new([]),
                                    },
                                    held: authority::Writer::Task,
                                })
                                .expect("workspace write check room");
                            names.push(resource).expect("admitted holding count");
                        }
                        tasks::Holding::Slot { .. } => {}
                    }
                }
            }
            assert!(domain.workspace_writes.insert(task, names.into_boxed()).is_ok(), "workspace per task");
            let prepared = workspace::credentials(domain, &env.limits, context.project, &context.spec);
            let (credentials, writes) = match prepared {
                Some(credentials) => (credentials, Some(checks.into_boxed())),
                None => (Box::<[u32]>::from([]), None),
            };
            let holds = match context.executor {
                tasks::Executor::Job { .. } => domain.workspace_writes.get(&task).expect("prepared writes").clone(),
                tasks::Executor::Agent { .. } | tasks::Executor::Procedure { .. } | tasks::Executor::Person(_) => {
                    Box::new([])
                }
            };
            work.push(Work::Core(core::Event::WorkspacePrepared {
                task,
                attempt,
                holds,
                credentials,
                writes,
                workspace: None,
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
            let answered = domain.core.settled_calls(task, attempt);
            let writes = domain.workspace_writes.remove(&task).expect("assembled workspace writes");
            crate::link::route(
                domain,
                env.now,
                jig_domain_hub_link::Event::Prepared {
                    assignment: crate::Assignment {
                        live,
                        task,
                        attempt,
                        conversation,
                        brief: Box::new([]),
                        charter,
                        run,
                        messages,
                        transcript,
                        answered,
                        grants,
                    },
                    writes,
                },
                work,
            );
        }
        core::Now::RunPreparationFailed { task } | core::Now::DropAssignment { task } => {
            drop(domain.hub_link.take_assignment(task));
            drop(domain.hub_link.take_brief(task));
            drop(domain.workspace_writes.remove(&task));
        }
        core::Now::DropPayload { payload } => {
            drop(domain.hub_link.take_payload(payload));
        }
        core::Now::TurnPayload { run, attempt, turn, body } => crate::link::route(
            domain,
            env.now,
            jig_domain_hub_link::Event::TurnPayload { run, attempt, turn, body },
            work,
        ),
        core::Now::AnswerPayload { run, attempt, payload } => crate::link::answer(domain, run, attempt, payload, work),
        core::Now::AcceptedTurn { payload, task, attempt, turn, accepted } => crate::link::route(
            domain,
            env.now,
            jig_domain_hub_link::Event::AcceptedTurn { payload, task, attempt, turn, accepted },
            work,
        ),
        core::Now::RefusedPayload { request, problem } => {
            crate::link::route(domain, env.now, jig_domain_hub_link::Event::RefusedPayload { request, problem }, work);
        }
        core::Now::RestoreRefused => {
            assert!(domain.start_stopped(), "a refused restore has stopped the core");
        }

        other @ (core::Now::SeenConversation { .. }
        | core::Now::LoadEndedRead { .. }
        | core::Now::EndedPersonTask { .. }
        | core::Now::EndedTask { .. }
        | core::Now::PartyInputs { .. }
        | core::Now::EndedProposal { .. }
        | core::Now::DecidedProposal { .. }
        | core::Now::EndedEscalation { .. }
        | core::Now::SettledCallRefused { .. }
        | core::Now::Call { .. }
        | core::Now::EffectAnswer { .. }
        | core::Now::SettingsAnswer { .. }
        | core::Now::SignInRefused { .. }
        | core::Now::WatchRefused { .. }
        | core::Now::Account(_)
        | core::Now::RunText { .. }
        | core::Now::NotesIndexed { .. }
        | core::Now::NotesRecalled { .. }
        | core::Now::NotesRefused { .. }) => domain.now.push(crate::Request::Now(other)),
    }
}

#[expect(clippy::too_many_lines, reason = "one exhaustive connector boundary dispatcher")]
pub(super) fn route_connector(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision<Write, Delivery>,
    number: u16,
    requests: &mut Queue<connector::Request>,
    work: &mut Queue<Work>,
) {
    for _ in 0..requests.len() {
        match requests.pop().expect("connector request count") {
            connector::Request::SettingsJudged { token, judged } => {
                let owner = domain.settings_judges.remove(&token).expect("settings judge has a core owner");
                let mut answers = List::with_capacity(u32::try_from(judged.len()).expect("bounded judgments"));
                for answer in judged {
                    answers
                        .push(match answer {
                            connector::SettingJudged::Accept { size, charters } => {
                                core::effects::connector::Judged::Accept { size, charters }
                            }
                            connector::SettingJudged::Same => core::effects::connector::Judged::Same,
                            connector::SettingJudged::Refuse { why } => {
                                core::effects::connector::Judged::Refuse { why }
                            }
                        })
                        .expect("one answer per judgment");
                }
                work.push(Work::Core(core::Event::EffectConnector(core::effects::connector::Event::SettingsJudged {
                    owner,
                    judged: answers.into_boxed(),
                })));
            }
            connector::Request::SettingsRestored { token, judged } => {
                let owner = domain.settings_judges.remove(&token).expect("settings restore has a core owner");
                let mut answers = List::with_capacity(u32::try_from(judged.len()).expect("bounded judgments"));
                for answer in judged {
                    answers
                        .push(match answer {
                            connector::SettingJudged::Accept { size, charters } => {
                                core::effects::connector::Judged::Accept { size, charters }
                            }
                            connector::SettingJudged::Same => core::effects::connector::Judged::Same,
                            connector::SettingJudged::Refuse { why } => {
                                core::effects::connector::Judged::Refuse { why }
                            }
                        })
                        .expect("one answer per judgment");
                }
                work.push(Work::Core(core::Event::EffectConnector(
                    core::effects::connector::Event::SettingsRestored { owner, judged: answers.into_boxed() },
                )));
            }
            connector::Request::SettingsDefaults { defaults } => {
                let mut states = List::with_capacity(u32::try_from(defaults.len()).expect("bounded defaults"));
                for default in defaults {
                    states
                        .push(core::effects::connector::DefaultState {
                            kind: default.kind,
                            scope: default.scope,
                            number: default.number,
                            size: default.size,
                            charters: default.charters,
                            differs: default.differs,
                        })
                        .expect("one state per default");
                }
                work.push(Work::Core(core::Event::EffectConnector(
                    core::effects::connector::Event::SettingsDefaults {
                        connector: number,
                        defaults: states.into_boxed(),
                    },
                )));
            }
            connector::Request::SettingValue { .. } => unreachable!("a setting read returns to its synchronous caller"),
            connector::Request::RestoreRefused { kind, scope, number } => {
                crate::refuse_settings_restore(kind, scope, number);
            }
            connector::Request::ProjectReleased { project } => {
                work.push(Work::Core(core::Event::ProjectReleased { connector: number, project }));
            }
            connector::Request::Document { watcher, subject, document } => {
                crate::client_documents::document(domain, env, decision, number, watcher, subject, document, work);
            }
            connector::Request::Topic { token, resources } => {
                crate::topics::described(domain, number, token, Some(resources), work);
            }
            connector::Request::TopicRefused { token } => crate::topics::described(domain, number, token, None, work),
            connector::Request::News { topic, subscribers } => {
                crate::topics::news(domain, number, topic, subscribers, work);
            }
            connector::Request::ReadAnswered { .. } => unreachable!("client read completes in its synchronous route"),
            connector::Request::Marked { subjects } => {
                work.push(Work::Core(core::Event::Marked { connector: number, subjects }));
            }
            connector::Request::ApplicationDone { request, token } => {
                work.push(Work::Core(core::Event::ApplicationFinished { request, outcome: Ok(token) }));
            }
            connector::Request::ApplicationAgain { request, why } => {
                let why = match why {
                    connector::application_core::Again::Busy => people::Again::Busy,
                    connector::application_core::Again::NotReady => people::Again::NotReady,
                };
                work.push(Work::Core(core::Event::ApplicationFinished { request, outcome: Err(why) }));
            }
            connector::Request::ApplicationRelease { request, project, release } => {
                work.push(Work::Core(core::Event::ApplicationPart {
                    request,
                    ask: Box::new(people::Ask::Release {
                        project,
                        task: release.task,
                        revision: release.revision,
                        again: release.again,
                    }),
                }));
            }
            connector::Request::Closed { task } => {
                work.push(Work::Core(core::Event::EffectConnector(core::effects::connector::Event::Closed {
                    task,
                    connector: number,
                })));
            }
            connector::Request::Drift { pool: resource, tasks }
            | connector::Request::DriftResource { tasks, resource } => {
                let resource = name(resource);
                for task in tasks {
                    work.push(Work::Core(core::Event::EffectConnector(core::effects::connector::Event::Drift {
                        task,
                        resource: tasks::Name { connector: number, path: resource.segments.clone() },
                    })));
                }
            }
            connector::Request::RestartDone => work.push(Work::ConnectorRestartDone { number }),
            connector::Request::Save { record } => {
                match &record {
                    connector::Record::Outbox(row) => {
                        assert!(crate::remember_outbox(domain, number, row), "bounded outbox owner routes");
                    }
                    connector::Record::SettingDefault { .. }
                    | connector::Record::SettingChange { .. }
                    | connector::Record::SettingReplaced { .. }
                    | connector::Record::SettingProposal { .. }
                    | connector::Record::ApplicationObject { .. }
                    | connector::Record::ApplicationAnswer { .. }
                    | connector::Record::Proposal { .. }
                    | connector::Record::Task { .. }
                    | connector::Record::Adoption { .. }
                    | connector::Record::Subscription { .. }
                    | connector::Record::Pool { .. }
                    | connector::Record::Procedure(_)
                    | connector::Record::ProcedureDelegateOutcome { .. }
                    | connector::Record::Result { .. }
                    | connector::Record::Made { .. } => {}
                }
                write(domain, decision, Write::Save(Record::Connector { number, record }));
            }
            connector::Request::Erase { key } => {
                write(domain, decision, Write::Erase(Key::Connector { number, key }));
            }
            connector::Request::Step { task, decision: step } => {
                let procedure = *domain.hub_link.procedure(task).expect("procedure chose a step");
                let numbered_step = procedure.step;
                let Some(action) = crate::steps::translate(domain, task, numbered_step, number, step, work) else {
                    continue;
                };
                work.push(Work::Core(core::Event::ProcedureStep {
                    task,
                    step: numbered_step,
                    connector: number,
                    code: procedure.code,
                    decision: action,
                }));
            }
            connector::Request::System(call) => hold(decision, Delivery::System { connector: number, call }),
            connector::Request::Described { token, description, waits_for_dependencies } => {
                work.push(Work::Core(core::Event::EffectConnector(core::effects::connector::Event::Described {
                    owner: token,
                    description: Box::new(describe(number, description, waits_for_dependencies)),
                })));
            }
            connector::Request::EffectBusy { token } => {
                work.push(Work::Core(core::Event::EffectConnector(core::effects::connector::Event::DescribeBusy {
                    owner: token,
                })));
            }
            connector::Request::EffectRefused { token } => {
                work.push(Work::Core(core::Event::EffectConnector(core::effects::connector::Event::DescribeRefused {
                    owner: token,
                })));
            }
            connector::Request::Verdict { token, verdict } => {
                let Some((owner, judge, asked_state)) = domain.hub_link.take_judge(token) else { continue };
                let (verdict, at, guarded, state) = match verdict {
                    connector::Verdict::Met { state, observed, guarded } => {
                        let pin = state_pin(state);
                        (authority::Verdict::Met, observed, guarded, pin)
                    }
                    connector::Verdict::Wait => (authority::Verdict::Wait, env.wall, false, asked_state),
                    connector::Verdict::Refuse { .. } => (authority::Verdict::Refuse, env.wall, false, asked_state),
                };
                work.push(Work::Core(core::Event::EffectConnector(core::effects::connector::Event::Verdict {
                    owner,
                    judge,
                    verdict,
                    at,
                    guarded,
                    state,
                })));
            }
            connector::Request::Outcome { entry, outcome } => {
                let owner = match domain.hub_link.outbox_owner(entry) {
                    Some(owner) => *owner,
                    None => continue,
                };
                let made = match outcome {
                    connector::Outcome::Made { .. } => true,
                    connector::Outcome::Failed
                    | connector::Outcome::Uncertain
                    | connector::Outcome::Skipped
                    | connector::Outcome::Withdrawn
                    | connector::Outcome::Held => false,
                };
                let settled = match outcome {
                    connector::Outcome::Made { .. }
                    | connector::Outcome::Failed
                    | connector::Outcome::Skipped
                    | connector::Outcome::Withdrawn => true,
                    connector::Outcome::Uncertain | connector::Outcome::Held => false,
                };
                match domain.hub_link.outbox_procedure(entry) {
                    Some(number) if settled => work.push(Work::Connector {
                        number: *number,
                        event: connector::Event::EffectDecision { task: owner.task, made },
                    }),
                    Some(_) | None => {}
                }
                if settled {
                    let _removed = domain.hub_link.take_outbox_owner(entry);
                    let _removed = domain.hub_link.take_outbox_procedure(entry);
                }
                let outcome = match outcome {
                    connector::Outcome::Made { .. } => core::effects::connector::OutboxOutcome::Made,
                    connector::Outcome::Failed => core::effects::connector::OutboxOutcome::Failed,
                    connector::Outcome::Uncertain => core::effects::connector::OutboxOutcome::Uncertain,
                    connector::Outcome::Held => core::effects::connector::OutboxOutcome::Held { entry },
                    connector::Outcome::Withdrawn => core::effects::connector::OutboxOutcome::Withdrawn,
                    connector::Outcome::Skipped => core::effects::connector::OutboxOutcome::NotMade,
                };
                let event = if owner.projection {
                    core::effects::connector::Event::ProjectionOutbox { connector: number, entry, outcome }
                } else {
                    core::effects::connector::Event::Outbox { connector: number, entry, task: owner.task, outcome }
                };
                work.push(Work::Core(core::Event::EffectConnector(event)));
            }
            connector::Request::Slots { pool, slots } => {
                let name = tasks::Name { connector: number, path: name(pool).segments };
                work.push(Work::Core(core::Event::EffectConnector(core::effects::connector::Event::PoolSlots {
                    name,
                    slots,
                })));
            }
            connector::Request::Named { task: _, project, resources } => {
                resource_reports(domain, number, project, resources, work);
            }
            connector::Request::Resource { project, resource } => {
                resource_reports(domain, number, project, Box::new([resource]), work);
            }
            connector::Request::Make { entry: _ }
            | connector::Request::Adopted { project: _, resource: _, result: _ }
            | connector::Request::Unknown { task: _, resource: _ }
            | connector::Request::Refused { task: _ }
            | connector::Request::Changed { resource: _ }
            | connector::Request::Answer { token: _, bytes: _ }
            | connector::Request::Ready { token: _, size: _ }
            | connector::Request::Section { token: _, bytes: _ }
            | connector::Request::Workspace { token: _, items: _ } => {}
        }
    }
}

/// Translate the core's choice without choosing a judge or admitting a write.
#[expect(clippy::too_many_lines, reason = "one exhaustive effect handoff dispatcher")]
fn route_effect_ask(domain: &mut Domain, number: u16, ask: core::effects::connector::Ask, work: &mut Queue<Work>) {
    let event = match ask {
        core::effects::connector::Ask::ForgetReplaced { before } => connector::Event::ForgetReplaced { before },
        core::effects::connector::Ask::RestoreSettings { owner, proposal, items } => {
            let token = domain.hub_link.fresh();
            domain.settings_judges.insert(token, owner).expect("settings restore route fits");
            let mut translated = List::with_capacity(u32::try_from(items.len()).expect("bounded restore items"));
            for item in items {
                let edit = match item.edit {
                    core::settings::application::RestoredEdit::Set { revision } => {
                        connector::RestoredEdit::Set { revision }
                    }
                    core::settings::application::RestoredEdit::Remove => connector::RestoredEdit::Remove,
                    core::settings::application::RestoredEdit::Reset => connector::RestoredEdit::Reset,
                };
                translated
                    .push(connector::RestoredItem { kind: item.kind, scope: item.scope, number: item.number, edit })
                    .expect("one translated restore item");
            }
            connector::Event::RestoreSettings { token, proposal, items: translated.into_boxed() }
        }
        core::effects::connector::Ask::ResolveUncertain { entry, again } => {
            connector::Event::Resolve { entry, repeat: again }
        }
        core::effects::connector::Ask::Describe { owner } => connector::Event::Describe {
            token: owner,
            effect: domain.effects.remove(&owner).expect("connector-owned decoded effect"),
        },
        core::effects::connector::Ask::DescribeProposal { owner, proposal } => {
            connector::Event::DescribeProposal { token: owner, number: proposal }
        }
        core::effects::connector::Ask::KeepProposal { owner, proposal, task, project } => {
            connector::Event::KeepProposal { token: owner, number: proposal, task, project }
        }
        core::effects::connector::Ask::DropProposal { proposal } => connector::Event::DropProposal { number: proposal },
        core::effects::connector::Ask::Keep { owner, entry, task, key } => {
            let _removed = domain.hub_link.take_effect_procedure(owner);
            let origin = match key.origin {
                core::EffectPurpose::Call { attempt, completion, position } => {
                    connector::Purpose::Call { attempt, completion, position }
                }
                core::EffectPurpose::Procedure { .. } => connector::Purpose::Procedure,
                core::EffectPurpose::Projection { .. } => connector::Purpose::Projection,
            };
            connector::Event::Keep {
                token: owner,
                entry,
                task,
                key: connector::Key { deployment: key.deployment, task: key.task, purpose: key.purpose, origin },
            }
        }
        core::effects::connector::Ask::Drop { owner, .. } => {
            if let Some(task) = domain.hub_link.take_effect_procedure(owner) {
                work.push(Work::Connector { number, event: connector::Event::EffectDecision { task, made: false } });
            }
            drop(domain.effects.remove(&owner));
            connector::Event::Drop { token: owner }
        }
        core::effects::connector::Ask::Judge { owner, judge, state, resources } => {
            let token = domain.hub_link.fresh();
            domain.hub_link.keep_judge(token, (owner, judge, state));
            let mut names = List::with_capacity(u32::try_from(resources.len()).expect("bounded resource names"));
            for name in resources {
                names.push(path(name.segments)).expect("one path per name");
            }
            let bytes = [state[24], state[25], state[26], state[27], state[28], state[29], state[30], state[31]];
            connector::Event::Judge {
                token,
                requirement: judge.requirement,
                resources: names.into_boxed(),
                state: u64::from_be_bytes(bytes),
            }
        }
        core::effects::connector::Ask::Unname { .. } => return,
        core::effects::connector::Ask::Names { .. }
        | core::effects::connector::Ask::Hold { .. }
        | core::effects::connector::Ask::ReleaseHold { .. }
        | core::effects::connector::Ask::Writer { .. }
        | core::effects::connector::Ask::Make { .. }
        | core::effects::connector::Ask::ProcedureMade { .. }
        | core::effects::connector::Ask::ProcedureActivate { .. }
        | core::effects::connector::Ask::ProcedureMessage { .. }
        | core::effects::connector::Ask::ProcedureClose { .. }
        | core::effects::connector::Ask::Project { .. }
        | core::effects::connector::Ask::Release { .. }
        | core::effects::connector::Ask::Subscribe { .. }
        | core::effects::connector::Ask::Unsubscribe { .. }
        | core::effects::connector::Ask::Read { .. }
        | core::effects::connector::Ask::Gather { .. }
        | core::effects::connector::Ask::CutTo { .. }
        | core::effects::connector::Ask::Take { .. }
        | core::effects::connector::Ask::Prepare { .. }
        | core::effects::connector::Ask::Left { .. }
        | core::effects::connector::Ask::Adopt { .. }
        | core::effects::connector::Ask::Restore { .. }
        | core::effects::connector::Ask::ReadAfresh
        | core::effects::connector::Ask::SettleOutbox => unreachable!("effect handoff vocabulary"),
        core::effects::connector::Ask::JudgeSettings { owner, revision, proposal, items } => {
            let token = domain.hub_link.fresh();
            domain.settings_judges.insert(token, owner).expect("settings judgment route fits");
            connector::Event::JudgeSettings { token, revision, proposal, items: settings_items(items) }
        }
        core::effects::connector::Ask::KeepSettings { revision, proposal, items } => {
            connector::Event::KeepSettings { revision, proposal, items: settings_items(items) }
        }
        core::effects::connector::Ask::KeepProposalSettings { proposal, items } => {
            connector::Event::KeepProposalSettings { proposal, items: settings_items(items) }
        }
        core::effects::connector::Ask::DropProposalSettings { proposal } => {
            connector::Event::DropProposalSettings { proposal }
        }
        core::effects::connector::Ask::SaveSettingsDefaults { revision } => {
            connector::Event::SaveSettingsDefaults { revision }
        }
    };
    work.push(Work::Connector { number, event });
}

/// Translate edits without inspecting the connector's bytes (connectors.md, 17).
fn settings_items(items: Box<[core::settings::application::Item]>) -> Box<[connector::SettingItem]> {
    let mut translated = List::with_capacity(u32::try_from(items.len()).expect("bounded settings items"));
    for item in items {
        let edit = match item.edit {
            core::settings::application::Edit::Set(bytes) => connector::SettingEdit::Set(bytes),
            core::settings::application::Edit::Remove => connector::SettingEdit::Remove,
            core::settings::application::Edit::Reset => connector::SettingEdit::Reset,
        };
        translated
            .push(connector::SettingItem { kind: item.kind, scope: item.scope, number: item.number, edit })
            .expect("one translated item per item");
    }
    translated.into_boxed()
}

fn name(path: connector::Path) -> authority::Name {
    let mut segments = List::with_capacity(u32::try_from(path.segments().len()).expect("bounded segments"));
    for segment in path.segments() {
        segments.push(segment.clone()).expect("one segment per path");
    }
    authority::Name { segments: segments.into_boxed() }
}

fn describe(
    number: u16,
    description: connector::Description,
    waits_for_dependencies: bool,
) -> core::effects::connector::EffectDescription {
    let form = match description.form {
        connector::Form::Creation => core::effects::connector::EffectForm::Creation,
        connector::Form::Transition => core::effects::connector::EffectForm::Transition,
        connector::Form::Set => core::effects::connector::EffectForm::Set,
    };
    let recovery = match description.recovery {
        connector::Recovery::Keyed => core::effects::connector::Recovery::Keyed,
        connector::Recovery::Conditional => core::effects::connector::Recovery::Conditional,
        connector::Recovery::Idempotent => core::effects::connector::Recovery::Idempotent,
        connector::Recovery::Unrecoverable => core::effects::connector::Recovery::Unrecoverable,
    };
    let mut resources = List::with_capacity(u32::try_from(description.resources.len()).expect("bounded resources"));
    for resource in description.resources {
        resources.push(name(resource)).expect("one name per resource");
    }
    let mut additional = List::with_capacity(resources.len().saturating_sub(1));
    let first = resources.as_slice().first().expect("connector described at least one resource").clone();
    for resource in resources.as_slice().get(1..).expect("at least one resource") {
        additional
            .push(authority::EffectResource { name: resource.clone(), access: authority::EffectAccess::Owned })
            .expect("additional resources");
    }
    let state = state_pin(description.state);
    core::effects::connector::EffectDescription {
        waits_for_dependencies,
        connector: number,
        purpose: description.purpose,
        form,
        recovery,
        effect: authority::Effect {
            connector: number,
            kind: description.kind,
            name: first,
            state,
            price: description.price,
            access: authority::EffectAccess::Owned,
            additional: additional.into_boxed(),
            guards: Box::new([]),
        },
    }
}

fn state_pin(state: u64) -> [u8; 32] {
    let bytes = state.to_be_bytes();
    let mut pin = [0; 32];
    for offset in 0..8 {
        *pin.get_mut(24usize.checked_add(offset).expect("pin offset")).expect("eight bytes fit the pin") =
            *bytes.get(offset).expect("eight-byte integer");
    }
    pin
}

/// Hold modes and current pool facts are translated without choosing admission.
fn report_hold(
    domain: &mut Domain,
    number: u16,
    path: &connector::Path,
    hold: connector::Hold,
) -> core::effects::connector::HoldKind {
    match hold {
        connector::Hold::None => core::effects::connector::HoldKind::Shared,
        connector::Hold::Exclusive { mode } => core::effects::connector::HoldKind::Exclusive {
            wait: match mode {
                connector::HoldMode::Wait => true,
                connector::HoldMode::Refuse => false,
            },
        },
        connector::Hold::Pooled { mode } => core::effects::connector::HoldKind::Pooled {
            slots: crate::numbered(domain, number).slots(path).unwrap_or(0),
            wait: match mode {
                connector::HoldMode::Wait => true,
                connector::HoldMode::Refuse => false,
            },
        },
    }
}

/// Translate current roles and holds together, scoped to the adopting project.
fn resource_reports(
    domain: &mut Domain,
    number: u16,
    project: u32,
    resources: Box<[connector::Named]>,
    work: &mut Queue<Work>,
) {
    for resource in resources {
        let hold = report_hold(domain, number, &resource.path, resource.hold);
        let role = match resource.role {
            connector::ResourceRole::Owned => core::effects::connector::ResourceRole::Owned,
            connector::ResourceRole::Participating => core::effects::connector::ResourceRole::Participant,
            connector::ResourceRole::Context => core::effects::connector::ResourceRole::Context,
            connector::ResourceRole::Unavailable => core::effects::connector::ResourceRole::Unavailable,
        };
        let name = tasks::Name { connector: number, path: name(resource.path).segments };
        work.push(Work::Core(core::Event::EffectConnector(core::effects::connector::Event::Resource {
            project,
            name,
            role,
            hold,
        })));
    }
}

fn application_answer(outcome: people::Outcome) -> connector::ApplicationCoreAnswer {
    match outcome {
        people::Outcome::Restored { .. } => unreachable!("a restore: restore-request"),
        people::Outcome::Released { .. } => connector::ApplicationCoreAnswer::Released,
        people::Outcome::Refused(why) => {
            connector::ApplicationCoreAnswer::Refused(crate::application_facts::refusal(why))
        }
        people::Outcome::Again(why) => connector::ApplicationCoreAnswer::Again(crate::application_facts::again(why)),
        people::Outcome::Compound { .. }
        | people::Outcome::CompoundRefused { .. }
        | people::Outcome::Applied { .. }
        | people::Outcome::SignedOut
        | people::Outcome::NoteEdited { .. }
        | people::Outcome::ServiceMade { .. }
        | people::Outcome::Adopted { .. }
        | people::Outcome::Created { .. }
        | people::Outcome::Proposed { .. }
        | people::Outcome::Withdrawn { .. }
        | people::Outcome::Stopped { .. }
        | people::Outcome::Cancelled { .. }
        | people::Outcome::PersonTaken { .. }
        | people::Outcome::PersonHandedBack { .. }
        | people::Outcome::PersonAnswered { .. }
        | people::Outcome::Moved { .. }
        | people::Outcome::ProposalDecided { .. }
        | people::Outcome::Said { .. }
        | people::Outcome::QuestionAnswered { .. }
        | people::Outcome::Amended { .. }
        | people::Outcome::RolesSet { .. }
        | people::Outcome::Renamed { .. }
        | people::Outcome::ProjectMade { .. }
        | people::Outcome::ProjectRemoved { .. }
        | people::Outcome::SettingsChanged { .. }
        | people::Outcome::MovedToNewest { .. }
        | people::Outcome::PolicyChanged { .. }
        | people::Outcome::PoolSet { .. }
        | people::Outcome::EscalationDecided { .. }
        | people::Outcome::AccountSignInStarted { .. }
        | people::Outcome::AccountProgramDeclared { .. }
        | people::Outcome::AccountKeyKept { .. }
        | people::Outcome::AccountMovingOut { .. }
        | people::Outcome::AccountFinishing { .. }
        | people::Outcome::AccountMovedIn { .. }
        | people::Outcome::AccountChanged
        | people::Outcome::AccountRemoved { .. }
        | people::Outcome::AccountSignedOut { .. }
        | people::Outcome::Lifted { .. }
        | people::Outcome::MarkedSeen { .. } => unreachable!("release core part returns a release terminal"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::fixtures;

    #[test]
    #[expect(clippy::too_many_lines, reason = "one restore round trip checks its edits, records and owner binding")]
    fn a_restore_ask_reaches_the_connector_and_its_answer_reaches_the_core() {
        let mut domain = fixtures::domain();
        let env = fixtures::env();
        let mut config = jig_test_connector_world::config(1);
        config.settings = Some(connector::SettingsKind {
            kind: 71,
            project: false,
            value_bytes: 16,
            per_scope: 4,
            charters: false,
            defaults: Box::from([]),
        });
        domain.first = connector::Domain::new(config, &env.limits.connector);
        let mut requests = Queue::uncapped();
        connector::step(
            &mut domain.first,
            &env.with(env.limits.connector),
            connector::Event::Restore {
                record: connector::Record::SettingReplaced {
                    kind: 71,
                    scope: 0,
                    number: 1,
                    revision: 7,
                    value: Box::from(&b"old"[..]),
                },
            },
            &mut requests,
        );
        assert!(requests.is_empty());
        let owner = Token::new(900);
        let edits = [
            core::settings::application::RestoredEdit::Set { revision: 7 },
            core::settings::application::RestoredEdit::Remove,
            core::settings::application::RestoredEdit::Reset,
            core::settings::application::RestoredEdit::Set { revision: 6 },
        ];
        let mut items = List::with_capacity(4);
        for (index, edit) in edits.into_iter().enumerate() {
            items
                .push(core::settings::application::Restored {
                    kind: 71,
                    scope: 0,
                    number: u32::try_from(index).expect("four items") + 1,
                    edit,
                })
                .expect("four items");
        }
        let mut out = Queue::uncapped();
        out.push(core::Request::Ask {
            connector: 1,
            ask: core::Ask::Effect(core::effects::connector::Ask::RestoreSettings {
                owner,
                proposal: 12,
                items: items.into_boxed(),
            }),
        });
        let mut decision = domain.journal.decide();
        let mut work = fixtures::work();
        route_core(&mut domain, &env, &mut decision, out, &mut work);
        let Work::Connector { number, event } = work.pop().expect("restore reaches connector") else {
            panic!("restore routes to connector");
        };
        assert_eq!(number, 1);
        let connector::Event::RestoreSettings { token, proposal, items } = &event else {
            panic!("restore keeps its proposal and edits");
        };
        assert_ne!(*token, owner);
        assert_eq!(*proposal, 12);
        assert_eq!(
            items.as_ref(),
            [
                connector::RestoredItem {
                    kind: 71,
                    scope: 0,
                    number: 1,
                    edit: connector::RestoredEdit::Set { revision: 7 }
                },
                connector::RestoredItem { kind: 71, scope: 0, number: 2, edit: connector::RestoredEdit::Remove },
                connector::RestoredItem { kind: 71, scope: 0, number: 3, edit: connector::RestoredEdit::Reset },
                connector::RestoredItem {
                    kind: 71,
                    scope: 0,
                    number: 4,
                    edit: connector::RestoredEdit::Set { revision: 6 }
                },
            ]
        );
        connector::step(&mut domain.first, &env.with(env.limits.connector), event, &mut requests);
        route_connector(&mut domain, &env, &mut decision, number, &mut requests, &mut work);
        let Some(Work::Core(core::Event::EffectConnector(core::effects::connector::Event::SettingsRestored {
            owner: returned,
            judged,
        }))) = work.pop()
        else {
            panic!("restored answer reaches core");
        };
        assert_eq!(returned, owner);
        assert_eq!(
            judged.as_ref(),
            [
                core::effects::connector::Judged::Accept { size: 3, charters: Box::from([]) },
                core::effects::connector::Judged::Accept { size: 0, charters: Box::from([]) },
                core::effects::connector::Judged::Accept { size: 0, charters: Box::from([]) },
                core::effects::connector::Judged::Refuse { why: Box::from(&b"no value kept"[..]) },
            ]
        );
        assert!(domain.settings_judges.is_empty());
        assert!(work.is_empty());
        domain.journal.accept(decision);
        assert_eq!(domain.journal.commit().expect("proposal records share one commit").writes.len(), 3);
        route_effect_ask(&mut domain, 1, core::effects::connector::Ask::ForgetReplaced { before: 8 }, &mut work);
        let Some(Work::Connector { number: 1, event }) = work.pop() else {
            panic!("forget reaches connector");
        };
        assert_eq!(event, connector::Event::ForgetReplaced { before: 8 });
        connector::step(&mut domain.first, &env.with(env.limits.connector), event, &mut requests);
        assert_eq!(
            requests.pop(),
            Some(connector::Request::Erase {
                key: connector::RecordKey::SettingReplaced { kind: 71, scope: 0, number: 1, revision: 7 },
            })
        );
        assert!(requests.is_empty());
    }
}
