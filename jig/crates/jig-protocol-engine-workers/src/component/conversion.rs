//! Holding values for a written converter (jig/docs/design/protocol/channel/layouts.md, 6 and 7).
use super::{Component, Connection, Phase, Sending};
use crate::{Bound, Loss, Request};
use alloc::boxed::Box;
use jig_protocol_worker_wire::{
    self as wire,
    convert::{self, Advance, Converting, Waiting},
};
use skein_channel::{Frame, LowerEvent, Request as ChannelRequest};
use skein_codec::layout::Refusal;
use skein_lib::{Queue, Time, Token, stream};

#[derive(Debug)]
pub(crate) struct Conversion {
    held: Option<Held>,
    sends: Queue<Prepared>,
    pub(crate) read: Option<stream::Up>,
}

impl Conversion {
    pub(crate) fn new(frames: u32) -> Self {
        Self { held: None, sends: Queue::with_capacity(frames), read: None }
    }

    pub(super) fn waiting(&self) -> bool {
        self.held.is_some()
    }

    pub(super) fn clear(&mut self) -> bool {
        self.sends.clear();
        self.read = None;
        self.held.take().is_some()
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Route {
    Send(Sending),
    Figures { bytes: u32 },
    Read(u16),
}

#[derive(Debug)]
struct Prepared {
    frame: Frame,
    route: Route,
}

#[derive(Debug)]
struct Held {
    frame: Box<Waiting>,
    route: Route,
}

impl Component {
    /// Lends the oldest waiting value. A lost channel no longer has a value to lend.
    #[must_use]
    pub fn converting(&self) -> Option<Converting<'_>> {
        let connection = self.conversions.iter().next()?;
        let state = self.connections.get(connection)?;
        let held = state.conversion.held.as_ref()?;
        Some(held.frame.converting(state.peer.as_ref().expect("a written pass has a peer")))
    }

    /// Takes the owner's answer to the oldest request, whether or not its channel still exists.
    pub fn converted(&mut self, now: Time, answer: Result<Box<[u8]>, Refusal>, out: &mut Queue<Request>) {
        let Some(&connection) = self.conversions.iter().next() else { return };
        if let Some(mut state) = self.connections.remove(&connection) {
            if let Some(held) = state.conversion.held.take() {
                let peer = state.peer.as_mut().expect("a written pass has a peer");
                let result = held.frame.answered(peer, answer, &self.codec);
                self.advance(now, connection, &mut state, result, held.route, out);
                self.resume_channel(now, connection, &mut state, out);
                self.progress(now, connection, &mut state, out);
            }
            self.keep(connection, state);
        }
        // Keep the head reserved until the answer settles, so new requests
        // (including the next pass of this frame) go behind earlier requests.
        let _answered = self.conversions.pop();
        if !self.conversions.is_empty() {
            out.push(Request::Convert);
        }
    }

    fn resume_channel(&mut self, now: Time, connection: Token, state: &mut Connection, out: &mut Queue<Request>) {
        for _ in 0..state.conversion.sends.len() {
            if state.conversion.waiting() || state.terminal || state.phase == Phase::Closing {
                return;
            }
            let ready = state.conversion.sends.pop().expect("counted sends");
            self.offer(now, connection, state, ready.frame, ready.route, out);
        }
        if !state.conversion.waiting()
            && !state.terminal
            && state.phase != Phase::Closing
            && let Some(event) = state.conversion.read.take()
        {
            state.up(LowerEvent::Read(event), connection, out);
        }
    }

    // A queued conversion for a lost channel needs no answer, except for
    // the head already announced to the owner, whose answer is still due.
    pub(super) fn forget_conversion(&mut self, connection: Token) {
        let length = self.conversions.len();
        for index in 0..length {
            let queued = self.conversions.pop().expect("counted conversions");
            if index == 0 || queued != connection {
                self.conversions.push(queued);
            }
        }
    }

    pub(super) fn offer(
        &mut self,
        now: Time,
        connection: Token,
        state: &mut Connection,
        frame: Frame,
        route: Route,
        out: &mut Queue<Request>,
    ) {
        if state.conversion.waiting() {
            // Untranslated records are counted against the same frame room as
            // admitted records. The full-output policy is to lose the channel.
            if state.conversion.sends.len() >= state.machine.room().frames {
                self.loss(now, connection, state, Loss::Full, out);
            } else {
                state.conversion.sends.push(Prepared { frame, route });
            }
            return;
        }
        let result = match &mut state.peer {
            Some(peer) => convert::start_send(peer, frame, &self.codec),
            None => Advance::Frame(frame),
        };
        self.advance(now, connection, state, result, route, out);
    }

    pub(super) fn advance(
        &mut self,
        now: Time,
        connection: Token,
        state: &mut Connection,
        result: Advance,
        route: Route,
        out: &mut Queue<Request>,
    ) {
        match result {
            Advance::Waiting(frame) => {
                assert!(!state.conversion.waiting(), "one held value per channel");
                state.conversion.held = Some(Held { frame, route });
                if self.conversions.is_empty() {
                    out.push(Request::Convert);
                }
                self.conversions.push(connection);
            }
            Advance::Frame(frame) => match route {
                Route::Read(_) => self.decoded_body(
                    now,
                    connection,
                    state,
                    frame.kind(),
                    frame.bytes().get(8..).expect("frame header"),
                    out,
                ),
                Route::Send(sending) => self.send_frame(now, connection, state, frame, sending, out),
                Route::Figures { .. } => self.figures_frame(now, connection, state, frame, out),
            },
            Advance::Refused(problem) => match route {
                Route::Read(source) => {
                    let worker = state.worker.expect("a body from an authenticated channel");
                    let peer = state.peer.as_ref().expect("a converted read has a peer");
                    let kind = convert::received_kind(peer, source).expect("the admitted table maps this worker kind");
                    out.push(Request::Unread { worker, kind, problem });
                    self.refuse(now, connection, state, 3, Box::new([]), out);
                }
                Route::Send(sending) => self.unsent(now, state, sending, Bound::Translation(problem), out),
                Route::Figures { bytes } => {
                    let sending = Sending { kind: wire::Kind::Figures, bytes, reply: super::Reply::Drop };
                    self.unsent(now, state, sending, Bound::Translation(problem), out);
                    if state.phase == Phase::Terms {
                        self.refuse(now, connection, state, 3, Box::new([]), out);
                    }
                }
            },
        }
    }

    fn send_frame(
        &mut self,
        now: Time,
        connection: Token,
        state: &mut Connection,
        frame: Frame,
        mut sending: Sending,
        out: &mut Queue<Request>,
    ) {
        sending.bytes = frame.wire_len();
        if sending.bytes > self.limits.room {
            self.unsent(now, state, sending, Bound::Room(self.limits.room), out);
            return;
        }
        self.sending = Some(sending);
        state.down(ChannelRequest::Send { token: state.channel, frame }, connection, out);
        self.progress(now, connection, state, out);
        assert!(self.sending.is_none(), "a channel send answers within its entrance");
    }
}
