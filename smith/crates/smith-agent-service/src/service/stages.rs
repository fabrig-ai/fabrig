//! Shared LLM, machine, program and file composition for framed and inline owners.
//! It keeps their queues, routes, projected events and teardown, and never owns
//! a root domain, host channel or native IO. Borrowed framed consumers are explicit
//! at the passes. Contract: smith/docs/design/protocol/agent.md, sections 4–6.

use super::{
    Config, ConfigError, Failure, Frame, Parts, START_REPLY, Stages, Teardown, channel_event_ready,
    domain_entrance_ready, domain_request_credit, domain_requests_dispatched, face, llm_events_ready,
    pending_message_ready, program, sinks_take, submissions_ready, teardown,
};
use alloc::boxed::Box;
use skein_io::kernel;
use skein_lib::{Env, List, Map, Queue, ReplyTo, Slab, Time, Token, Wall, earliest};
use smith_domain::{self as domain, run};
use smith_protocol_llm as llm;
use smith_protocol_machine as machine;

pub(super) fn clocks(service: &mut Stages, now: Time, wall: Wall) {
    service.domain_env.now = now;
    service.domain_env.wall = wall;
    service.llm_env.now = now;
    service.llm_env.wall = wall;
    service.machine_env.now = now;
    service.machine_env.wall = wall;
    service.io_env.now = now;
    service.io_env.wall = wall;
    service.file_env.now = now;
    service.file_env.wall = wall;
}

/// Earliest active deadline over all child machines.
#[must_use]
pub(super) fn next_deadline(service: &Stages, frame: Option<&Frame>) -> Option<Time> {
    let mut next = match face::domain(frame) {
        Some(domain) => domain.next_deadline(),
        None => None,
    };
    next = earliest(next, service.llm.next_deadline());
    next = earliest(next, program::deadline(service));
    next = earliest(next, service.machine.next_deadline());
    next = earliest(
        next,
        match face::io(frame) {
            Some(io) => io.next_deadline(),
            None => None,
        },
    );
    next = earliest(next, service.files.next_deadline());
    next = earliest(next, service.live.deadline());
    next
}

/// Earliest policy deadline, excluding IO and file settlement.
#[must_use]
pub(super) fn next_policy_deadline(service: &Stages, frame: Option<&Frame>) -> Option<Time> {
    let mut next = match face::domain(frame) {
        Some(domain) => domain.next_deadline(),
        None => None,
    };
    next = earliest(next, service.llm.next_deadline());
    next = earliest(next, program::policy_deadline(service));
    next = earliest(next, service.machine.next_deadline());
    next = earliest(next, service.live.deadline());
    next
}

// Count internal handoffs, including the optional program's lower queue.
pub(super) fn backlog(service: &Stages) -> u64 {
    let mut total = u64::from(program::backlog(service));
    for entries in [
        service.llm_events.len(),
        service.llm_below.len(),
        service.machine_events.len(),
        service.machine_below.len(),
        service.domain_events.len(),
        service.lower_events.len(),
        service.domain_requests.len(),
        service.io_events.len(),
        service.io_requests.len(),
        service.file_events.len(),
        service.io_submissions.len(),
        service.file_submissions.len(),
    ] {
        total = total.checked_add(u64::from(entries)).expect("bounded queue counts fit u64");
    }
    total
}

