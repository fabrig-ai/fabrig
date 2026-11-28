//! Total boundary translations for the testing application's tools, effects
//! and restart script (`domain/root.md`, sections 3 and 11;
//! `domain/engine/restart.md` and `domain/engine/executors.md`).

use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_contract as jig;
use jig_domain_core as core;
use jig_domain_core::engine::{self, In, Out, Peer, application};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_connector as connector;
use skein_lib::{Decision, Env, List, Queue};

mod workspace;

use crate::{Delivery, Domain, Key, Limits, Record, Work, Write, hold, write};

pub(super) fn route_core(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision<Write, Delivery>,
    requests: Queue<core::Request>,
    work: &mut Queue<Work>,
) {
    let mut out = Queue::with_capacity(requests.len());
    engine::append(requests, &mut out);
    route_engine(domain, env, decision, out, work);
}

pub(super) fn step_engine(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision<Write, Delivery>,
    input: In,
    work: &mut Queue<Work>,
) {
    // The connector keeps its document under the construction token, not the party's watcher.
    let document = if let In::Connector(number, jig::Up::Parts(jig::parts::Up::Made { watcher, .. })) = &input
        && let Some(document) = crate::numbered(domain, number.0).take_document(*watcher)
    {
        Some((number.0, document))
    } else {
        None
    };
    let mut out = Queue::uncapped();
    engine::step(&mut domain.engine, &env.with(env.limits.core), input, &mut out);
    route_engine_outputs(domain, env, decision, out, work, document);
}

pub(super) fn route_engine(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision<Write, Delivery>,
    out: Queue<Out>,
    work: &mut Queue<Work>,
) {
    route_engine_outputs(domain, env, decision, out, work, None);
}

fn route_engine_outputs(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision<Write, Delivery>,
    mut out: Queue<Out>,
    work: &mut Queue<Work>,
    mut document: Option<(u16, connector::ClientDocument)>,
) {
    for _ in 0..out.len() {
        match out.pop().expect("engine output count") {
            Out::Core(request) => {
                if let core::Request::Held(held) = &request
                    && let core::Held::ApplicationDelivery { watcher, connector: number, .. } = **held
                {
                    let (owner, value) = document.take().expect("construction owns the delivered document");
                    assert_eq!(owner, number, "construction keeps its connector");
                    hold(decision, Delivery::ApplicationDocument { watcher, number, document: Box::new(value) });
                } else {
                    route_request(domain, env, decision, request, work);
                }
            }
            Out::Write(write) => route_request(domain, env, decision, core::Request::Write(write), work),
            Out::Now(Peer::Application(report)) => match report {
                application::Out::Start { phase } => {
                    domain.start_phase = phase;
                    domain.start_opened = phase.is_none();
                }
                application::Out::Numbered { commits } => domain.decision_numbering = Some(commits),
                application::Out::Refused { phase, failure } => domain.start_failure = Some((phase, failure)),
            },
            Out::Held(Peer::Resume(resume)) => hold(decision, Delivery::Resume(resume)),
            Out::Now(Peer::Store(_) | Peer::Connector(..) | Peer::Resume(_)) | Out::Held(Peer::Application(_)) => {
                unreachable!("engine peers use their declared durability class")
            }
            Out::Ask(connector, ask) => match connector.0 {
                1 => work.push(Work::Connector { number: 1, event: connector::Event::Ask(ask) }),
                2 => work.push(Work::Connector { number: 2, event: connector::Event::Ask(ask) }),
                _ => unreachable!("engine asks a configured connector"),
            },
            Out::Held(Peer::Connector(connector, released)) => hold(decision, Delivery::Connector(connector, released)),
            Out::Held(Peer::Store(load)) => hold(decision, Delivery::Store(load)),
        }
    }
}

pub(super) fn route_request(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision<Write, Delivery>,
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
        core::Request::Held(held) => route_held(domain, env, decision, *held, work),
        core::Request::Now(now) => route_now(domain, env, decision, *now, work),
        core::Request::Ask { connector, ask } => route_core_ask(domain, env, decision, connector, ask),
        core::Request::ProjectMade { project } => hold(decision, Delivery::ProjectMade { project }),
    }
}

