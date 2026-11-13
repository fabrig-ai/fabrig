//! The protocol layer the engine world plays (testing-strategy.md, 4): the
//! hub's link records a worker sends, the decode and render of engine tools,
//! the reading of an ended run's outcome, and the accounts component.
//! jig's worlds play the same parts (`testing/jig-test-service`).

use jig_domain_accounts as accounts;
use jig_domain_core as core;
use jig_domain_host as host;
use jig_domain_link::{Bounds, EndpointBound, Form, Hosting, JobBound, JobKind, Phase, Scheme};
use jig_domain_tasks as tasks;
use skein_lib::{Duration, ReplyTo, Token};
use std::collections::{BTreeMap, BTreeSet};
use temper_engine_domain as engine;

/// The bounds every scripted worker says in its hello: wide enough for the
/// world's one charter (jig's `domain/hosts/charters.md`).
#[must_use]
pub fn bounds() -> Bounds {
    Bounds {
        program_endpoints: Box::new([]),
        endpoints: Box::new([EndpointBound { endpoint: 1, window: 1024, output: 1024 }]),
        turns: 8,
        spend: 1000,
        time: Duration::from_secs(60),
        waiting: Duration::from_secs(60),
        // Checks finish within a changing charter's reserved time.
        checks: Duration::from_secs(10),
        message_bytes: 128,
        inbox: 4,
        unacknowledged: 8,
        reply_bytes: None,
    }
}

/// The merge kind advertised by the scripted worker (temper/docs/design/domain/worker/jobs.md).
#[must_use]
pub fn merge_bound() -> JobBound {
    JobBound { kind: JobKind(3), spec: 4096, result: 512, deadline: Duration::from_secs(600) }
}

/// One worker's hello on `channel`, listing the runs it hosts.
#[must_use]
pub fn hello(channel: Token, hosting: Box<[Hosting]>) -> engine::Event {
    hello_slots(channel, 1, hosting)
}

/// One worker's hello on `channel` with `slots` slots, listing the runs it hosts.
#[must_use]
pub fn hello_slots(channel: Token, slots: u32, hosting: Box<[Hosting]>) -> engine::Event {
    hello_at(channel, 0, slots, hosting)
}

/// A configured worker's hello at `worker`, with its slots and hosted runs.
#[must_use]
pub fn hello_at(channel: Token, worker: u32, slots: u32, hosting: Box<[Hosting]>) -> engine::Event {
    hello_with(channel, worker, slots, bounds(), hosting)
}

/// A configured worker's hello at `worker`, saying `bounds`.
#[must_use]
pub fn hello_with(channel: Token, worker: u32, slots: u32, bounds: Bounds, hosting: Box<[Hosting]>) -> engine::Event {
    link(
        channel,
        engine::Link::Hello {
            worker,
            hello: jig_domain_link::Hello {
                jobs: Box::new([JobBound { kind: JobKind(1), ..merge_bound() }, merge_bound()]),
                slots,
                stop_bound: Duration::from_millis(100),
                workstreams: Box::new([]),
                hosting,
                bounds,
            },
        },
    )
}

/// A hosted run in a worker's hello.
#[must_use]
pub fn hosting(task: u64, attempt: u64, phase: Phase) -> Hosting {
    Hosting { run: Token::new(task), attempt: Token::new(attempt), phase }
}

/// A worker channel closing.
#[must_use]
pub fn lost(channel: Token) -> engine::Event {
    link(channel, engine::Link::Lost)
}

/// One numbered turn of a run: its cumulative spend, the last message it
/// read, and its opaque body. The first turn opens the run's window.
#[must_use]
pub fn turn(
    channel: Token,
    task: u64,
    attempt: u64,
    number: u32,
    cumulative: u64,
    read: Option<u64>,
    body: &[u8],
) -> engine::Event {
    up(
        channel,
        host::LinkUp::Turn {
            run: Token::new(task),
            attempt: Token::new(attempt),
            turn: host::Turn {
                opens_window: number == 1,
                refused: Box::new([]),
                turn: number,
                spent: cumulative,
                read: read.map(Token::new),
                body: body.into(),
            },
        },
        engine::Parts::none(),
    )
}

/// A scripted worker's terminal answer when its checkout could not be prepared.
#[must_use]
pub fn failed_preparation(channel: Token, task: u64, attempt: u64, preparation: engine::Preparation) -> engine::Event {
    let ending = host::Ending::Failed {
        failure: host::Failure::Unprepared(host::Preparation::Permanent { resource: None }),
        detail: Box::new([]),
        work: host::Work { left: None, saved: None },
    };
    let answer = host::RunAnswer { read: None, refused: Box::new([]), turns: 0, spent: 1, ending };
    let parts = engine::Parts { saved: None, pushed: Box::new([]), preparation: Some(preparation) };
    up(channel, host::LinkUp::Answer { run: Token::new(task), attempt: Token::new(attempt), answer }, parts)
}

