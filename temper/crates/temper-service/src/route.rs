//! Exhaustive root routing to its protocol components.
use crate::{engine::Engine, stop};
use alloc::boxed::Box;
use temper_engine_domain as root;

pub(crate) mod box_link;
mod refusal;

#[expect(clippy::too_many_lines, reason = "one exhaustive dispatcher routes root outputs to their components")]
pub(crate) fn request(engine: &mut Engine, request: root::Request) {
    #[expect(clippy::match_same_arms, reason = "each request's arm is filled by its own task")]
    match request {
        root::Request::MakingCopied { key } => crate::setup::making::copied(engine, &key),
        root::Request::PlaygroundActs { .. } => {}
        root::Request::ResetAct { .. } => {}
        root::Request::ForgeKept(record) => crate::forges::record(engine, record),
        root::Request::ForgeGone { forge } => crate::forge::gone::request(engine, forge),
        root::Request::BoxGroup { forge, kind, request } => {
            let mut out = skein_lib::Queue::with_capacity(3);
            engine.box_link.group(forge, kind, request, &mut out);
            box_link::drain(engine, &mut out);
        }
        root::Request::BoxRemove { forge } => {
            let mut out = skein_lib::Queue::with_capacity(1);
            engine.box_link.remove(forge, &mut out);
            box_link::drain(engine, &mut out);
        }
        root::Request::Bound { request, answer } => engine.clients.bound(request, answer),
        root::Request::SetUp { forge, request, first } => {
            crate::setup::lifecycle::first(engine, forge, first);
            let mut out = skein_lib::Queue::with_capacity(3);
            engine.box_link.group(forge, temper_engine_domain_forge::BoxKind::Owned, request, &mut out);
            box_link::drain(engine, &mut out);
        }
        root::Request::Connect { forge, request, how, carried } => match how {
            temper_engine_domain_forge::How::GitHubApp { app, .. } => {
                crate::setup::public::connect(engine, forge, request, app, carried);
            }
            temper_engine_domain_forge::How::ForgejoAdmin { first } => {
                crate::setup::public::admin_connect(engine, forge, request, first, carried);
            }
            temper_engine_domain_forge::How::GitHubManifest { .. } => unreachable!("manifest has its own door"),
            temper_engine_domain_forge::How::ForgejoTokens => {
                crate::setup::public::tokens_connect(engine, forge, request, carried);
            }
        },

        root::Request::Replace { forge, request, carried } => {
            crate::setup::public::replace(engine, forge, request, carried);
        }
        root::Request::Manifest { forge, request, back } => {
            crate::setup::public::manifest(engine, forge, request, back);
        }
        root::Request::AdoptAccess { forge, request, owner, name } => {
            crate::setup::public::adopt_access(engine, forge, request, owner, name);
        }
        root::Request::Reachable { forge, request, installation, page } => {
            crate::setup::public::reachable(engine, forge, request, installation, page);
        }
        root::Request::Accept { forge, request, installation } => {
            crate::setup::public::accept(engine, forge, request, installation);
        }
        root::Request::Install { forge, request, back } => crate::setup::public::install(engine, forge, request, back),
        root::Request::EraseCarried { number } => crate::setup::public::erase(engine, number),
        root::Request::Setup { run } => crate::setup::request(engine, run),
        root::Request::Store(request) => store(engine, request),
        root::Request::Account(request) => {
            if let Some(accounts) = &mut engine.accounts {
                accounts.request(request);
            }
        }
        root::Request::Forge { call, repository, op } => {
            if engine.domain.forge_ended(repository.forge) {
                engine.events.push(root::Event::Forge(Box::new(temper_engine_domain_forge::Event::Client(
                    skein_forge_client::Event::Answered {
                        call,
                        cost: 0,
                        result: Err(skein_forge::api::Error::NotConnected),
                        free: 0,
                        rate: None,
                    },
                ))));
            } else {
                engine.forge.down(skein_forge_connection::Request::Call { call, repository, op });
            }
        }
        root::Request::ToChild(released) => {
            if engine.start_runtime() {
                engine.events.push(root::Event::Released(released));
            } else {
                engine.step(root::Event::Released(released));
            }
        }
        root::Request::Busy(event) => {
            use temper_engine_domain_forge::Event;
            #[expect(clippy::wildcard_enum_match_arm, reason = "only the routing producers below cross this boundary")]
            match *event {
                Event::Client(skein_forge_client::Event::Answered { call, cost, result, free, rate }) => {
                    engine.forge.down(skein_forge_connection::Request::Returned(skein_forge_connection::Answered {
                        call,
                        cost,
                        result,
                        free,
                        rate,
                    }));
                }
                event @ (Event::MakingStarted { .. }
                | Event::MakingCopying { .. }
                | Event::MakingMade { .. }
                | Event::MakingStopped { .. }
                | Event::MakingAdopt { .. }) => engine.events.push(root::Event::Forge(Box::new(event))),
                // skein's forge/connection.md, section 2: no returned-hint request; never offer it again.
                // protocol/forge/engine.md: these views are not kept; restart publishes connections and skein's forge/connection.md, 4.4's baseline.
                Event::AppRecorded { .. }
                | Event::Hint { .. }
                | Event::Connection { .. }
                | Event::Fetched { .. }
                | Event::Installation { .. }
                | Event::BoxClosed
                | Event::ConnectAnswered { .. }
                | Event::InstallAnswered { .. }
                | Event::ConnectEnded { .. }
                | Event::SetUpAnswered { .. }
                | Event::FirstMade { .. }
                | Event::FirstFailed { .. } => {}
                other => unreachable!("only forge, setup and box link routing produce root forge inputs: {other:?}"),
            }
        }
        root::Request::Unsettled(write) => {
            let drain = engine.drain.as_mut().expect("settling follows the service’s stop");
            drain.unsettled.push(write).expect("at most one unfinished write per forge entry");
        }
        root::Request::Drained => stop::drained(engine),
        root::Request::Stop => stop::root(engine),
        root::Request::Worker(request) => worker(engine, request),
        root::Request::Party(request) => {
            if let root::PartyRequest::WatchRefused { watcher, .. } = &request
                && engine.admission == Some(*watcher)
            {
                engine.admission = None;
            }
            let event = match request {
                root::PartyRequest::Deliver(root::Delivery::PlaygroundNote { line }) => {
                    engine.notes.push(crate::Note::Playground { line });
                    None
                }
                root::PartyRequest::Deliver(root::Delivery::Forges { watcher, forges }) => {
                    match forge_rows(engine, forges) {
                        Some(record) => Some(jig_protocol_engine_clients::Event::Application(
                            jig_protocol_engine_clients::FromApplication::Delivery {
                                watcher,
                                family: temper_client_protocol_wire::FORGE,
                                record,
                            },
                        )),
                        None => Some(jig_protocol_engine_clients::Event::Abort),
                    }
                }
                root::PartyRequest::Deliver(root::Delivery::Ways { watcher, document }) => {
                    match crate::family::ways_panel(*document, &temper_client_protocol_wire::v1::CEILINGS) {
                        Some(record) => Some(jig_protocol_engine_clients::Event::Application(
                            jig_protocol_engine_clients::FromApplication::Delivery {
                                watcher,
                                family: temper_client_protocol_wire::FORGE,
                                record,
                            },
                        )),
                        None => Some(jig_protocol_engine_clients::Event::Abort),
                    }
                }
                root::PartyRequest::Deliver(root::Delivery::Approvals { watcher, document }) => {
                    match crate::family::approvals(
                        *document,
                        &engine.workers.forges,
                        &temper_client_protocol_wire::v1::CEILINGS,
                    ) {
                        Some(record) => Some(jig_protocol_engine_clients::Event::Application(
                            jig_protocol_engine_clients::FromApplication::Delivery {
                                watcher,
                                family: temper_client_protocol_wire::FORGE,
                                record,
                            },
                        )),
                        None => Some(jig_protocol_engine_clients::Event::Abort),
                    }
                }
                root::PartyRequest::Deliver(root::Delivery::Trials { watcher, rows }) => {
                    match crate::family::trials(rows, &temper_client_protocol_wire::v1::CEILINGS) {
                        Some(record) => Some(jig_protocol_engine_clients::Event::Application(
                            jig_protocol_engine_clients::FromApplication::Delivery {
                                watcher,
                                family: temper_client_protocol_wire::FORGE,
                                record,
                            },
                        )),
                        None => Some(jig_protocol_engine_clients::Event::Abort),
                    }
                }
                root::PartyRequest::Deliver(root::Delivery::ForgeDocument { watcher, document }) => {
                    match crate::family::document(
                        *document,
                        &engine.workers.forges,
                        &temper_client_protocol_wire::v1::CEILINGS,
                    ) {
                        Some(record) => Some(jig_protocol_engine_clients::Event::Application(
                            jig_protocol_engine_clients::FromApplication::Delivery {
                                watcher,
                                family: temper_client_protocol_wire::FORGE,
                                record,
                            },
                        )),
                        None => Some(jig_protocol_engine_clients::Event::Abort),
                    }
                }
                root::PartyRequest::Deliver(root::Delivery::Panel { watcher, panel }) => {
                    match crate::family::panel(
                        *panel,
                        &engine.workers.forges,
                        &temper_client_protocol_wire::v1::CEILINGS,
                    ) {
                        Some(record) => Some(jig_protocol_engine_clients::Event::Application(
                            jig_protocol_engine_clients::FromApplication::Delivery {
                                watcher,
                                family: temper_client_protocol_wire::FORGE,
                                record,
                            },
                        )),
                        None => Some(jig_protocol_engine_clients::Event::Abort),
                    }
                }
                root::PartyRequest::Deliver(root::Delivery::ForgeRead { reply, answer }) => {
                    match crate::family::forge_read(
                        answer,
                        &engine.clients.program_environment,
                        &temper_client_protocol_wire::v1::CEILINGS,
                    ) {
                        Some(record) => Some(jig_protocol_engine_clients::Event::Application(
                            jig_protocol_engine_clients::FromApplication::Read {
                                reply,
                                family: temper_client_protocol_wire::FORGE,
                                record: Some(record),
                            },
                        )),
                        None => Some(jig_protocol_engine_clients::Event::Abort),
                    }
                }
                root::PartyRequest::Deliver(root::Delivery::Application { to, outcome }) => {
                    match crate::family::answer(
                        outcome,
                        &temper_client_protocol_wire::v1::CEILINGS,
                        &engine.clients.env.limits.wire,
                    ) {
                        Some(record) => Some(jig_protocol_engine_clients::Event::Application(
                            jig_protocol_engine_clients::FromApplication::Answered {
                                to,
                                family: temper_client_protocol_wire::FORGE,
                                record,
                            },
                        )),
                        None => Some(jig_protocol_engine_clients::Event::Abort),
                    }
                }
                request @ (root::PartyRequest::Catalogue { .. }
                | root::PartyRequest::SignInRefused { .. }
                | root::PartyRequest::WatchRefused { .. }
                | root::PartyRequest::RunText { .. }
                | root::PartyRequest::Deliver(_)) => crate::clients::translation::to_component(request),
            };
            if let Some(event) = event {
                engine.clients.down(event);
            }
        }
    }
}

