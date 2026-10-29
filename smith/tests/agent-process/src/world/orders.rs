//! Output-pressure controls and observations for the entrance orders
//! (skein/docs/design/draft/dynamic-memory.md, 4.6). It retains only kernel
//! completions and wire records, never the service's domain state.

use super::Proc;
use skein_io::kernel::{Complete, Op, Submit};
use skein_lib::{Duration, Time, Token, Writer};

/// The real output whose completion the process adapter withholds.
#[derive(Clone, Copy, Debug)]
pub enum Order {
    /// Hold host admission output until later turn output and a Message queue.
    Requests,
    /// Hold trace output while two later Messages arrive.
    Facts,
}

/// Kernel and trace observations made while output remains blocked.
#[derive(Default, Debug)]
pub struct OrderSnapshot {
    /// The held kernel completion was reached before the Messages were sent.
    pub output_held: bool,
    /// Message frames fully written by the parent before releasing output.
    pub messages_written: usize,
    /// Milliseconds from the simulated session origin when output is released.
    pub released_at_ms: u64,
    /// Native trace records submitted before releasing output.
    pub records: Vec<smith_events::Record>,
}

pub(crate) struct Pressure {
    order: Order,
    pub(crate) held: Option<Complete>,
    pub(crate) released: bool,
    pub(crate) snapshot: OrderSnapshot,
    bytes: Vec<u8>,
    response_started: Option<Token>,
}

impl Pressure {
    fn new(order: Order) -> Self {
        Self {
            order,
            held: None,
            released: false,
            snapshot: OrderSnapshot::default(),
            bytes: Vec::new(),
            response_started: None,
        }
    }

    pub(crate) fn holds(&self, complete: &Complete) -> bool {
        if self.released || self.held.is_some() {
            return false;
        }
        match self.order {
            Order::Requests => matches!(&complete.kind, Op::PipeWrite { bytes, .. } if bytes.get(..2) == Some(&[1, 6])),
            Order::Facts => self.response_started == Some(complete.op),
        }
    }

    pub(crate) fn observe(&mut self, submission: &Submit) {
        if self.released {
            return;
        }
        if let Op::Append { bytes, from, .. } = &submission.kind {
            self.bytes.extend_from_slice(&bytes[usize::try_from(*from).expect("append offset")..]);
            if self.bytes.last() == Some(&b'\n') {
                let limits = crate::tracing::event_limits(&crate::limits());
                let records = crate::events::read(&self.bytes, &limits);
                if records.iter().any(|record| matches!(record.event, smith_events::Event::ResponseStarted(_))) {
                    self.response_started = Some(submission.op);
                }
                self.snapshot.records.extend(records);
                self.bytes.clear();
            }
        }
    }
}

// Leave time for both input frames to cross the simulated pipes, then release
// well before the trace's one-second write deadline.
const INPUT_DELAY: Duration = Duration::from_millis(10);
const OUTPUT_DELAY: Duration = Duration::from_millis(20);

// One application frame and one queued IO send make later output wait behind
// the active admission write and live text piece in the request-order story.
pub(super) const OUTPUT_FRAMES: u32 = 1;
pub(super) const OUTPUT_SENDS: u32 = 1;
const REQUEST_MESSAGES: usize = 1;
const FACT_MESSAGES: usize = 2;

pub(super) struct Referee {
    order: Order,
    at: Option<Time>,
    sent: usize,
    released: bool,
}

impl Referee {
    pub(super) fn new(order: Order) -> Self {
        Self { order, at: None, sent: 0, released: false }
    }

    pub(super) fn deadline(&self) -> Option<Time> {
        if self.released { None } else { self.at }
    }

    pub(super) fn act(&mut self, now: Time, procs: &mut [Proc]) {
        let trace = procs.iter_mut().find_map(|proc| match proc {
            Proc::Tracing(trace) => Some(trace),
            Proc::Parent(_) | Proc::Agent(..) | Proc::Peer(..) => None,
        });
        let Some(trace) = trace else { return };
        let pressure = trace.pressure.get_or_insert_with(|| Pressure::new(self.order));
        if pressure.held.is_some() && self.at.is_none() && !self.released {
            pressure.snapshot.output_held = true;
            self.at = Some(now.saturating_add(INPUT_DELAY));
        }
        if self.released || self.at.is_none_or(|at| at > now) {
            return;
        }
        let count = match self.order {
            Order::Requests => REQUEST_MESSAGES,
            Order::Facts => FACT_MESSAGES,
        };
        if self.sent < count {
            let parent = procs
                .iter_mut()
                .find_map(|proc| match proc {
                    Proc::Parent(parent) => Some(parent),
                    Proc::Tracing(_) | Proc::Agent(..) | Proc::Peer(..) => None,
                })
                .expect("host parent");
            parent.program_messages.push_back(message(self.sent));
            parent.program_message = true;
            self.sent += 1;
            self.at = Some(now.saturating_add(OUTPUT_DELAY));
        } else {
            let written = procs
                .iter()
                .find_map(|proc| match proc {
                    Proc::Parent(parent) => Some(parent.order_messages_written),
                    Proc::Tracing(_) | Proc::Agent(..) | Proc::Peer(..) => None,
                })
                .expect("host parent");
            let trace = procs
                .iter_mut()
                .find_map(|proc| match proc {
                    Proc::Tracing(trace) => Some(trace),
                    Proc::Parent(_) | Proc::Agent(..) | Proc::Peer(..) => None,
                })
                .expect("trace adapter");
            let pressure = trace.pressure.as_mut().expect("pressure installed");
            pressure.snapshot.released_at_ms = now.as_nanos() / 1_000_000;
            pressure.snapshot.messages_written = written;
            pressure.released = true;
            self.released = true;
            self.at = None;
        }
    }
}

fn message(index: usize) -> skein_channel::Frame {
    let message = smith_channel::Message::new(
        &smith_channel::CEILINGS,
        smith_channel::MessageParts {
            finish_requested: false,
            name: u64::try_from(index + 1).expect("two named messages"),
            label: b"host".as_slice().into(),
            text: b"hello".as_slice().into(),
        },
    )
    .expect("bounded message");
    let mut body = Writer::new(usize::try_from(message.measure()).expect("message size"));
    message.encode(&mut body).expect("message body");
    let mut frame = skein_channel::frame_writer(0x0101, message.measure()).expect("message frame");
    frame.put(&body.finish()).expect("message bytes");
    frame.finish().expect("whole message frame")
}