/// Whether another iteration can progress without waiting on the kernel.
#[must_use]
pub(super) fn work_pending(service: &Stages, frame: Option<&Frame>, now: Time) -> bool {
    let takes = sinks_take(service);
    let due = match next_deadline(service, frame) {
        Some(deadline) => deadline <= now,
        None => false,
    };
    due || teardown::work_pending(service, frame)
        || match face::io(frame) {
            Some(io) => io.is_ready(),
            None => false,
        }
        || (takes && (service.llm.has_work() || !service.llm.measures_drained()))
        || match &service.events {
            Some(events) => events.has_work(),
            None => false,
        }
        || match face::domain(frame) {
            Some(domain) => takes && (domain.has_facts() || (domain_requests_dispatched(service) && domain.is_ready())),
            None => false,
        }
        || !service.completions.is_empty()
        || !service.io_completions.is_empty()
        || !service.io_events.is_empty()
        || !service.file_events.is_empty()
        || channel_event_ready(service)
        || match face::channel(frame) {
            Some(channel) => channel.has_work() && super::channel_room(service, frame),
            None => false,
        }
        || (!service.pending_messages.is_empty() && pending_message_ready(service))
        || (llm_events_ready(service, frame) && !service.llm_events.is_empty())
        || !service.machine_events.is_empty()
        || !service.lower_events.is_empty()
        || (takes && domain_entrance_ready(service, frame) && !service.domain_events.is_empty())
        || (domain_request_credit(service, frame)
            && !service.domain_requests.is_empty()
            && program::requests_ready(service))
        || !service.channel_below.is_empty()
        || program::pending(service)
        || !service.llm_below.is_empty()
        || !service.machine_below.is_empty()
        || !service.io_requests.is_empty()
        || ((!service.io_submissions.is_empty() || !service.file_submissions.is_empty()) && submissions_ready(service))
}

/// The channel has ended, every lower child has settled, and the final
/// answer crossed the channel.
#[must_use]
pub(super) fn done(service: &Stages) -> Option<bool> {
    match service.teardown {
        Teardown::Closed { answer, .. } => Some(answer.sent()),
        Teardown::Live { .. } | Teardown::Answered { .. } | Teardown::Closing { .. } => None,
    }
}

/// The first concrete failure, retained for the shell's final stderr line.
#[must_use]
pub(super) const fn failure(service: &Stages) -> Option<Failure> {
    service.teardown.failure()
}

