//! One physical connection and its current call (llm-connection.md, sections
//! 4 and 5), over a link (skein/docs/design/link.md, 3). A connection keeps
//! its LLM client and bounded routing queues. It never knows the owner's
//! domain or retry policy. `route` advances the selected transport and the LLM client.
//!
//! | State | Input | Next | Emits |
//! | --- | --- | --- | --- |
//! | Connecting | connected | Handshaking | TLS handshake |
//! | Handshaking | ready | Calling | LLM request |
//! | Draining | drained, live | Idle | reusable binding |
//! | Draining | drained, owner closing | Ready | take a waiting call or close |
//! | any live | failure or close | Closing | one call terminal, socket close |
//! | Closing | socket closed | Closed | settlement |

use skein_io::Request as IoRequest;
use skein_lib::stream::{Down, Up};
use skein_lib::{Env, Queue, Time, Token};
use skein_link::{Event as LinkEvent, Limits as LinkLimits, Link, Moved, Start};
use skein_llm::client as llm;
use skein_tls::client as tls;

use crate::boundary::{Deadlines, Event};
use crate::deadlines::{Due, Phase as DeadlinePhase, Table};
use crate::limits::Limits;
use Phase::{Closing, Connecting, Draining, Handshaking, Head, Idle, Ready, Streaming};

/// The physical stream's lifecycle, independent of the call's terminal.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum Phase {
    Connecting { call: Option<Token> },
    Handshaking { call: Option<Token> },
    Head { call: Token },
    Streaming { call: Token },
    Draining,
    Idle,
    Ready,
    Closing { call: Option<Token> },
}

/// The owner's close policy and whether its physical abort has started.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Control {
    Live,
    Draining,
    AbortPending,
    Aborting,
}

/// One socket, TLS session and LLM client, named to io by its slab token.
pub(crate) struct Connection {
    pub(crate) endpoint: u32,
    pub(crate) reservation: u64,
    pub(crate) phase: Phase,
    pub(crate) link: Link,
    pub(crate) llm: llm::Client,
    llm_limits: llm::Limits,
    pub(crate) deadlines: Table,
    pub(crate) waiting: bool,
    pub(crate) activity: u64,
    control: Control,
    llm_events: Queue<llm::Event>,
    plain_down: Queue<Down>,
}

impl Connection {
    pub(crate) fn has_work(&self) -> bool {
        !self.llm_events.is_empty()
            || !self.plain_down.is_empty()
            || self.link.has_work()
            || (self.calling() && self.llm.has_work())
            || (self.abort_pending())
            || self.close_ready()
    }

    pub(crate) fn new(
        endpoint: u32,
        call: Token,
        tls: Option<tls::Client>,
        llm: llm::Client,
        llm_limits: llm::Limits,
        deadlines: Deadlines,
        now: Time,
    ) -> Connection {
        Connection {
            endpoint,
            reservation: 0,
            phase: Connecting { call: Some(call) },
            link: Link::new(tls),
            llm,
            llm_limits,
            deadlines: Table::new(deadlines, now),
            waiting: false,
            activity: 0,
            control: Control::Live,
            llm_events: Queue::uncapped(),
            plain_down: Queue::uncapped(),
        }
    }

    pub(crate) fn request_close(&mut self, abort: bool) {
        self.control = match self.control {
            Control::Live | Control::Draining => {
                if abort {
                    Control::AbortPending
                } else {
                    Control::Draining
                }
            }
            Control::AbortPending => Control::AbortPending,
            Control::Aborting => Control::Aborting,
        };
    }

    pub(crate) fn aborting(&self) -> bool {
        match self.control {
            Control::Live | Control::Draining => false,
            Control::AbortPending | Control::Aborting => true,
        }
    }

    fn closing(&self) -> bool {
        match self.control {
            Control::Live => false,
            Control::Draining | Control::AbortPending | Control::Aborting => true,
        }
    }

    fn abort_pending(&self) -> bool {
        match self.control {
            Control::AbortPending => true,
            Control::Live | Control::Draining | Control::Aborting => false,
        }
    }

