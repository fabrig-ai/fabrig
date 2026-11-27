//! One accepted stream and its opening (protocol/channel/worker-link.md, 1.3).
use crate::{Loss, Statement, component::conversion::Conversion};
use alloc::boxed::Box;
use jig_protocol_worker_wire::{Kind, layouts::Peer};
use skein_channel::{
    Event, Kind as ChannelKind, Lower, LowerEvent, Machine, Request, Role, Schema, StreamMode, WriteWait,
};
use skein_lib::{Duration, List, Queue, Time, Token};
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Phase {
    Opening,
    Terms,
    Hello,
    Open,
    Closing,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Ending {
    Active,
    Graceful,
    Abort,
}
#[derive(Debug)]
#[expect(clippy::struct_excessive_bools, reason = "independent terminal, log and hello lifecycle facts")]
pub(crate) struct Connection {
    pub machine: Machine,
    pub in_process: bool,
    pub pending_opening: Option<Event>,
    pub phase: Phase,
    pub opening: jig_protocol_worker_wire::Opening,
    pub worker: Option<u32>,
    pub channel: Token,
    pub peer: Option<Box<Peer>>,
    pub conversion: Conversion,
    pub table: Option<List<ChannelKind>>,
    pub converted: List<Box<str>>,
    pub statement: Option<Statement>,
    pub refusal: bool,
    pub deadline: Option<Time>,
    pub above: Queue<Event>,
    pub below: Queue<Lower>,
    pub terminal: bool,
    pub loss: Option<Loss>,
    pub ended: bool,
    pub heard: bool,
    pub hello_deadline: Option<Time>,
    pub ping: Option<Time>,
    pub silence: Option<Time>,
    pub stall: Option<Time>,
    pub ending: Ending,
}
impl Connection {
    pub fn new(
        schema: Schema,
        channel: skein_channel::Limits,
        handshake: Duration,
        now: Time,
        in_process: bool,
    ) -> Self {
        let machine = Machine::with_mode(schema, Role::Responder, channel, StreamMode::One).expect("checked schema");
        Self {
            machine,
            in_process,
            pending_opening: None,
            phase: Phase::Opening,
            opening: jig_protocol_worker_wire::Opening::Link,
            worker: None,
            channel: Token::new(0),
            peer: None,
            conversion: Conversion::new(channel.output_frames),
            table: None,
            converted: List::uncapped(),
            statement: None,
            refusal: false,
            deadline: Some(now.saturating_add(handshake)),
            above: Queue::uncapped(),
            below: Queue::uncapped(),
            terminal: false,
            loss: None,
            ended: false,
            heard: false,
            hello_deadline: None,
            ping: None,
            silence: None,
            stall: None,
            ending: Ending::Active,
        }
    }
    pub fn sends(&self, kind: Kind) -> Option<u16> {
        self.peer.as_ref()?.sends(kind)
    }

    pub fn omits(&self, kind: Kind) -> bool {
        kind.optional_at_worker() && self.sends(kind).is_none()
    }

    pub fn down(&mut self, request: Request, connection: Token, out: &mut Queue<crate::Request>) {
        self.machine.down(request, &mut self.above, &mut self.below);
        self.flush(connection, out);
    }
    pub fn up(&mut self, event: LowerEvent, connection: Token, out: &mut Queue<crate::Request>) {
        self.machine.up(event, &mut self.above, &mut self.below);
        self.flush(connection, out);
    }
    pub fn poll(&mut self, connection: Token, out: &mut Queue<crate::Request>) {
        self.machine.poll(&mut self.above, &mut self.below);
        self.flush(connection, out);
    }
    pub fn queued(&self) -> bool {
        match self.machine.waiting().write {
            WriteWait::Frames => true,
            WriteWait::Nothing => false,
        }
    }
    pub fn next_deadline(&self) -> Option<Time> {
        let next = earlier(self.deadline, self.silence);
        let next = earlier(next, self.stall);
        if self.queued() { next } else { earlier(next, self.ping) }
    }

    fn flush(&mut self, connection: Token, out: &mut Queue<crate::Request>) {
        for _ in 0..skein_channel::MAX_DOWN {
            let Some(lower) = self.below.pop() else { break };
            out.push(crate::Request::Lower { connection, lower });
        }
    }
}

fn earlier(left: Option<Time>, right: Option<Time>) -> Option<Time> {
    match left {
        Some(left) => match right {
            Some(right) => Some(left.min(right)),
            None => Some(left),
        },
        None => right,
    }
}
