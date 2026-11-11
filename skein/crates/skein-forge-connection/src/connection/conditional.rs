//! Keeping wire answers and replaying their body stream (forge/connection.md, 4.6).
use super::{Connection, Moved, Phase, http};
use crate::{
    Limits,
    grow::Grow,
    kept::{Identity, Key, Store, Validators},
};
use alloc::boxed::Box;
use skein_lib::{
    Env, List, Secret,
    stream::{Down, Read, Up},
    usize_of,
};

pub(super) struct Conditional {
    key: Option<Key>,
    pin: Option<u64>,
    capture: Option<Box<Capture>>,
    replay: Option<Box<Replay>>,
}
struct Capture {
    head: http::Response,
    validators: Validators,
    body: Grow,
    ended: bool,
    reserved: u64,
    cap: u32,
    reserved_capacity: u32,
}
struct Replay {
    id: u64,
    at: usize,
    read: Option<Read>,
    ended: bool,
    reuse: Option<http::Reuse>,
}
impl Conditional {
    fn empty() -> Self {
        Self { key: None, pin: None, capture: None, replay: None }
    }
    pub(super) fn new(c: &Connection, call: &mut http::Call, kept: &mut Store) -> Self {
        if call.method != skein_http::Method::Get {
            return Self::empty();
        }
        let mut range = None;
        let mut token = Secret::new(Box::from([]));
        let mut headers =
            List::with_capacity(u32::try_from(call.headers.len()).expect("request fields").saturating_add(2));
        for header in core::mem::replace(&mut call.headers, Box::from([])) {
            if header.is(b"Range") {
                range = Some(header.value.clone());
            }
            if header.is(b"Authorization") {
                token = Secret::new(header.value.clone());
            }
            // The store is the only source, including the delivery log's first page.
            if !header.is(b"If-None-Match") && !header.is(b"If-Modified-Since") {
                headers.push(header).expect("request header room");
            }
        }
        let identity = match c.identity {
            Some(installation) => Identity::Installation(installation),
            None => Identity::Token(token),
        };
        let key = Key { forge: c.forge, origin: c.origin, identity, target: call.target.clone(), range };
        let pin = kept.pin(&key, &mut headers);
        call.headers = headers.into_boxed();
        Self { key: Some(key), pin, capture: None, replay: None }
    }
}
impl Connection {
    pub(crate) fn kept_pin(&self) -> Option<u64> {
        match &self.state {
            super::Sending { exchange, .. } | super::Reading { exchange, .. } => exchange.conditional.pin,
            super::Closed { .. } | super::Idle { .. } | super::Connecting { .. } | super::Handshaking { .. } => None,
        }
    }
    pub(crate) fn kept_room(&self) -> u64 {
        match &self.state {
            super::Sending { exchange, .. } | super::Reading { exchange, .. } => match &exchange.conditional.capture {
                Some(capture) => capture.reserved,
                None => 0,
            },
            super::Closed { .. } | super::Idle { .. } | super::Connecting { .. } | super::Handshaking { .. } => 0,
        }
    }
    pub(crate) fn forget_kept(&mut self) {
        if let Some(exchange) = self.state.exchange_mut() {
            exchange.conditional = Conditional::empty();
        }
    }
    pub(crate) fn cancel_capture(&mut self) {
        if let Some(exchange) = self.state.exchange_mut() {
            exchange.conditional.capture = None;
        }
    }
    pub(crate) fn take_free(&mut self) -> u32 {
        core::mem::replace(&mut self.reused, 0)
    }
    pub(super) fn conditional_head(
        &mut self,
        env: &Env<Limits>,
        response: http::Response,
        kept: &mut Store,
    ) -> http::Response {
        let conditional = &mut self.exchange().conditional;
        if response.status == 304
            && let Some(id) = conditional.pin
            && let Some(head) = kept.replay(id, &response)
        {
            // End the wire's bodyless exchange before serving the local body stream.
            self.http_down(env, http::Request::Discard);
            self.exchange().conditional.replay =
                Some(Box::new(Replay { id, at: 0, read: None, ended: false, reuse: None }));
            self.reused = self.reused.checked_add(1).expect("bounded exchanges");
            return head;
        }
        if (response.status == 200 || response.status == 206)
            && let Some(key) = &conditional.key
        {
            kept.drop_key(key);
            if let Some(id) = conditional.pin.take() {
                kept.release(id);
            }
            if let Some(validators) = Validators::of(&response)
                && let Some(room) = Store::body_room(key, &response, &validators, env.limits.kept_bytes)
            {
                let (cap, reserved_capacity) = match response.framing {
                    http::Framing::Length(length) if length > u64::from(room) => return response,
                    http::Framing::Length(length) => {
                        let length = u32::try_from(length).expect("body fits the copy room");
                        (length, length)
                    }
                    http::Framing::Empty => (0, 0),
                    http::Framing::Chunked | http::Framing::UntilEnd => (room, 0),
                };
                let reserved =
                    Store::overhead(key, &response, &validators).saturating_add(u64::from(reserved_capacity));
                if kept.reserve(reserved) {
                    conditional.capture = Some(Box::new(Capture {
                        head: response.clone(),
                        validators,
                        body: Grow::new(0, cap),
                        ended: false,
                        reserved,
                        cap,
                        reserved_capacity,
                    }));
                }
            }
        }
        response
    }
    pub(super) fn capture(&mut self, event: &Up, kept: &mut Store) {
        let Some(exchange) = self.state.exchange_mut() else { return };
        let Some(capture) = &mut exchange.conditional.capture else { return };
        match event {
            Up::Bytes(bytes) => {
                let need = u64::from(capture.body.len()).saturating_add(skein_lib::len_u64(bytes.len()));
                let capacity = capture.body.capacity();
                let grown = if need > u64::from(capacity) {
                    u32::try_from(need).unwrap_or(u32::MAX).max(capacity.saturating_mul(2)).min(capture.cap)
                } else {
                    capacity
                };
                let extra = u64::from(grown.saturating_sub(capture.reserved_capacity));
                if need > u64::from(capture.cap) || !kept.reserve(extra) {
                    kept.unreserve(capture.reserved);
                    exchange.conditional.capture = None;
                } else {
                    capture.reserved = capture.reserved.checked_add(extra).expect("bounded copy");
                    capture.reserved_capacity = capture.reserved_capacity.max(grown);
                    capture.body.write(bytes).expect("copy room reserved");
                }
            }
            Up::End => capture.ended = true,
            Up::Failed(_) => {
                kept.unreserve(capture.reserved);
                exchange.conditional.capture = None;
            }
            Up::Room => unreachable!("response bodies only read"),
        }
    }
    pub(super) fn end_kept(&mut self, kept: &mut Store) {
        let conditional = core::mem::replace(&mut self.exchange().conditional, Conditional::empty());
        if let Some(id) = conditional.pin {
            kept.release(id);
        }
        if let Some(capture) = conditional.capture {
            kept.unreserve(capture.reserved);
            if capture.ended {
                kept.keep(
                    conditional.key.expect("a captured GET has a key"),
                    capture.head,
                    capture.validators,
                    capture.body.finish(),
                );
            }
        }
    }
    /// Local demands are answered in the same bounded pump, never sent to HTTP's finished 304.
    pub(super) fn replay_demand(&mut self, request: &http::Request) -> bool {
        let Some(exchange) = self.state.exchange_mut() else { return false };
        let Some(replay) = &mut exchange.conditional.replay else { return false };
        match request {
            http::Request::Body(Down::Demand { read, .. }) => {
                replay.read = if *read == Read::Nothing { None } else { Some(*read) };
                true
            }
            http::Request::Discard => {
                replay.ended = true;
                true
            }
            http::Request::Body(Down::Send(_) | Down::Finish) => unreachable!("response bodies only read"),
            http::Request::Call(_) | http::Request::Upload(_) | http::Request::Close => false,
        }
    }
    pub(super) fn replay_done(&mut self, reuse: http::Reuse) -> bool {
        let Some(replay) = &mut self.exchange().conditional.replay else { return false };
        replay.reuse = Some(reuse);
        true
    }
    pub(super) fn replay_ready(&self) -> bool {
        match &self.state {
            super::Sending { exchange, .. } | super::Reading { exchange, .. } => match &exchange.conditional.replay {
                Some(replay) => replay.read.is_some() || (replay.ended && replay.reuse.is_some()),
                None => false,
            },
            super::Closed { .. } | super::Idle { .. } | super::Connecting { .. } | super::Handshaking { .. } => false,
        }
    }
    pub(super) fn pump_replay(&mut self, env: &Env<Limits>, kept: &mut Store) -> Option<Moved> {
        let replay = self.exchange_if_reading()?.conditional.replay.as_mut()?;
        let reuse = replay.reuse?;
        if replay.ended {
            // The tokenizer's queued movements run before this one, just as with a wire body.
            self.exchange().conditional.replay = None;
            return Some(self.http_event(env, http::Event::Done(reuse), kept));
        }
        let read = replay.read.take()?;
        let Some(body) = kept.body(replay.id) else {
            self.failed(super::Failure::Seen);
            return Some(Moved::Within);
        };
        let left = body.get(replay.at..).expect("replay position in kept body");
        let event = if left.is_empty() {
            replay.ended = true;
            Up::End
        } else {
            let max = match read {
                Read::Nothing => unreachable!("a replay read demands bytes"),
                Read::Fill(n) | Read::Arrived { max: n } | Read::Scan { max: n, .. } | Read::Line { max: n } => n,
            };
            let count = left.len().min(usize_of(max));
            let bytes = Box::from(left.get(..count).expect("bounded replay piece"));
            replay.at = replay.at.checked_add(count).expect("position in kept body");
            Up::Bytes(bytes)
        };
        Some(self.http_event(env, http::Event::Body(event), kept))
    }
    fn exchange_if_reading(&mut self) -> Option<&mut super::Exchange> {
        if self.phase() != Phase::Reading && !self.in_flight() {
            return None;
        }
        self.state.exchange_mut()
    }
}
