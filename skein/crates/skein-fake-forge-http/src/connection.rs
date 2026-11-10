//! One connection's core (skein/docs/design/testing/fakes/github.md, 3).
use alloc::boxed::Box;
use skein_http::{Header, server as http};
use skein_lib::{
    Env, Queue, bytes,
    stream::{Down, Read, Up},
    usize_of,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum State {
    New,
    Reading,
    Receiving,
    Body,
    Waiting,
    Streaming,
    Closing,
    Closed,
}

#[expect(missing_debug_implementations, reason = "HTTP queues temporarily hold authorization headers")]
#[expect(
    clippy::partial_pub_fields,
    reason = "the connection exposes the server pass fields and keeps head and reply ownership private"
)]
pub struct Connection {
    pub state: State,
    pub events: Queue<http::Event>,
    pub requests: Queue<http::Request>,
    pub http: http::Server,
    head: Option<http::Call>,
    reply: Option<Box<[u8]>>,
    offset: usize,
}

/// One HTTP event, as the server has to answer it.
#[derive(PartialEq, Eq, Debug)]
pub enum Turn {
    /// The core has kept the head's snapshot, Authorization blanked.
    Call(http::Call),
    Body(Up),
    Done(http::Reuse),
    Failed,
    Handled,
}

#[derive(Debug)]
pub struct Reply {
    pub status: u16,
    pub headers: Box<[Header]>,
    pub body: Box<[u8]>,
    pub bodyless: bool,
}

impl Connection {
    #[must_use]
    pub fn new(limits: &http::Limits) -> Self {
        Self {
            state: State::New,
            events: Queue::uncapped(),
            requests: Queue::uncapped(),
            http: http::Server::new(limits),
            head: None,
            reply: None,
            offset: 0,
        }
    }
    pub fn take_head(&mut self) -> Option<http::Call> {
        self.head.take()
    }
    #[must_use]
    pub fn head_pending(&self) -> bool {
        self.head.is_some()
    }
    #[must_use]
    pub fn is_settled(&self) -> bool {
        self.state == State::Reading
    }
    #[must_use]
    pub fn has_work(&self) -> bool {
        !self.events.is_empty() || !self.requests.is_empty()
    }
}

/// Hands the event to HTTP unless the connection's state refuses it.
pub fn up(c: &mut Connection, env: &Env<http::Limits>, event: Up, below: &mut Queue<Down>) -> bool {
    match c.state {
        State::New | State::Closing | State::Closed => return false,
        State::Reading | State::Receiving | State::Body | State::Waiting | State::Streaming => {}
    }
    if c.state == State::Reading
        && let Up::Bytes(_) = &event
    {
        c.state = State::Receiving;
    }
    http::up(&mut c.http, env, event, &mut c.events, below);
    true
}

/// Pops one HTTP event and answers as far as the core can.
pub fn turn(c: &mut Connection, limits: &http::Limits) -> Option<Turn> {
    let event = c.events.pop()?;
    Some(match event {
        http::Event::Call(call) => {
            assert!(c.head.is_none(), "the owner drains each captured head before reuse");
            let mut snapshot = call.clone();
            for header in &mut snapshot.headers {
                if header.is(b"authorization") {
                    header.value = Box::new([]);
                }
            }
            c.head = Some(snapshot);
            Turn::Call(call)
        }
        http::Event::Body(up) => Turn::Body(up),
        http::Event::Done(reuse) => Turn::Done(reuse),
        http::Event::Reply(Up::Room) => {
            let reply = c.reply.as_ref().expect("reply body");
            let end = c.offset.saturating_add(usize_of(limits.send)).min(reply.len());
            let chunk = bytes::copy_of(reply.get(c.offset..end).expect("bounded reply slice"));
            c.offset = end;
            c.requests.push(http::Request::Reply(Down::Send(chunk)));
            if end == reply.len() {
                c.reply = None;
                c.requests.push(http::Request::Reply(Down::Finish));
            } else {
                room(c, limits);
            }
            Turn::Handled
        }
        http::Event::Reply(Up::Bytes(_) | Up::End | Up::Failed(_))
        | http::Event::Refused(_)
        | http::Event::Failed(_)
        | http::Event::Ended => Turn::Failed,
        http::Event::Closed => Turn::Handled,
    })
}

/// Hands HTTP the oldest queued request; false when there is none.
pub fn request(c: &mut Connection, env: &Env<http::Limits>, below: &mut Queue<Down>) -> bool {
    let Some(request) = c.requests.pop() else {
        return false;
    };
    down(c, env, request, below);
    true
}

pub fn down(c: &mut Connection, env: &Env<http::Limits>, request: http::Request, below: &mut Queue<Down>) {
    http::down(&mut c.http, env, request, &mut c.events, below);
}

/// Enters Reading and asks HTTP for the next request.
pub fn read_next(c: &mut Connection) {
    c.state = State::Reading;
    c.requests.push(http::Request::Next);
}

pub fn respond(c: &mut Connection, limits: &http::Limits, reply: Reply) {
    c.state = State::Streaming;
    c.offset = 0;
    let length = skein_lib::len_u64(reply.body.len());
    c.reply = if length == 0 { None } else { Some(reply.body) };
    c.requests.push(http::Request::Respond(http::Response {
        status: reply.status,
        headers: reply.headers,
        body: if reply.bodyless { http::Body::None } else { http::Body::Length(length) },
        close: false,
    }));
    if length == 0 && !reply.bodyless {
        c.requests.push(http::Request::Reply(Down::Finish));
    } else if length > 0 {
        room(c, limits);
    }
}

fn room(c: &mut Connection, limits: &http::Limits) {
    let remaining = c.reply.as_ref().expect("reply body").len().saturating_sub(c.offset);
    c.requests.push(http::Request::Reply(Down::Demand {
        read: Read::Nothing,
        room: u32::try_from(remaining).unwrap_or(u32::MAX).min(limits.send),
    }));
}

/// Drops the reply and both queues, closes HTTP and enters Closing.
pub fn close(c: &mut Connection, env: &Env<http::Limits>, below: &mut Queue<Down>) {
    if c.state == State::Closing || c.state == State::Closed {
        return;
    }
    c.reply = None;
    c.events.clear();
    c.requests.clear();
    down(c, env, http::Request::Close, below);
    c.state = State::Closing;
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