fn store(engine: &mut Engine, request: root::StoreRequest) {
    if request == root::StoreRequest::Switch && !engine.store.pending_generation {
        engine.events.push(root::Event::Store(root::StoreInput::Switched));
        return;
    }
    if let root::StoreRequest::Commit { number, writes } = &request {
        engine.clients.commit(*number, writes);
    }
    if !engine.store.down(request) {
        stop::unavailable(engine, b"deployment");
    }
}

fn worker(engine: &mut Engine, request: root::WorkerRequest) {
    match request {
        root::WorkerRequest::Host(request) => host_request(engine, *request),
        root::WorkerRequest::ReadResult { payload, outcome } => {
            let change = match engine.domain.change_resource(payload) {
                Some((connector, kind, resource)) => {
                    Some(temper_engine_smith::ChangeResource { connector, kind, resource })
                }
                None => None,
            };
            engine.events.push(crate::tools::result(payload, &outcome, change));
        }
        root::WorkerRequest::Deliver(delivery) => match delivery {
            root::Delivery::AssignedJob { channel, job, workspace } => {
                let (run, attempt) = (job.run, job.attempt);
                match job_assignment(engine, channel, job, workspace.as_ref()) {
                    Some(event) => engine.workers.down(engine.env.now, event, &mut engine.events),
                    None => engine.events.push(root::Event::Worker(root::WorkerInput::Link {
                        channel,
                        link: root::Link::Up {
                            up: jig_domain_host::LinkUp::JobAnswer {
                                run,
                                attempt,
                                answer: jig_domain_host::JobEnding::Refused(jig_domain_host::JobRefusal::Invalid(
                                    jig_domain_host::JobInvalid::Start {
                                        why: b"assignment cannot encode".as_slice().into(),
                                    },
                                )),
                            },
                            parts: root::Parts::none(),
                        },
                    })),
                }
            }
            root::Delivery::Assigned { channel, assignment } => {
                // Take the restored deployment id, as the forge binds it at restore (domain/root.md, 8.1).
                engine.workers.encoding.deployment = engine.domain.deployment().id;
                engine.workers.assignment(engine.env.now, channel, assignment, &engine.values, &mut engine.events);
            }
            root::Delivery::Link { channel, down } => {
                // Includes addressed retryable capacity refusals, without a retained call flight.
                engine.workers.send(engine.env.now, channel, down, &engine.values, &mut engine.events);
            }
            root::Delivery::TurnBusy { channel, task, attempt, turn } => engine.workers.down(
                engine.env.now,
                jig_protocol_engine_workers::Event::TurnBusy {
                    channel,
                    run: skein_lib::Token::new(task),
                    attempt: skein_lib::Token::new(attempt),
                    turn,
                },
                &mut engine.events,
            ),
            root::Delivery::Refuse { channel, hello } => {
                let event = refuse(engine, channel, hello);
                engine.workers.down(engine.env.now, event, &mut engine.events);
            }
            root::Delivery::Host(delivery) => host_delivery(engine, *delivery),
            root::Delivery::PlaygroundNote { .. }
            | root::Delivery::Procedure { .. }
            | root::Delivery::Application { .. }
            | root::Delivery::Forges { .. }
            | root::Delivery::ForgeDocument { .. }
            | root::Delivery::Trials { .. }
            | root::Delivery::Ways { .. }
            | root::Delivery::Approvals { .. }
            | root::Delivery::Panel { .. }
            | root::Delivery::Public { .. }
            | root::Delivery::WebReply { .. }
            | root::Delivery::CarriedUnused { .. }
            | root::Delivery::ForgeRead { .. }
            | root::Delivery::Read { .. }
            | root::Delivery::Document { .. }
            | root::Delivery::WatchEnded { .. }
            | root::Delivery::SourceSignedOut { .. }
            | root::Delivery::Result { .. }
            | root::Delivery::ProjectMade { .. }
            | root::Delivery::ProjectReleased { .. }
            | root::Delivery::MoveStanding { .. }
            | root::Delivery::ForgeCommitted { .. }
            | root::Delivery::ForgeCall { .. }
            | root::Delivery::Bound { .. }
            | root::Delivery::Relay { .. }
            | root::Delivery::ReadEscalationDecision { .. }
            | root::Delivery::Fleet(_)
            | root::Delivery::ReadLoad { .. }
            | root::Delivery::PublicWait { .. }
            | root::Delivery::NotesLoad { .. }
            | root::Delivery::Start(_)
            | root::Delivery::Load { .. }
            | root::Delivery::Drained => stop::unavailable(engine, b"workers"),
        },
    }
}

