//! Deterministic stream insertion and completion holding for the worker's orders.
//! The service still owns its IO tokens, framing, roots and process read demands.
use crate::processes::Wire;
use jig_protocol_worker_wire as wire;
use skein_io::kernel::{Complete, Done, Fd, Op, Submit};
use skein_lib::{Queue, Writer};
use std::collections::VecDeque;

// The marker names no run in the one-chat fixture.
const OTHER_RUN: u64 = 991;
const OTHER_ATTEMPT: u64 = 992;
// Smith's Call record, smith-channel's version-five kind table.
const PROCESS_CALL: u16 = 0x0107;
const HEADER: usize = 8;

#[derive(Default)]
struct Frames {
    bytes: Vec<u8>,
}
impl Frames {
    fn room(&self) -> usize {
        if self.bytes.len() < HEADER {
            return HEADER - self.bytes.len();
        }
        let head = skein_channel::parse_header(&self.bytes[..HEADER]).expect("frame header");
        HEADER + usize::try_from(head.body_len).expect("frame size") - self.bytes.len()
    }
    fn feed(&mut self, bytes: &[u8]) -> Vec<u16> {
        self.bytes.extend_from_slice(bytes);
        let mut kinds = Vec::new();
        while self.bytes.len() >= HEADER {
            let head = skein_channel::parse_header(&self.bytes[..HEADER]).expect("framed fixture stream");
            let end = HEADER + usize::try_from(head.body_len).expect("frame size");
            if self.bytes.len() < end {
                break;
            }
            kinds.push(head.kind);
            self.bytes.drain(..end);
        }
        kinds
    }
}

pub(crate) struct Probe {
    calls: bool,
    inserted: bool,
    native: Frames,
    pipe: Frames,
    pipe_fd: Option<Fd>,
    channel: Option<Fd>,
    pending: VecDeque<u8>,
    pipe_pending: VecDeque<u8>,
    held_call: Option<Complete>,
    call_stage: bool,
    marker_stage: bool,
    marker_queued: bool,
    marker_seen: bool,
    pub(crate) call_held: bool,
    pub(crate) marker_passed: bool,
}
impl Probe {
    pub(crate) fn new(calls: bool) -> Self {
        Self {
            calls,
            inserted: false,
            native: Frames::default(),
            pipe: Frames::default(),
            pipe_fd: None,
            channel: None,
            pending: VecDeque::new(),
            pipe_pending: VecDeque::new(),
            held_call: None,
            call_stage: false,
            marker_stage: false,
            marker_queued: false,
            marker_seen: false,
            call_held: false,
            marker_passed: false,
        }
    }
    pub(crate) fn waiting_for_cancel(&self) -> bool {
        self.held_call.is_some()
    }