fn route_held(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision<Write, Delivery>,
    held: core::Held,
    work: &mut Queue<Work>,
) {
    if let core::Held::PublicRead { .. } | core::Held::PublicWait { .. } = held {
        hold(decision, Delivery::Core(held));
    } else if let core::Held::ApplicationDelivery { .. } = held {
        unreachable!("document delivery is paired with its construction output");
    } else if let core::Held::ApplicationWatchEnded { watcher, connector: number, subject, why } = held {
        work.push(Work::Connector { number, event: connector::Event::DocumentForget { subject } });
        hold(decision, Delivery::Core(core::Held::WatchEnded { watcher, why }));
    } else if let core::Held::Applied { to, connector: number, token } = held {
        let outcome = crate::numbered(domain, number)
            .application_outcome(token)
            .expect("the connector retains every kept people token");
        hold(decision, Delivery::Application { to, number, outcome });
    } else if let core::Held::Assign { channel, run, attempt, .. } = held {
        crate::link::held(domain, env.now, decision, jig_domain_hub_link::Event::Assign { channel, run, attempt });
    } else if let core::Held::SettledCall { to, call, .. } = held {
        crate::link::held(domain, env.now, decision, jig_domain_hub_link::Event::SettledCall { to, call });
    } else if let core::Held::Relayed { channel, run, attempt, call, answer } = held {
        crate::link::held(
            domain,
            env.now,
            decision,
            jig_domain_hub_link::Event::Relayed { channel, run, attempt, call, answer },
        );
    } else if let core::Held::Inbound { channel, run, attempt, message } = held {
        crate::link::held(
            domain,
            env.now,
            decision,
            jig_domain_hub_link::Event::Inbound { channel, run, attempt, message },
        );
    } else {
        if let core::Held::Store(_) = held {
            unreachable!("loads cross the store port");
        }
        hold(decision, crate::link::down(domain, Delivery::Core(held)));
    }
}

fn route_core_ask(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision<Write, Delivery>,
    number: u16,
    ask: core::Ask,
) {
    if let core::Ask::ClaimDone { task, attempt, kinds, version } = ask {
        if number == 1 {
            crate::link::held(
                domain,
                env.now,
                decision,
                jig_domain_hub_link::Event::Start { task, attempt, kinds, version },
            );
        }
        return;
    }
    if let core::Ask::JobClaimed { task, attempt, kind, spec, deadline, result, grants, workspace } = ask {
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
        return;
    }
    if let core::Ask::Workers(workers) = ask {
        domain.now.push(crate::Request::Workers(workers));
        return;
    }
    if let core::Ask::Catalogue { models } = ask {
        domain.catalogue_pending = true;
        let names = core::settings::model_names(&core::settings::catalogue::rows(&models));
        domain.now.push(crate::Request::StoreCatalogue { names });
        domain.now.push(crate::Request::ClientsCatalogue { models });
        return;
    }
    if let core::Ask::Providers(_) = ask {
        return;
    }
    unreachable!("connector asks cross the engine port: {ask:?}");
}