fn host_request(engine: &mut Engine, request: root::HostRequest) {
    match request {
        root::HostRequest::Decode { to, task, attempt, call } => {
            engine.events.push(crate::tools::decode_configured(
                to,
                task,
                attempt,
                &call,
                engine.domain.models(),
                &engine.domain.catalogue(),
                &engine.domain.repository_identities(task),
            ));
        }
        root::HostRequest::SettingsNote { to, call, body } => {
            engine.events.push(crate::tools::settings_note(to, *call, &body, &engine.domain.catalogue()));
        }
        root::HostRequest::Dropped { .. } => {}
    }
}

pub(crate) fn forge_rows(engine: &Engine, forges: temper_engine_domain_forge::Forges) -> Option<Box<[u8]>> {
    crate::family::forges(
        engine.forge.origins.rows(forges),
        engine.clients.linked(),
        &temper_client_protocol_wire::v1::CEILINGS,
    )
}

fn host_delivery(engine: &mut Engine, delivery: root::HostDelivery) {
    match delivery {
        root::HostDelivery::Settings {
            to,
            answer,
            repositories,
            people,
            forge_people,
            offer,
            application,
            from,
            requested,
        } => {
            let (body, error) = temper_engine_smith::settings_repository_answer(
                &answer,
                from,
                requested,
                &engine.domain.catalogue(),
                application.as_ref(),
                &repositories,
                &people,
                &forge_people,
                &offer,
                engine.env.limits.call_answer_bytes.saturating_sub(8),
            )
            .into_parts();
            engine.events.push(root::Event::Worker(root::WorkerInput::RenderedHostCall {
                to,
                answer: jig_domain_core::SettledAnswer::Host { error, body },
            }));
        }
        root::HostDelivery::See { to, line, from, answer } => {
            let maximum = engine
                .env
                .limits
                .call_answer_bytes
                .saturating_sub(3)
                .min(engine.env.limits.host_reply_bytes.unwrap_or(u32::MAX));
            let (body, error) = temper_engine_smith::see_answer(
                &line,
                from,
                &answer,
                &engine.domain.catalogue(),
                maximum,
                temper_engine_smith::ForgeReadBounds {
                    rows: engine.env.limits.forge.client.rows,
                    answer_bytes: engine.env.limits.forge.client.answer_bytes,
                },
            )
            .into_parts();
            engine.events.push(root::Event::Worker(root::WorkerInput::RenderedHostCall {
                to,
                answer: jig_domain_core::SettledAnswer::Host { error, body },
            }));
        }
        root::HostDelivery::Render { to, answer, tool, key, job, page, .. } => {
            render_host(engine, to, &answer, &tool, key, job, page);
        }
        root::HostDelivery::Answer { channel, task, attempt, name, call } => engine.workers.send(
            engine.env.now,
            channel,
            jig_domain_host::LinkDown::Relayed {
                run: skein_lib::Token::new(task),
                attempt: skein_lib::Token::new(attempt),
                call: name,
                answer: crate::tools::settled(call.answer),
            },
            &engine.values,
            &mut engine.events,
        ),
    }
}