    fn calling(&self) -> bool {
        match self.phase {
            Head { .. } | Streaming { .. } | Draining => true,
            Connecting { .. } | Handshaking { .. } | Idle | Ready | Closing { .. } => false,
        }
    }

    pub(crate) fn reusable(&self) -> bool {
        match self.phase {
            Idle | Ready => true,
            Connecting { .. } | Handshaking { .. } | Head { .. } | Streaming { .. } | Draining | Closing { .. } => {
                false
            }
        }
    }

    pub(crate) fn idle(&self) -> bool {
        self.phase == Idle
    }

    pub(crate) fn call(&self) -> Option<Token> {
        match self.phase {
            Connecting { call } | Handshaking { call } | Closing { call } => call,
            Head { call } | Streaming { call } => Some(call),
            Draining | Idle | Ready => None,
        }
    }

    fn close_ready(&self) -> bool {
        self.closing()
            && !self.waiting
            && match self.phase {
                Draining | Idle | Ready => true,
                Connecting { call } | Handshaking { call } => call.is_none(),
                Head { .. } | Streaming { .. } | Closing { .. } => false,
            }
    }

    fn terminal(&mut self) {
        self.phase = match self.phase {
            Head { .. } | Streaming { .. } => Draining,
            Connecting { .. } => Connecting { call: None },
            Handshaking { .. } => Handshaking { call: None },
            Closing { .. } => Closing { call: None },
            Draining | Idle | Ready => self.phase,
        };
    }

    pub(crate) fn sync_deadlines(&mut self, env: &Env<Limits>) {
        let phase = match self.phase {
            Connecting { .. } => DeadlinePhase::Connecting,
            Handshaking { .. } => DeadlinePhase::Handshaking,
            Head { .. } => DeadlinePhase::Head,
            Streaming { .. } => DeadlinePhase::Streaming,
            Draining => DeadlinePhase::Draining,
            Idle => DeadlinePhase::Idle,
            Ready | Closing { .. } => DeadlinePhase::Closing,
        };
        self.deadlines.arm(phase, env.now, env.limits.idle_keep);
    }

    pub(crate) fn connected(
        &mut self,
        env: &Env<Limits>,
        owner: Token,
        up: &mut Queue<Event>,
        io: &mut Queue<IoRequest>,
    ) {
        if self.aborting() {
            self.route(env, owner, up, io);
            return;
        }
        match self.link.connected(&link_env(env)) {
            Start::Handshaking => {
                self.phase = Handshaking { call: self.call() };
                self.route(env, owner, up, io);
            }
            Start::Ready => {
                self.phase = Head { call: self.call().expect("an open link starts its pending call") };
                self.sync_deadlines(env);
                self.start_call(env, owner, up, io);
            }
            Start::Closing => self.phase = Closing { call: self.call() },
        }
    }

    pub(crate) fn start_call(
        &mut self,
        env: &Env<Limits>,
        owner: Token,
        up: &mut Queue<Event>,
        io: &mut Queue<IoRequest>,
    ) {
        let llm_env = env.with(self.llm_limits);
        llm::down(&mut self.llm, &llm_env, llm::Request::Start, &mut self.llm_events, &mut self.plain_down);
        self.route(env, owner, up, io);
    }

    pub(crate) fn next(&mut self, env: &Env<Limits>, owner: Token, up: &mut Queue<Event>, io: &mut Queue<IoRequest>) {
        if self.aborting() {
            self.route(env, owner, up, io);
            return;
        }
        let llm_env = env.with(self.llm_limits);
        llm::down(&mut self.llm, &llm_env, llm::Request::Next, &mut self.llm_events, &mut self.plain_down);
        self.route(env, owner, up, io);
    }

    pub(crate) fn cancel(&mut self, env: &Env<Limits>, owner: Token, up: &mut Queue<Event>, io: &mut Queue<IoRequest>) {
        let llm_env = env.with(self.llm_limits);
        llm::down(&mut self.llm, &llm_env, llm::Request::Cancel, &mut self.llm_events, &mut self.plain_down);
        self.route(env, owner, up, io);
        match self.phase {
            Connecting { .. } => self.close_transport(env, true, io),
            Handshaking { .. } | Head { .. } | Streaming { .. } | Draining | Idle | Ready | Closing { .. } => {}
        }
    }

