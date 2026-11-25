//! A scripted host can stop reading without hanging up, or wake a waiting run.
//! The I/O observer timestamps progress and the first channel abort independently
//! of the agent's deadline and failure state (smith/docs/design/domain/host.md, section 4).

use super::World;
use skein_fake_channel::ScriptedPeer;
use skein_io::kernel::{Complete, Done, Fd, Op, Submit};
use skein_lib::{Duration, Time, Token};
use smith_channel::kind;

#[derive(Clone, Copy)]
pub(super) enum Plan {
    StopAfter(u16),
    Withhold,
    WakeAfter(Duration),
}

impl Plan {
    fn expects_failure(self) -> bool {
        matches!(self, Self::StopAfter(_) | Self::Withhold)
    }
}

pub(super) struct Host {
    plan: Plan,
    reviewed: usize,
    pub(super) last_taken: Time,
    pub(super) stopped: bool,
    pub(super) waiting_at: Option<Time>,
    pub(super) woke_at: Option<Time>,
    wake_at: Option<Time>,
}

impl Host {
    pub(super) fn new(plan: Plan) -> Self {
        Self {
            plan,
            reviewed: 0,
            last_taken: Time::ZERO,
            stopped: false,
            waiting_at: None,
            woke_at: None,
            wake_at: None,
        }
    }

    pub(super) fn step(&mut self, now: Time, peer: &mut ScriptedPeer, exited: bool) {
        for frame in &peer.observed()[self.reviewed..] {
            self.last_taken = now;
            match self.plan {
                Plan::StopAfter(kind) if frame.kind == kind => self.stopped = true,
                Plan::WakeAfter(delay) if frame.kind == kind::WAITING && self.waiting_at.is_none() => {
                    self.waiting_at = Some(now);
                    self.wake_at = Some(now.saturating_add(delay));
                }
                Plan::StopAfter(_) | Plan::Withhold | Plan::WakeAfter(_) => {}
            }
        }
        self.reviewed = peer.observed().len();
        if self.wake_at.is_some_and(|at| at <= now) {
            assert!(!exited, "a host that owes nothing can still send its next message");
            self.wake_at = None;
            self.woke_at = Some(now);
            peer.play(
                smith_channel::frame(smith_channel::Body::Message(smith_channel::Message {
                    finish_requested: true,
                    name: 18,
                    label: Box::from(*b"request"),
                    text: Box::from(*b"Now answer."),
                }))
                .expect("wake message"),
            );
        }
    }

    pub(super) fn expects_failure(&self) -> bool {
        self.plan.expects_failure()
    }

    pub(super) fn withholds(&self) -> bool {
        matches!(self.plan, Plan::Withhold)
    }

    pub(super) fn deadline(&self) -> Option<Time> {
        self.wake_at
    }
}

pub(crate) struct Watch {
    input: Fd,
    output: Fd,
    read: Option<Token>,
    write: Option<(Token, u16)>,
    last_progress: Time,
    now: Time,
    read_aborted: Option<Time>,
    write_aborted: Option<(Time, u16)>,
}

impl Watch {
    pub(crate) fn new(input: Fd, output: Fd) -> Self {
        Self {
            input,
            output,
            read: None,
            write: None,
            last_progress: Time::ZERO,
            now: Time::ZERO,
            read_aborted: None,
            write_aborted: None,
        }
    }

    pub(crate) fn clock(&mut self, now: Time) {
        self.now = now;
    }

    pub(crate) fn completed(&mut self, complete: &Complete) {
        match (&complete.kind, &complete.result) {
            (Op::PipeRead { fd, .. }, Ok(Done::Count(n))) if *fd == self.input && *n > 0 => {
                self.last_progress = self.now;
            }
            (Op::PipeWrite { fd, bytes, from }, Ok(Done::Count(n)))
                if *fd == self.output && usize::try_from(from + n).expect("write offset") == bytes.len() =>
            {
                self.last_progress = self.now;
            }
            _ => {}
        }
    }

    pub(crate) fn submitted(&mut self, submit: &Submit) {
        match submit.kind {
            Op::PipeRead { fd, .. } if fd == self.input => self.read = Some(submit.op),
            Op::PipeWrite { fd, ref bytes, .. } if fd == self.output => {
                let header = skein_channel::parse_header(&bytes[..8]).expect("frame handed to the pipe");
                self.write = Some((submit.op, header.kind));
            }
            Op::Cancel { target } if Some(target) == self.read => {
                self.read_aborted.get_or_insert(self.now);
            }
            Op::Cancel { target } => {
                if let Some((write, kind)) = self.write
                    && write == target
                {
                    self.write_aborted.get_or_insert((self.now, kind));
                }
            }
            _ => {}
        }
    }
}

impl World {
    /// Stop taking agent frames after the selected kind, keeping both ends open.
    pub fn stop_reading_after(&mut self, kind: u16) {
        self.silence = Some(Plan::StopAfter(kind));
        self.hold_channel = true;
        self.setup.watch_channel = true;
    }

    /// Keep reading, but send no acknowledgements or answers to calls.
    pub fn withhold_host_replies(&mut self) {
        self.silence = Some(Plan::Withhold);
        self.hold_channel = true;
        self.setup.watch_channel = true;
    }

    /// Read every frame and send a final message this long after Waiting.
    pub fn wake_waiting_after(&mut self, delay: Duration) {
        self.silence = Some(Plan::WakeAfter(delay));
    }

    pub(super) fn silence_expected(&self) -> bool {
        self.silence.is_some_and(Plan::expects_failure)
    }

    /// The last progress, verdict, and kind of any frame whose write it cut.
    #[must_use]
    pub fn silence_observations(&self) -> (Time, Time, Option<u16>) {
        let watch = self.observations().channel.as_ref().expect("watched channel");
        let host = self.parent().silence.as_ref().expect("scripted silent host");
        // After an answer, the ordinary shutdown already stopped reading;
        // the verdict is the abort of the write still in flight.
        let verdict =
            watch.write_aborted.map(|(at, _)| at).or(watch.read_aborted).expect("the agent aborted its own channel");
        (watch.last_progress.max(host.last_taken), verdict, watch.write_aborted.map(|(_, kind)| kind))
    }

    /// When the host observed Waiting and sent its next message, without seeing an exit.
    #[must_use]
    pub fn wake_observations(&self) -> (Time, Time) {
        let host = self.parent().silence.as_ref().expect("host scheduled to wake");
        (host.waiting_at.expect("run waited"), host.woke_at.expect("host sent a message"))
    }
}
