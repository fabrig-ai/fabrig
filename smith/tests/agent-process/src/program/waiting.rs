//! Host messages released after Waiting or inside a held program turn (smith/docs/design/protocol/programs.md 5.3, 5.9).
//! Keeps a script cursor and release deadline; observes wire frames and fake
//! state, never agent internals. `observe` schedules; `act` only sets the
//! parent's existing release flag, leaving heap ownership inside its calls.
use super::{Proc, Time};
use skein_io::kernel::Spawn;
use skein_lib::{Duration, Writer};

pub(super) fn timeout(spawn: &Spawn) {
    if spawn.args.first().is_some_and(|argument| argument.as_ref() == b"-p")
        && spawn.env.iter().any(|entry| entry.as_ref() == b"WAIT_PROOF=1")
    {
        assert!(spawn.env.iter().any(|entry| entry.as_ref() == b"MCP_TOOL_TIMEOUT=60000"), "run time in milliseconds");
        assert_eq!(spawn.env.iter().filter(|entry| entry.starts_with(b"MCP_TOOL_TIMEOUT=")).count(), 1);
    }
}

/// When the parent releases a message, observed between world iterations.
#[derive(Clone, Copy, Debug)]
pub enum When {
    /// After this one-based Waiting frame from the run.
    Waiting(u32),
    /// While the fake holds this one-based program turn on a call or hang.
    Turn(u32),
}

/// A host message released by the shared waiting referee.
#[derive(Clone, Debug)]
pub struct HostMessage {
    pub when: When,
    pub text: String,
    pub finish_requested: bool,
}

pub(super) struct Release {
    messages: Vec<HostMessage>,
    next: usize,
    held: bool,
    at: Option<Time>,
}

impl Release {
    pub(super) fn new(enabled: bool) -> Self {
        Self {
            messages: if enabled {
                vec![HostMessage {
                    when: When::Waiting(1),
                    text: "The next instruction".into(),
                    finish_requested: false,
                }]
            } else {
                Vec::new()
            },
            next: 0,
            held: true,
            at: None,
        }
    }

    pub(super) fn open(messages: &[HostMessage]) -> Self {
        Self { messages: messages.to_vec(), next: 0, held: false, at: None }
    }

    pub(super) fn deadline(&self) -> Option<Time> {
        self.at
    }

    pub(super) fn passed(&self) -> bool {
        self.next == self.messages.len()
    }

    pub(super) fn observe(&mut self, now: Time, procs: &[Proc]) {
        let Some(message) = self.messages.get(self.next) else { return };
        if self.at.is_some() {
            return;
        }
        let Proc::Parent(parent) = &procs[0] else { panic!("parent first") };
        let arrived = match message.when {
            When::Waiting(count) => {
                parent.peer.observed().iter().filter(|frame| frame.kind == 0x010a).count()
                    >= usize::try_from(count).expect("waiting count fits")
            }
            When::Turn(turn) => {
                procs.iter().any(|process| matches!(process, Proc::Fake(fake) if fake.active_turn() == Some(turn)))
            }
        };
        if arrived
            && (!self.held
                || procs
                    .iter()
                    .any(|process| matches!(process, Proc::Fake(fake) if fake.pending_tool() == Some("wait"))))
        {
            // Preserve the held-wait proof's delay; Open's short idle fixture
            // releases a message before its ten-millisecond parking interval.
            let delay = if self.held { Duration::from_millis(50) } else { Duration::from_millis(1) };
            self.at = Some(now.saturating_add(delay));
        }
    }

    pub(super) fn act(&mut self, now: Time, procs: &mut [Proc]) {
        if self.at.is_none_or(|at| at > now) {
            return;
        }
        if self.held {
            assert!(
                procs.iter().any(|process| matches!(process, Proc::Fake(fake) if fake.pending_tool() == Some("wait"))),
                "the MCP wait stays held before the host message"
            );
        }
        let Proc::Parent(parent) = &mut procs[0] else { panic!("parent first") };
        parent.program_message = true;
        self.next += 1;
        self.at = None;
    }
}

pub(super) fn initial(enabled: bool) -> Option<smith_channel::Message> {
    enabled.then(|| {
        smith_channel::Message::new(
            &smith_channel::CEILINGS,
            smith_channel::MessageParts {
                finish_requested: false,
                name: 0,
                label: b"host".as_slice().into(),
                text: b"Begin".as_slice().into(),
            },
        )
        .expect("initial message")
    })
}

pub(crate) fn message() -> skein_channel::Frame {
    message_frame(1, "The next instruction", false)
}

pub(super) fn messages(releases: &[HostMessage]) -> std::collections::VecDeque<skein_channel::Frame> {
    releases
        .iter()
        .enumerate()
        .map(|(index, message)| {
            message_frame(u32::try_from(index + 1).expect("message name"), &message.text, message.finish_requested)
        })
        .collect()
}

fn message_frame(name: u32, text: &str, finish_requested: bool) -> skein_channel::Frame {
    let message = smith_channel::Message::new(
        &smith_channel::CEILINGS,
        smith_channel::MessageParts {
            finish_requested,
            name: u64::from(name),
            label: b"host".as_slice().into(),
            text: text.as_bytes().into(),
        },
    )
    .expect("host message");
    let mut body = Writer::new(message.measure() as usize);
    message.encode(&mut body).expect("message body");
    let mut frame = skein_channel::frame_writer(0x0101, message.measure()).expect("message frame");
    frame.put(&body.finish()).expect("message bytes");
    frame.finish().expect("host message frame")
}