    pub(crate) fn failed(&mut self, env: &Env<Limits>, owner: Token, up: &mut Queue<Event>, io: &mut Queue<IoRequest>) {
        if self.aborting() {
            self.route(env, owner, up, io);
            return;
        }
        let llm_env = env.with(self.llm_limits);
        llm::abort(
            &mut self.llm,
            &llm_env,
            skein_llm::Failure::Unavailable,
            &mut self.llm_events,
            &mut self.plain_down,
        );
        self.route(env, owner, up, io);
    }

    pub(crate) fn cipher_up(
        &mut self,
        env: &Env<Limits>,
        owner: Token,
        event: Up,
        up: &mut Queue<Event>,
        io: &mut Queue<IoRequest>,
    ) {
        if self.aborting() {
            self.route(env, owner, up, io);
            return;
        }
        self.link.up(&link_env(env), event);
        self.route(env, owner, up, io);
    }

    fn plain_up(&mut self, env: &Env<Limits>, event: Up) {
        if self.call().is_some() && self.calling() {
            match &event {
                Up::Bytes(bytes) if !bytes.is_empty() => self.deadlines.response_bytes(env.now),
                Up::Bytes(_) | Up::End | Up::Room | Up::Failed(_) => {}
            }
        }
        let llm_env = env.with(self.llm_limits);
        llm::up(&mut self.llm, &llm_env, event, &mut self.llm_events, &mut self.plain_down);
    }

    fn sync_activity(&mut self, now: Time) {
        let activity = self.llm.activity();
        if activity != self.activity {
            let events = activity.checked_sub(self.activity).expect("call activity only increases");
            self.activity = activity;
            self.deadlines.activity(now, events);
        }
    }

    pub(crate) fn closed(&mut self, env: &Env<Limits>, owner: Token, up: &mut Queue<Event>, io: &mut Queue<IoRequest>) {
        let llm_env = env.with(self.llm_limits);
        self.link.closed();
        if self.abort_pending() {
            self.control = Control::Aborting;
            self.phase = Closing { call: self.call() };
            llm::down(&mut self.llm, &llm_env, llm::Request::Cancel, &mut self.llm_events, &mut self.plain_down);
        }
        llm::closed(&mut self.llm, &llm_env, &mut self.llm_events, &mut self.plain_down);
        self.route(env, owner, up, io);
        self.deadlines.arm(DeadlinePhase::Closed, env.now, env.limits.idle_keep);
    }

    fn close_transport(&mut self, env: &Env<Limits>, abort: bool, io: &mut Queue<IoRequest>) {
        self.link.close(&link_env(env), abort, io);
        self.phase = Closing { call: self.call() };
    }

    pub(crate) fn timeout(&mut self, env: &Env<Limits>, due: Due, io: &mut Queue<IoRequest>) {
        match self.phase {
            Draining | Idle => {
                self.close_transport(env, false, io);
                self.sync_deadlines(env);
                return;
            }
            Ready | Closing { .. } => return,
            Connecting { .. } | Handshaking { .. } | Head { .. } | Streaming { .. } => {}
        }
        let phase = match due {
            Due::Call(phase) => phase,
            Due::Keep => unreachable!("keep runs only on an idle connection"),
        };
        let llm_env = env.with(self.llm_limits);
        llm::abort(
            &mut self.llm,
            &llm_env,
            skein_llm::Failure::TimedOut { phase },
            &mut self.llm_events,
            &mut self.plain_down,
        );
    }