/// One hub record with no application part.
#[must_use]
pub fn up(channel: Token, up: host::LinkUp, parts: engine::Parts) -> engine::Event {
    link(channel, engine::Link::Up { up, parts })
}

fn link(channel: Token, link: engine::Link) -> engine::Event {
    engine::Event::Worker(engine::WorkerInput::Link { channel, link })
}

type DecodeAnswer = (bool, Box<[u8]>);

/// What the world's protocol layer keeps: tool bodies by call name, ended
/// runs' terminals by outcome bytes, and every typed answer it rendered.
#[derive(Hash, Debug, Default)]
pub struct Protocol {
    pub sign_in_sources: BTreeSet<u32>,
    /// The deployment models in numbered execution positions.
    pub(crate) models: Box<[engine::Model]>,
    /// Whole model settings used by internal settings-note decoding.
    pub(crate) catalogue: Box<[core::settings::CatalogueModel]>,
    /// Catalogue replacements for the clients component joined to this world.
    pub catalogues: Vec<Box<[core::settings::CatalogueModel]>>,
    calls: BTreeMap<Box<[u8]>, engine::Call>,
    decode_answers: BTreeMap<Box<[u8]>, DecodeAnswer>,
    ends: BTreeMap<Box<[u8]>, tasks::End>,
    serial: u64,
    /// Each decided call's typed answer, by its opaque call name, in render order.
    pub answers: Vec<(Box<[u8]>, engine::CallAnswer)>,
    pub read_pages: BTreeMap<Box<[u8]>, (u32, u32)>,
    pub read_jobs: BTreeMap<Box<[u8]>, skein_forge::api::JobAttempt>,
    /// The root's render-only asking route, separate from each durable decision.
    pub asks: BTreeMap<Box<[u8]>, engine::AskHow>,
    /// Let a story cut between the durable decision and its rendered answer.
    pub defer_render: bool,
    pub pending_renders: Vec<(ReplyTo, Box<[u8]>)>,
    /// Each account component request the root made.
    pub accounts: Vec<accounts::Request>,
    pub key_failure: Option<accounts::KeyFailure>,
    /// Worker channels that received their program-source inventory.
    pub sources: Vec<Token>,
    /// Core read loads the world answered as gone.
    pub reads: Vec<core::reads::Range>,
    /// List reads held by the scripted worker until the story supplies an answer.
    pub list_reads: std::collections::VecDeque<(Token, u64)>,
}

impl Protocol {
    /// A run's call that the world's protocol answers as an undecodable tool.
    pub fn undecodable(&mut self, channel: Token, task: u64, attempt: u64, name: u64) -> engine::Event {
        let call: Box<[u8]> = name.to_be_bytes().into();
        self.decode_answers.insert(call.clone(), (true, b"unknown tool".as_slice().into()));
        up(
            channel,
            host::LinkUp::Call {
                run: Token::new(task),
                attempt: Token::new(attempt),
                call,
                tool: b"unknown".as_slice().into(),
                writes: true,
                input: Box::new([]),
                deadline: Duration::from_secs(10),
            },
            engine::Parts::none(),
        )
    }

    /// A run's call answered successfully by the world's protocol at decode.
    pub fn answered_at_decode(
        &mut self,
        channel: Token,
        task: u64,
        attempt: u64,
        name: u64,
        text: &[u8],
    ) -> engine::Event {
        let event = self.undecodable(channel, task, attempt, name);
        self.decode_answers.insert(name.to_be_bytes().into(), (false, text.into()));
        event
    }

    /// A run's call of one engine tool, named `name`; its decoded body waits
    /// for the root's decode request.
    pub fn call(&mut self, channel: Token, task: u64, attempt: u64, name: u64, body: engine::Call) -> engine::Event {
        let call: Box<[u8]> = name.to_be_bytes().into();
        self.calls.insert(call.clone(), body);
        up(
            channel,
            host::LinkUp::Call {
                run: Token::new(task),
                attempt: Token::new(attempt),
                call,
                tool: b"engine".as_slice().into(),
                writes: true,
                input: Box::new([]),
                deadline: Duration::from_secs(10),
            },
            engine::Parts::none(),
        )
    }