impl Stages {
    pub(super) fn parts(config: Config) -> Result<Parts, ConfigError> {
        let events = crate::event_producer(&config);
        let Config { mut limits, domain, channel_endpoints, llm_endpoints, environment, stream_mode, program } = config;
        let program::Parts { program, files, file_limits } = program::parts(program, &mut limits, &environment)?;
        if limits.routes > 1 << 28_u32 {
            return Err(ConfigError::Queue);
        }
        let frames = limits.queue;
        let llm = match llm::Component::new(&limits.llm, llm_endpoints) {
            Ok(llm) => llm,
            Err(error) => return Err(ConfigError::Llm(error)),
        };
        let machine = machine::Component::new(&limits.machine, environment);
        Ok(Parts {
            stages: Stages {
                dumps: None,
                limits,
                events,
                teardown: Teardown::Live { failure: None },
                product_exit: None,
                usage_unread: false,
                live: crate::live::Live::new(None, limits.live_interval),
                llm,
                machine,
                program,
                files,
                domain_env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.domain },
                llm_env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.llm },
                machine_env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.machine },
                io_env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.io },
                file_env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: file_limits },
                channel_events: Queue::with_capacity(frames),
                pending_messages: Queue::with_capacity(frames),
                channel_below: Queue::with_capacity(frames),
                llm_events: Queue::uncapped(),
                llm_below: Queue::uncapped(),
                machine_events: Queue::uncapped(),
                machine_below: Queue::uncapped(),
                domain_events: Queue::uncapped(),
                lower_events: Queue::uncapped(),
                domain_requests: Queue::uncapped(),
                io_events: Queue::uncapped(),
                io_requests: Queue::uncapped(),
                file_events: Queue::uncapped(),
                io_submissions: Queue::uncapped(),
                file_submissions: Queue::uncapped(),
                submissions: Queue::with_capacity(limits.queue),
                completions: Queue::with_capacity(limits.queue),
                io_completions: Queue::uncapped(),
                owners: Slab::with_capacity(limits.routes),
                in_flight: 0,
                signal_reads: 0,
                submitted: 1,
                next_send: 1,
                input: None,
                output: None,
                signals: None,
                admission: super::admission::Admission::Waiting,
                run_started: Time::ZERO,
                line: crate::terminal::Last::Quiet,
                opening: super::opening::Opening::Idle,
                pipe_routes: Map::with_capacity(limits.machine.processes.checked_mul(2).ok_or(ConfigError::Limits)?),
                roots: super::roots::Roots::Empty,
            },
            domain,
            channel_endpoints,
            stream_mode,
        })
    }

    pub(super) fn next_send(&mut self) -> Token {
        super::link::token(&mut self.next_send)
    }

    pub(super) fn mark_failed(&mut self, reason: Failure) {
        teardown::failed(self, reason);
    }

    pub(super) fn refuse_start(&mut self, frame: &mut Frame, reason: run::Invalid) {
        self.opening.refuse();
        let token = self.next_send();
        let answer = run::Answer::Refused(run::Refusal::Invalid(reason));
        crate::terminal::remember(self, &answer, Box::default(), self.domain_env.now);
        let read = None;
        if let Some(events) = &mut self.events {
            events.answer(self.domain_env.now, self.run_started, &answer, read);
        }
        let result = frame.channel.send_answer(
            self.domain_env.now,
            answer,
            read,
            token,
            &mut self.channel_events,
            &mut self.channel_below,
        );
        super::link::sent(self, result.is_ok());
        teardown::answered(self);
    }

    pub(super) fn start_ready(&mut self, frame: &mut Frame) {
        let (start, directories) = self.opening.take();
        let workspace = match start.mounts {
            Some(_) => Some(run::Workspace { directories: directories.into_boxed() }),
            None => None,
        };
        if let Some(ref workspace) = workspace {
            self.machine.workspace(workspace);
        }
        if self.llm.bind_grants(&start.charter).is_err() {
            self.refuse_start(frame, run::Invalid::Grants);
            return;
        }
        let mut grants = List::with_capacity(u32::try_from(start.grants.len()).expect("bounded grant count"));
        for grant in &start.grants {
            let value = match frame.channel.grant_value(grant.name) {
                Some(value) => value.clone(),
                None => {
                    self.refuse_start(frame, run::Invalid::Grants);
                    return;
                }
            };
            let lapses = self.domain_env.now.saturating_add(grant.valid);
            if program::install(self, &start, grant.name, &value, lapses, workspace.as_ref()).is_err() {
                self.refuse_start(frame, run::Invalid::Grants);
                return;
            }
            grants
                .push(domain::Grant { name: grant.name, valid: grant.valid, renewable: grant.renewable })
                .expect("grant count was checked at Start");
        }
        let identity = match start.identity {
            Some(identity) => Some(identity.bytes()),
            None => None,
        };
        super::dump::started(self, identity, start.activation, &start.charter);
        self.run_started = self.domain_env.now;
        self.opening.contract(start.charter.outcome.clone(), start.charter.grants.deliver.clone());
        self.domain_events.push(domain::Event::Start(domain::Start {
            messages: start.messages,
            reply_to: ReplyTo::new(START_REPLY),
            host_run: START_REPLY,
            activation: start.activation,
            identity: start.identity,
            window: start.window,
            charter: start.charter,
            workspace,
            transcript: start.transcript,
            answered: start.answered,
            grants: grants.into_boxed(),
        }));
    }

    pub(super) fn completions(&mut self) -> &mut Queue<kernel::Complete> {
        &mut self.completions
    }
    pub(super) fn submissions(&mut self) -> &mut Queue<kernel::Submit> {
        &mut self.submissions
    }
    pub(super) fn start_session(&mut self, now: Time, header: smith_events::SessionStarted) {
        if let Some(events) = &mut self.events {
            events.started(now, header);
        }
    }
}