    pub(crate) fn receive(&mut self, input: &mut Queue<Complete>, wire: &Wire) {
        self.channel = wire.channel;
        let mut ready = Queue::uncapped();
        for _ in 0..input.len() {
            let mut complete = input.pop().expect("completion count");
            if let Op::Spawn { spawn } = &complete.kind
                && matches!(complete.result, Ok(Done::Spawned { .. }))
            {
                self.pipe_fd = spawn.pipes.iter().find(|pipe| pipe.child == 1).and_then(|pipe| pipe.parent);
            }
            if self.calls
                && let Op::PipeRead { fd, buf } = &mut complete.kind
                && Some(*fd) == self.pipe_fd
                && let Ok(Done::Count(count)) = complete.result
            {
                let count = shape(&self.pipe, &mut self.pipe_pending, buf, usize::try_from(count).expect("pipe bytes"));
                complete.result = Ok(Done::Count(u32::try_from(count).expect("pipe count")));
                let kinds = self.pipe.feed(&buf[..count]);
                if !self.inserted && kinds.contains(&PROCESS_CALL) {
                    self.inserted = true;
                    self.held_call = Some(complete);
                    continue;
                }
            }
            if let Op::Recv { fd, buf } = &mut complete.kind
                && Some(*fd) == self.channel
                && let Ok(Done::Count(count)) = complete.result
            {
                let count = usize::try_from(count).expect("receive bytes");
                if !self.calls && !self.inserted && wire.hello_sent() && self.native.bytes.is_empty() {
                    self.inserted = true;
                    self.pending.extend(prefix());
                    self.marker_queued = true;
                    self.pending.extend(&buf[..count]);
                    let take = buf.len().min(self.pending.len()).min(self.native.room());
                    for byte in &mut buf[..take] {
                        *byte = self.pending.pop_front().expect("pending bytes");
                    }
                    complete.result = Ok(Done::Count(u32::try_from(take).expect("receive count")));
                }
                let Ok(Done::Count(delivered)) = complete.result else { unreachable!("count retained") };
                let delivered =
                    shape(&self.native, &mut self.pending, buf, usize::try_from(delivered).expect("delivered bytes"));
                complete.result = Ok(Done::Count(u32::try_from(delivered).expect("receive count")));
                for kind in self.native.feed(&buf[..delivered]) {
                    if self.calls
                        && kind == wire::Kind::Cancel.number()
                        && let Some(call) = self.held_call.take()
                    {
                        // The final Call bytes precede the final Cancel bytes in this service iteration.
                        ready.push(call);
                        self.call_stage = true;
                        self.pending.extend(marker());
                        self.marker_queued = true;
                    }
                    if kind == wire::Kind::TurnKept.number() && self.marker_queued && !self.marker_seen {
                        self.marker_seen = true;
                        self.marker_stage = true;
                    }
                }
            }
            ready.push(complete);
        }
        *input = ready;
    }
    pub(crate) fn submitted(&mut self, submit: Submit) -> Result<Submit, Complete> {
        if let Op::Recv { fd, mut buf } = submit.kind {
            if Some(fd) == self.channel && !self.pending.is_empty() {
                let take = buf.len().min(self.pending.len()).min(self.native.room());
                for byte in &mut buf[..take] {
                    *byte = self.pending.pop_front().expect("pending bytes");
                }
                return Err(Complete {
                    op: submit.op,
                    kind: Op::Recv { fd, buf },
                    result: Ok(Done::Count(u32::try_from(take).expect("receive count"))),
                });
            }
            return Ok(Submit { op: submit.op, kind: Op::Recv { fd, buf } });
        }
        if let Op::PipeRead { fd, mut buf } = submit.kind {
            if Some(fd) == self.pipe_fd && !self.pipe_pending.is_empty() {
                let take = buf.len().min(self.pipe_pending.len()).min(self.pipe.room());
                for byte in &mut buf[..take] {
                    *byte = self.pipe_pending.pop_front().expect("pending pipe bytes");
                }
                return Err(Complete {
                    op: submit.op,
                    kind: Op::PipeRead { fd, buf },
                    result: Ok(Done::Count(u32::try_from(take).expect("pipe count"))),
                });
            }
            return Ok(Submit { op: submit.op, kind: Op::PipeRead { fd, buf } });
        }
        Ok(submit)
    }
    pub(crate) fn stepped(&mut self, steps: u64, waits: u64) {
        // Raw kernel bytes may need another IO iteration before the root receives their record.
        if self.call_stage {
            assert_eq!(steps, 1, "only the admitted Cancel enters while the process Call waits");
            self.call_held = true;
            self.call_stage = false;
        }
        if self.marker_stage && steps > 0 {
            self.marker_passed = if self.calls { self.call_held && steps == 1 } else { waits > 0 };
            self.marker_stage = false;
        }
    }
}

// Keep a frame's header and body in separate IO deliveries, so the protocol's
// outstanding demand names the final body when its completion is held.
fn shape(frames: &Frames, pending: &mut VecDeque<u8>, buf: &mut [u8], count: usize) -> usize {
    if count <= frames.room() {
        return count;
    }
    pending.extend(&buf[..count]);
    let take = buf.len().min(pending.len()).min(frames.room());
    for byte in &mut buf[..take] {
        *byte = pending.pop_front().expect("pending stream bytes");
    }
    take
}

fn marker() -> Vec<u8> {
    let kept = wire::TurnKept::new(
        &wire::CEILINGS,
        wire::TurnKeptParts { task: OTHER_RUN, attempt: OTHER_ATTEMPT, number: 1 },
    )
    .expect("marker");
    let mut frame = skein_channel::frame_writer(wire::Kind::TurnKept.number(), kept.measure()).expect("marker frame");
    let mut body = Writer::new(usize::try_from(kept.measure()).expect("marker size"));
    kept.encode(&mut body).expect("marker body");
    frame.put(&body.finish()).expect("marker frame body");
    frame.finish().expect("marker frame size").bytes().to_vec()
}
fn prefix() -> Vec<u8> {
    let inbound = wire::Inbound::new(
        &wire::CEILINGS,
        wire::InboundParts {
            task: OTHER_RUN,
            attempt: OTHER_ATTEMPT,
            name: 1,
            sender: Box::default(),
            words: b"new".as_slice().into(),
            finish_requested: false,
        },
    )
    .expect("new message");
    let mut body = Writer::new(usize::try_from(inbound.measure()).expect("message size"));
    inbound.encode(&mut body).expect("message body");
    let mut frame =
        skein_channel::frame_writer(wire::Kind::Message.number(), inbound.measure()).expect("message frame");
    frame.put(&body.finish()).expect("message frame body");
    let mut bytes = frame.finish().expect("message frame size").bytes().to_vec();
    bytes.extend(marker());
    bytes
}
