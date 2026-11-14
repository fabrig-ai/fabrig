//! The reference's assignments and root events around the workers' host (service.md, 11.3).
use crate::{Files, tools, values::Values};
use alloc::boxed::Box;
use jig_domain_host as host;
use jig_protocol_config::{self as config, Problems};
use jig_protocol_engine_workers as kit;
use jig_test_domain as root;
use skein_io as io;
use skein_lib::{List, Queue, Time, Token};
#[derive(Debug)]
pub(super) struct Workers {
    pub(super) host: kit::host::Workers,
    deployment: [u8; 16],
    endpoints: List<jig_protocol_charter::EndpointName>,
}
impl Workers {
    pub fn new(
        config: &config::Engine,
        files: &Files,
        deployment: [u8; 16],
        limits: &kit::Limits,
    ) -> Result<Self, Problems> {
        let tokens = kit::host::Tokens {
            listeners: crate::tokens::WORKER_LISTENERS,
            connections: crate::tokens::WORKER_CONNECTIONS,
        };
        let host = kit::host::Workers::new(
            config,
            files,
            tokens,
            jig_protocol_worker_wire::Application::opaque(List::with_capacity(0)),
            limits,
        )?;
        let mut endpoints = List::with_capacity(u32::try_from(config.llm.endpoints.len()).expect("endpoint count"));
        for endpoint in &config.llm.endpoints {
            endpoints
                .push(jig_protocol_charter::EndpointName {
                    number: endpoint.number,
                    name: endpoint.name.clone(),
                    dialect: endpoint.dialect,
                })
                .expect("endpoint count");
        }
        Ok(Self { host, deployment, endpoints })
    }
    pub fn down(&mut self, now: Time, event: kit::Event, events: &mut Queue<root::Event>) {
        self.host.down(now, event);
        self.pull(events);
    }
    pub fn assignment(
        &mut self,
        now: Time,
        channel: Token,
        value: root::Assignment,
        values: &Values,
        events: &mut Queue<root::Event>,
    ) {
        let (task, attempt) = (value.task, value.attempt);
        match self.encoded_assignment(now, channel, value, values) {
            Some(event) => self.down(now, event, events),
            None => events.push(root::Event::Answer {
                channel,
                task,
                attempt,
                cumulative: 0,
                read: None,
                refused: Box::default(),
                end: jig_domain_tasks::End::Failed(jig_domain_tasks::Class::Invalid),
            }),
        }
    }
    pub fn encoded_assignment(
        &self,
        now: Time,
        channel: Token,
        value: root::Assignment,
        values: &Values,
    ) -> Option<kit::Event> {
        Some(delivery(
            now,
            channel,
            host::LinkDown::Assign { assignment: tools::charter(value, self.endpoints.as_slice(), &self.deployment)? },
            values,
        ))
    }
    pub fn send(
        &mut self,
        now: Time,
        channel: Token,
        down: host::LinkDown,
        values: &Values,
        events: &mut Queue<root::Event>,
    ) {
        self.down(now, delivery(now, channel, down, values), events);
    }
    pub fn up(&mut self, now: Time, event: io::Event, events: &mut Queue<root::Event>) -> bool {
        let seen = self.host.up(now, event);
        self.pull(events);
        match seen {
            kit::host::Seen::Taken | kit::host::Seen::ListenerClosed { .. } => true,
            kit::host::Seen::Unknown => false,
        }
    }
    pub fn iterate(&mut self, now: Time, events: &mut Queue<root::Event>) {
        self.host.iterate(now);
        self.pull(events);
    }
    pub fn abort(&mut self, now: Time, events: &mut Queue<root::Event>) {
        self.host.abort(now);
        self.pull(events);
    }
    pub fn close(&mut self, now: Time, events: &mut Queue<root::Event>) {
        self.host.close(now);
        self.pull(events);
    }
    fn pull(&mut self, events: &mut Queue<root::Event>) {
        for _ in 0..self.host.pending() {
            match self.host.output() {
                Some(kit::host::Output::Hello { channel, worker, hello }) => {
                    events.push(root::Event::Link { channel, link: root::link::Event::Hello { worker, hello } });
                }
                Some(kit::host::Output::Up { channel, up, parts }) => {
                    drop(parts);
                    events.push(root::Event::Link { channel, link: root::link::Event::Up(up) });
                }
                Some(kit::host::Output::Lost { channel }) => {
                    events.push(root::Event::Link { channel, link: root::link::Event::Lost });
                }
                Some(kit::host::Output::Unsent { channel, unsent, .. }) => match unsent.bound {
                    kit::Bound::Room(_) | kit::Bound::Channel(_) => {}
                    kit::Bound::Translation { .. } => match unsent.kind {
                        jig_protocol_worker_wire::Kind::Assignment | jig_protocol_worker_wire::Kind::Job => {}
                        jig_protocol_worker_wire::Kind::Hello
                        | jig_protocol_worker_wire::Kind::Turn
                        | jig_protocol_worker_wire::Kind::Call
                        | jig_protocol_worker_wire::Kind::Answer
                        | jig_protocol_worker_wire::Kind::Fact
                        | jig_protocol_worker_wire::Kind::Unauthorized
                        | jig_protocol_worker_wire::Kind::Limited
                        | jig_protocol_worker_wire::Kind::AccountRefused
                        | jig_protocol_worker_wire::Kind::Message
                        | jig_protocol_worker_wire::Kind::Relayed
                        | jig_protocol_worker_wire::Kind::TurnKept
                        | jig_protocol_worker_wire::Kind::TurnBusy
                        | jig_protocol_worker_wire::Kind::Acknowledge
                        | jig_protocol_worker_wire::Kind::Cancel
                        | jig_protocol_worker_wire::Kind::Grant
                        | jig_protocol_worker_wire::Kind::Sources
                        | jig_protocol_worker_wire::Kind::Check
                        | jig_protocol_worker_wire::Kind::ReadList
                        | jig_protocol_worker_wire::Kind::ListRead
                        | jig_protocol_worker_wire::Kind::Draining
                        | jig_protocol_worker_wire::Kind::SignOut
                        | jig_protocol_worker_wire::Kind::Logins
                        | jig_protocol_worker_wire::Kind::SignedOut
                        | jig_protocol_worker_wire::Kind::JobAnswer
                        | jig_protocol_worker_wire::Kind::Ask
                        | jig_protocol_worker_wire::Kind::Asked
                        | jig_protocol_worker_wire::Kind::Wires => events.push(root::Event::Core(
                            jig_domain_core::Event::Fleet(jig_domain_fleet::Event::Mismatch { channel }),
                        )),
                    },
                },
                Some(
                    kit::host::Output::Opened { .. }
                    | kit::host::Output::Refused { .. }
                    | kit::host::Output::Ended { .. },
                )
                | None => {}
            }
        }
    }
}

fn delivery(now: Time, channel: Token, down: host::LinkDown, values: &Values) -> kit::Event {
    let values = values.grants(kit::host::grants(&down), now);
    kit::Event::Down { channel, down, parts: Box::default(), values }
}