    /// A run's answer: the hub carries its outcome as opaque bytes, which the
    /// world's protocol reads back as `end`; temper's parts ride beside it.
    pub fn answer(
        &mut self,
        channel: Token,
        task: u64,
        attempt: u64,
        cumulative: u64,
        end: tasks::End,
        parts: engine::Parts,
    ) -> engine::Event {
        self.serial += 1;
        let outcome: Box<[u8]> = self.serial.to_be_bytes().into();
        self.ends.insert(outcome.clone(), end);
        let answer = host::RunAnswer {
            read: None,
            refused: Box::new([]),
            turns: 0,
            spent: cumulative,
            ending: host::Ending::Ended { outcome, work: host::Work { left: None, saved: None } },
        };
        up(channel, host::LinkUp::Answer { run: Token::new(task), attempt: Token::new(attempt), answer }, parts)
    }

    /// Answer the oldest read through the worker link (jig/docs/design/domain/engine/settings.md, 12).
    pub fn answer_read(&mut self, outcome: jig_domain_link::ListOutcome) -> engine::Event {
        let (channel, read) = self.list_reads.pop_front().expect("a worker holds the list read");
        up(channel, host::LinkUp::ListRead { read, outcome }, engine::Parts::none())
    }

    /// A job's terminal arrives through the same worker link as a run's answer.
    pub fn answer_job(&mut self, channel: Token, task: u64, attempt: u64, answer: host::JobEnding) -> engine::Event {
        up(
            channel,
            host::LinkUp::JobAnswer { run: Token::new(task), attempt: Token::new(attempt), answer },
            engine::Parts::none(),
        )
    }

    /// Play one root request this layer answers; return any other to the caller.
    ///
    /// # Errors
    /// Returns the request when it is not the protocol layer's to answer.
    #[expect(clippy::too_many_lines, reason = "the protocol fixture routes each root request at one boundary")]
    pub fn play(&mut self, request: engine::Request) -> Result<Vec<engine::Event>, Box<engine::Request>> {
        match request {
            engine::Request::Party(engine::PartyRequest::Catalogue { models }) => {
                self.catalogue.clone_from(&models);
                let highest = models.iter().map(|entry| entry.number).max().unwrap_or(0);
                self.models = vec![engine::Model::hole(); usize::try_from(highest).expect("bounded catalogue")].into();
                for entry in &models {
                    let at = usize::try_from(entry.number - 1).expect("bounded model number");
                    self.models[at] = entry.model.clone();
                }
                self.catalogues.push(models);
                Ok(Vec::new())
            }
            engine::Request::Worker(engine::WorkerRequest::Deliver(engine::Delivery::Link {
                channel,
                down: host::LinkDown::Sources { sources: _ },
            })) => {
                self.sources.push(channel);
                // This scripted worker supplies an empty inventory, including for newly
                // declared sources: no login has been observed on it.
                Ok(vec![up(channel, host::LinkUp::Logins { logins: Box::new([]) }, engine::Parts::none())])
            }
            engine::Request::Worker(engine::WorkerRequest::Deliver(engine::Delivery::Link {
                channel,
                down: host::LinkDown::ReadList { read, .. },
            })) => {
                self.list_reads.push_back((channel, read));
                Ok(Vec::new())
            }
            engine::Request::Worker(engine::WorkerRequest::Host(host_request)) => match *host_request {
                engine::HostRequest::Decode { to, task, attempt, call } => match self.calls.get(&call.name) {
                    Some(body) => {
                        Ok(vec![engine::Event::Worker(engine::WorkerInput::DecodedHostCall { to, body: body.clone() })])
                    }
                    None if self.decode_answers.contains_key(&call.name) => {
                        let (error, body) = self.decode_answers.get(&call.name).expect("registered decode answer");
                        Ok(vec![engine::Event::Worker(engine::WorkerInput::RenderedHostCall {
                            to,
                            answer: core::SettledAnswer::Host { error: *error, body: body.clone() },
                        })])
                    }
                    None => Err(Box::new(engine::Request::Worker(engine::WorkerRequest::Host(Box::new(
                        engine::HostRequest::Decode { to, task, attempt, call },
                    ))))),
                },
                engine::HostRequest::SettingsNote { to, call, body } => {
                    Ok(vec![temper_engine_service::settings_note(to, *call, &body, &self.catalogue)])
                }
                engine::HostRequest::Dropped { .. } => Ok(Vec::new()),
            },
            engine::Request::Worker(engine::WorkerRequest::Deliver(engine::Delivery::Host(delivery))) => {
                match *delivery {
                    engine::HostDelivery::Render { to, name, answer, job, page, ask, .. } => {
                        if let engine::CallAnswer::Decided { .. } = &answer {
                            self.asks.insert(name.clone(), ask);
                        }
                        if let Some(job) = job {
                            self.read_jobs.insert(name.clone(), job);
                        }
                        if let Some(page) = page {
                            self.read_pages.insert(name.clone(), page);
                        }
                        self.answers.push((name.clone(), answer));
                        if self.defer_render {
                            self.pending_renders.push((to, name));
                            Ok(Vec::new())
                        } else {
                            Ok(vec![rendered(to, name)])
                        }
                    }
                    other @ (engine::HostDelivery::Settings { .. }
                    | engine::HostDelivery::See { .. }
                    | engine::HostDelivery::Answer { .. }) => Err(Box::new(engine::Request::Worker(
                        engine::WorkerRequest::Deliver(engine::Delivery::Host(Box::new(other))),
                    ))),
                }
            }
            engine::Request::Worker(engine::WorkerRequest::ReadResult { payload, outcome }) => {
                let end = self.ends.get(&outcome).expect("the world wrote this outcome").clone();
                Ok(vec![engine::Event::Worker(engine::WorkerInput::ResultRead { payload, end })])
            }
            engine::Request::Store(engine::StoreRequest::Read { load, range }) => {
                self.reads.push(*range);
                Ok(vec![engine::Event::Store(engine::StoreInput::ReadLoaded {
                    load,
                    result: core::reads::Loaded::Gone,
                })])
            }
            engine::Request::Account(request) => {
                let answer = self.account(&request);
                self.accounts.push(request);
                Ok(answer)
            }
            other @ (engine::Request::Store(_)
            | engine::Request::Worker(_)
            | engine::Request::Party(_)
            | engine::Request::Forge { .. }
            | engine::Request::ForgeKept(_)
            | engine::Request::ForgeGone { .. }
            | engine::Request::BoxGroup { .. }
            | engine::Request::PlaygroundActs { .. }
            | engine::Request::ResetAct { .. }
            | engine::Request::BoxRemove { .. }
            | engine::Request::Connect { .. }
            | engine::Request::Replace { .. }
            | engine::Request::Manifest { .. }
            | engine::Request::AdoptAccess { .. }
            | engine::Request::Reachable { .. }
            | engine::Request::Install { .. }
            | engine::Request::Accept { .. }
            | engine::Request::EraseCarried { .. }
            | engine::Request::MakingCopied { .. }
            | engine::Request::SetUp { .. }
            | engine::Request::Bound { .. }
            | engine::Request::Setup { .. }
            | engine::Request::ToChild(_)
            | engine::Request::Busy(_)
            | engine::Request::Unsettled(_)
            | engine::Request::Drained
            | engine::Request::Stop) => Err(Box::new(other)),
        }
    }