#[expect(clippy::too_many_lines, reason = "one exhaustive root continuation boundary")]
fn route_now(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision<Write, Delivery>,
    now: core::Now,
    work: &mut Queue<Work>,
) {
    match now {
        core::Now::PartBusy { key } => work.push(Work::Core(core::Event::Fleet(jig_domain_fleet::Event::PartBusy {
            run: skein_lib::Token::new(key.task),
            attempt: skein_lib::Token::new(key.attempt),
            part: key.number,
        }))),
        core::Now::Ask { channel, run, attempt, number, bytes } => {
            work.push(Work::Core(core::Event::Fleet(jig_domain_fleet::Event::Asked {
                channel,
                run,
                attempt,
                number,
                answer: jig_domain_link::AskAnswer::Bytes(bytes),
            })));
        }
        core::Now::Link(core::LinkNow::AnswerPayload { run, attempt, payload, worker, window }) => {
            crate::link::answer(domain, run, attempt, payload, worker, window, work);
        }
        core::Now::Link(value) => crate::link::route(domain, env.now, jig_domain_hub_link::now(value), work),
        core::Now::RelayBusy { channel, run, attempt, call } => {
            domain.hub_link.forget_call(&call);
            let delivery = crate::link::down(
                domain,
                Delivery::CallBusy { channel, task: run.raw(), attempt: attempt.raw(), name: call },
            );
            assert!(domain.journal.now(delivery).is_ok(), "reserved busy answer room");
        }
        core::Now::CallBusy { to } | core::Now::NoteBusy { to, .. } => crate::stopping::call(domain, to, work),
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
        core::Now::WorkspaceRequest { task, attempt, context } => {
            prepare_workspace(domain, env, decision, work, task, attempt, &context);
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
            parts,
            grants,
            brief,
            ..
        } => {
            let answered = domain.engine.core().settled_calls(task, attempt);
            let writes = domain.workspace_writes.remove(&task).expect("assembled workspace writes");
            crate::link::route(
                domain,
                env.now,
                jig_domain_hub_link::Event::Prepared {
                    assignment: crate::Assignment {
                        parts,
                        live,
                        task,
                        attempt,
                        conversation,
                        brief,
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
        core::Now::BriefFailed { task, failure } => {
            domain.brief_failures.push((task, failure));
        }
        core::Now::RunPreparationFailed { task } | core::Now::DropAssignment { task } => {
            drop(domain.hub_link.take_assignment(task));
            drop(domain.workspace_writes.remove(&task));
        }
        core::Now::RestoreRefused => {} // The application port carries the typed refusal.

        other @ (core::Now::SettledCallRefused { .. }
        | core::Now::Call { .. }
        | core::Now::EffectAnswer { .. }
        | core::Now::TaskRead { .. }
        | core::Now::ProposalRead { .. }
        | core::Now::SettingsAnswer { .. }
        | core::Now::PersonAnswer { .. }
        | core::Now::SignInRefused { .. }
        | core::Now::WatchRefused { .. }
        | core::Now::Account(_)
        | core::Now::RunText { .. }
        | core::Now::NotesIndexed { .. }
        | core::Now::NotesRecalled { .. }
        | core::Now::NotesRefused { .. }) => domain.now.push(crate::Request::Now(other)),
    }
}

pub(super) fn route_connector(
    domain: &mut Domain,
    decision: &mut Decision<Write, Delivery>,
    number: u16,
    requests: &mut Queue<connector::Out>,
    work: &mut Queue<Work>,
) {
    for _ in 0..requests.len() {
        match requests.pop().expect("connector output count") {
            connector::Out::Up(up) => work.push(Work::Engine(In::Connector(jig::Connector(number), up))),
            connector::Out::Save(record) => write(domain, decision, Write::Save(Record::Connector { number, record })),
            connector::Out::Erase(key) => write(domain, decision, Write::Erase(Key::Connector { number, key })),
            connector::Out::System(call) => hold(decision, Delivery::System { connector: number, call }),
            connector::Out::LostRead { task, attempt, resource, call } => {
                hold(decision, Delivery::LostRead { connector: number, task, attempt, resource, call });
            }
            connector::Out::Projection(feed) => hold(decision, Delivery::Projection { connector: number, feed }),
            connector::Out::Procedure { task, step, code, context } => {
                hold(decision, Delivery::Procedure { task, step, connector: number, code, context });
            }
            connector::Out::ReadAnswered { reply, answer } => {
                hold(decision, Delivery::ApplicationRead { reply, number, answer });
            }
            connector::Out::EffectOwner { effect, origin } => {
                let owner = domain.engine.core_mut().effect_owner();
                work.push(Work::Connector {
                    number,
                    event: connector::Event::StageEffect { owner, effect, start: Some(origin) },
                });
            }
            connector::Out::ApplicationRelease { request, project, release } => {
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
            connector::Out::Topic { token, resources } => {
                crate::topics::described(domain, number, token, resources, work);
            }
            connector::Out::LoadLive { after } => hold(decision, Delivery::ConnectorLoad { number, after }),
        }
    }
}

pub(crate) fn declaration(
    connector: jig_domain_contract::Connector,
    config: &connector::Config,
    resources: &[tasks::ResourceKind],
) -> jig_domain_contract::Declaration {
    use jig_domain_contract::effects::{EffectForm, EffectKind, Recovery};
    let mut effects = List::with_capacity(u32::try_from(config.kinds.len()).expect("configured effect kinds"));
    for kind in &config.kinds {
        let form = match kind.form {
            connector::Form::Creation => EffectForm::Creation,
            connector::Form::Transition => EffectForm::Transition,
            connector::Form::Set => EffectForm::Set,
        };
        let recovery = match kind.recovery {
            connector::Recovery::Keyed => Recovery::Keyed,
            connector::Recovery::Conditional => Recovery::Conditional,
            connector::Recovery::Idempotent => Recovery::Idempotent,
            connector::Recovery::Unrecoverable => Recovery::Unrecoverable,
        };
        effects
            .push(EffectKind { kind: kind.kind, form, recovery, waits_for_dependencies: kind.waits_for_dependencies })
            .expect("one declaration per configured effect kind");
    }
    jig_domain_contract::Declaration {
        connector,
        resources: Box::from(resources),
        effects: effects.into_boxed(),
        projects_goals: true,
        sections: true,
    }
}

fn prepare_workspace(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision<Write, Delivery>,
    work: &mut Queue<Work>,
    task: u64,
    attempt: u64,
    context: &tasks::RunContext,
) {
    let checked = workspace_checks(domain, env, decision, context, work);
    let (holds, credentials, writes) = match checked {
        Some(WorkspaceChecks { names, writes: checks }) => {
            let holds = match context.executor {
                tasks::Executor::Job { .. } => names.clone(),
                tasks::Executor::Agent { .. } | tasks::Executor::Procedure { .. } | tasks::Executor::Person(_) => {
                    Box::new([])
                }
            };
            assert!(domain.workspace_writes.insert(task, names).is_ok(), "workspace per task");
            match workspace::credentials(domain, &env.limits, context.project, &context.spec) {
                Some(credentials) => (holds, credentials, Some(checks)),
                None => (holds, Box::<[u32]>::from([]), None),
            }
        }
        None => (Box::<[tasks::Name]>::from([]), Box::<[u32]>::from([]), None),
    };
    let event = core::Event::WorkspacePrepared { task, attempt, holds, credentials, writes, workspace: None };
    if let Some(waiting) = &mut domain.scripted_workspaces {
        waiting.push(event);
    } else {
        work.push(Work::Core(event));
    }
}

struct WorkspaceChecks {
    names: Box<[tasks::Name]>,
    writes: Box<[authority::Write]>,
}

fn workspace_checks(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision<Write, Delivery>,
    context: &tasks::RunContext,
    work: &mut Queue<Work>,
) -> Option<WorkspaceChecks> {
    let mut names = List::with_capacity(env.limits.core.tasks.holdings);
    let mut checks = List::with_capacity(env.limits.core.authority.writes);
    for number in [1, 2] {
        let mut out = Queue::uncapped();
        let holdings = crate::numbered(domain, number).workspace_holdings(context.project, &context.spec, &mut out);
        route_connector(domain, decision, number, &mut out, work);
        for holding in holdings? {
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
    Some(WorkspaceChecks { names: names.into_boxed(), writes: checks.into_boxed() })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::fixtures;
    use skein_lib::Token;

    #[test]
    #[expect(clippy::too_many_lines, reason = "check the restore edits, answer and writes in one round trip")]
    fn a_restore_ask_reaches_the_connector_and_its_answer_reaches_the_core() {
        let mut domain = fixtures::domain();
        let env = fixtures::env();
        let mut config = jig_test_connector_kit::config(1);
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
        for (index, edit) in edits.iter().cloned().enumerate() {
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
        let ask = jig::Ask::Settings(jig::settings::Ask::Restore { owner, proposal: 12, items: items.into_boxed() });
        out.push(Out::Ask(jig::Connector(1), ask));
        let mut decision = domain.journal.decide();
        let mut work = fixtures::work();
        route_engine(&mut domain, &env, &mut decision, out, &mut work);
        let Work::Connector { number, event } = work.pop().expect("restore reaches connector") else {
            panic!("restore routes to connector");
        };
        assert_eq!(number, 1);
        let connector::Event::Ask(jig::Ask::Settings(jig::settings::Ask::Restore { owner: asked, proposal, items })) =
            &event
        else {
            panic!("restore keeps its owner, proposal and edits");
        };
        assert_eq!((*asked, *proposal), (owner, 12));
        assert_eq!(items.len(), 4);
        for (item, edit) in items.iter().zip(edits) {
            assert_eq!((item.kind, item.scope, &item.edit), (71, 0, &edit));
        }
        connector::step(&mut domain.first, &env.with(env.limits.connector), event, &mut requests);
        route_connector(&mut domain, &mut decision, number, &mut requests, &mut work);
        let Some(Work::Engine(In::Connector(
            jig::Connector(1),
            jig::Up::Settings(jig::settings::Up::Restored { owner: returned, judged }),
        ))) = work.pop()
        else {
            panic!("restored answer reaches core");
        };
        assert_eq!(returned, owner);
        assert_eq!(
            judged.as_ref(),
            [
                jig::settings::Judged::Accept { size: 3, charters: Box::from([]) },
                jig::settings::Judged::Accept { size: 0, charters: Box::from([]) },
                jig::settings::Judged::Accept { size: 0, charters: Box::from([]) },
                jig::settings::Judged::Refuse { why: Box::from(&b"no value kept"[..]) },
            ]
        );
        assert!(work.is_empty());
        domain.journal.accept(decision);
        assert_eq!(domain.journal.commit().expect("proposal records share one commit").writes.len(), 3);
        let mut out = Queue::uncapped();
        out.push(Out::Ask(jig::Connector(1), jig::Ask::Settings(jig::settings::Ask::ForgetReplaced { before: 8 })));
        let mut decision = domain.journal.decide();
        route_engine(&mut domain, &env, &mut decision, out, &mut work);
        let Some(Work::Connector { number: 1, event }) = work.pop() else {
            panic!("forget reaches connector");
        };
        connector::step(&mut domain.first, &env.with(env.limits.connector), event, &mut requests);
        let Some(connector::Out::Erase(connector::RecordKey::SettingReplaced {
            kind: 71,
            scope: 0,
            number: 1,
            revision: 7,
        })) = requests.pop()
        else {
            panic!("that replaced setting was erased");
        };
        assert!(requests.is_empty());
    }
}