    #[expect(clippy::too_many_lines, reason = "the bounded routing pass covers the LLM client and link")]
    pub(crate) fn route(&mut self, env: &Env<Limits>, _owner: Token, up: &mut Queue<Event>, io: &mut Queue<IoRequest>) {
        let llm_env = env.with(self.llm_limits);
        if self.abort_pending() {
            self.control = Control::Aborting;
            llm::down(&mut self.llm, &llm_env, llm::Request::Cancel, &mut self.llm_events, &mut self.plain_down);
            self.close_transport(env, true, io);
        } else if self.close_ready() {
            llm::down(&mut self.llm, &llm_env, llm::Request::Close, &mut self.llm_events, &mut self.plain_down);
        }
        for _ in 0_u32..64_u32 {
            self.sync_activity(env.now);
            if let Some(event) = self.llm_events.pop() {
                match event {
                    llm::Event::Delta { owner: call, delta } => {
                        up.push(Event::Delta { call, delta });
                        continue;
                    }
                    llm::Event::Block { owner: call, block } => {
                        up.push(Event::Block { call, block });
                        continue;
                    }
                    llm::Event::Completed { owner: call, completion } => {
                        up.push(Event::Completed { call, completion, measures: self.deadlines.measures() });
                    }
                    llm::Event::Failed { owner: call, failure, evidence, report, detail } => {
                        up.push(Event::Failed {
                            call,
                            failure,
                            evidence,
                            report,
                            detail,
                            measures: self.deadlines.measures(),
                        });
                    }
                    llm::Event::Cancelled { owner: call } => {
                        up.push(Event::Cancelled { call, measures: self.deadlines.measures() });
                    }
                    llm::Event::Reusable => {
                        self.phase = if self.closing() { Ready } else { Idle };
                        continue;
                    }
                    llm::Event::Close => {
                        match self.phase {
                            Handshaking { .. } | Head { .. } | Streaming { .. } | Draining | Idle | Ready => {
                                self.close_transport(env, false, io);
                            }
                            Connecting { .. } => self.close_transport(env, true, io),
                            Closing { .. } => {}
                        }
                        continue;
                    }
                    llm::Event::Closed => continue,
                }
                self.terminal();
                if self.closing() && !self.waiting {
                    llm::down(&mut self.llm, &llm_env, llm::Request::Close, &mut self.llm_events, &mut self.plain_down);
                }
            } else if let Some(down) = self.plain_down.pop() {
                match self.phase {
                    Head { .. } => {
                        match &down {
                            Down::Send(bytes) if !bytes.is_empty() => {
                                self.deadlines.sent(env.now, self.llm.request_bytes());
                            }
                            Down::Send(_) | Down::Demand { .. } | Down::Finish => {}
                        }
                        self.link.down(&link_env(env), down);
                    }
                    Streaming { .. } | Draining | Idle | Ready => self.link.down(&link_env(env), down),
                    Connecting { .. } | Handshaking { .. } | Closing { .. } => {}
                }
            } else {
                match self.link.pump(io) {
                    Moved::Above(LinkEvent::Ready) => {
                        if !self.aborting() {
                            self.phase = Head { call: self.call().expect("an open link starts its pending call") };
                            self.sync_deadlines(env);
                            llm::down(
                                &mut self.llm,
                                &llm_env,
                                llm::Request::Start,
                                &mut self.llm_events,
                                &mut self.plain_down,
                            );
                        }
                    }
                    Moved::Above(LinkEvent::Stream(plain)) => {
                        if !self.aborting() {
                            self.plain_up(env, plain);
                        }
                    }
                    Moved::Above(LinkEvent::Failed) => llm::abort(
                        &mut self.llm,
                        &llm_env,
                        skein_llm::Failure::Unavailable,
                        &mut self.llm_events,
                        &mut self.plain_down,
                    ),
                    Moved::Sent | Moved::Within => {}
                    Moved::Nothing => {
                        if self.llm.has_work() && self.calling() {
                            llm::resume(&mut self.llm, &llm_env, &mut self.llm_events, &mut self.plain_down);
                        } else {
                            break;
                        }
                    }
                }
            }
        }
        match self.phase {
            Head { call } => self.phase = if self.llm.response_received() { Streaming { call } } else { Head { call } },
            Connecting { .. } | Handshaking { .. } | Streaming { .. } | Draining | Idle | Ready | Closing { .. } => {}
        }
        self.sync_activity(env.now);
        self.sync_deadlines(env);
    }
}

pub(crate) fn link_env(env: &Env<Limits>) -> Env<LinkLimits> {
    env.with(skein_link::Limits { tls: env.limits.tls, close_timeout: env.limits.io.close_timeout })
}