fn render_host(
    engine: &mut Engine,
    to: skein_lib::ReplyTo,
    answer: &root::CallAnswer,
    tool: &[u8],
    key: jig_domain_core::CallKey,
    job: Option<skein_forge::api::JobAttempt>,
    page: Option<(u32, u32)>,
) {
    let bytes =
        engine.env.limits.call_answer_bytes.saturating_sub(u32::try_from(tool.len()).expect("bounded tool name"));
    engine.events.push(crate::tools::render_for_task_page(
        to,
        answer,
        tool,
        bytes,
        key.task,
        engine.domain.reply_bound(key).unwrap_or(u32::MAX),
        temper_engine_smith::ForgeReadBounds {
            rows: engine.env.limits.forge.client.rows,
            answer_bytes: engine.env.limits.forge.client.answer_bytes,
        },
        job,
        page,
    ));
    if let Some(reason) = temper_engine_smith::unasked_kind(answer) {
        engine.notes.push(crate::Note::ReadRefused {
            task: key.task,
            attempt: key.attempt,
            tool: skein_lib::bytes::copy_of(tool),
            layer: "host-render",
            reason,
            findings: Box::new([]),
            error: None,
            job,
        });
    }
}

/// A job uses the same repository translation as an agent (temper/docs/design/domain/worker/jobs.md).
pub(crate) fn job_assignment(
    engine: &Engine,
    channel: skein_lib::Token,
    job: jig_domain_host::JobAssignment,
    workspace: Option<&root::ForgeWorkspace>,
) -> Option<jig_protocol_engine_workers::Event> {
    crate::assignment::job_down(
        job,
        workspace,
        &engine.workers.forges,
        &engine.workers.encoding,
        &engine.values,
        channel,
        engine.env.now,
    )
}

/// Build the channel event and note from the same words (protocol/cluster.md, 3).
pub(crate) fn refuse(
    engine: &mut Engine,
    channel: skein_lib::Token,
    hello: Option<jig_domain_fleet::HelloRefused>,
) -> jig_protocol_engine_workers::Event {
    let text = refusal::text(hello);
    if engine.last_hello_refusal.as_deref() != Some(text.as_ref()) {
        engine.last_hello_refusal = Some(text.clone());
        engine.notes.push(crate::Note::HelloRefused { channel: channel.raw(), text: text.clone() });
    }
    jig_protocol_engine_workers::Event::NotAdmitted { channel, text }
}