    /// The scripted account component's answer to one accounts request.
    fn account(&self, request: &accounts::Request) -> Vec<engine::Event> {
        match request {
            accounts::Request::Signing(accounts::Signing::Keep { source, .. }) => match self.key_failure {
                Some(why) => {
                    vec![engine::Event::Account(accounts::Event::Component(accounts::FromComponent::KeepFailed {
                        account: *source,
                        why,
                    }))]
                }
                None => vec![granted(*source, 1, false)],
            },
            accounts::Request::Component(accounts::ToComponent::Want { account }) => {
                vec![granted(*account, 1, self.sign_in_sources.contains(account))]
            }
            accounts::Request::Component(accounts::ToComponent::Rejected { account, generation }) => {
                vec![granted(*account, generation + 1, self.sign_in_sources.contains(account))]
            }
            accounts::Request::Component(accounts::ToComponent::Release { .. } | accounts::ToComponent::Close)
            | accounts::Request::Programs { .. }
            | accounts::Request::ReadList { .. }
            | accounts::Request::CheckProgram { .. }
            | accounts::Request::SignOutProgram { .. }
            | accounts::Request::Granted { .. }
            | accounts::Request::Save { .. }
            | accounts::Request::Erase { .. }
            | accounts::Request::Signing(_)
            | accounts::Request::Answer { .. }
            | accounts::Request::Closed => Vec::new(),
        }
    }
}

fn granted(account: u32, generation: u64, sign_in: bool) -> engine::Event {
    engine::Event::Account(accounts::Event::Component(accounts::FromComponent::Granted {
        form: Form::Credential(if sign_in { Scheme::Bearer } else { Scheme::Key }),
        account,
        generation,
        valid: Duration::from_secs(3600),
    }))
}

fn rendered(to: ReplyTo, name: Box<[u8]>) -> engine::Event {
    engine::Event::Worker(engine::WorkerInput::RenderedHostCall {
        to,
        answer: core::SettledAnswer::Host { error: false, body: name },
    })
}
