//! One streaming, authenticated webhook connection (forge/connection.md, 5 and 8).
use super::Hinted;
use crate::{Event, Limits, component::entry::ForgeEntry, limits};
use alloc::boxed::Box;
use skein_forge::api;
use skein_forge::forgejo::plans::hint;
use skein_forge::forgejo::{
    client::delivery as decode,
    delivery::{self, Delivery},
};
use skein_http::{Method, server as http};
use skein_json::tokenizer::{self as json, Tokenizer};
use skein_lib::stream::{Down, Read, Up};
use skein_lib::{
    Env, Queue, Time, Token,
    bytes::hex_digit,
    sha256::{self, Hmac},
};

// skein/docs/design/forge/connection.md, 8.4: each phase owns its data and clock.
#[derive(Debug)]
#[expect(clippy::large_enum_variant, reason = "the bounded connection holds its streaming decoder inline")]
enum State {
    Head {
        until: Time,
        held: Hint,
    },
    Body(Body),
    Answering {
        until: Time,
        held: Hint,
        /// The hint is offered before the close is sent.
        next: Option<http::Reuse>,
    },
    Closing(Closing),
}
#[derive(Debug)]
enum Closing {
    Pending { until: Time },
    Sent,
    Aborted,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum HintState {
    Awaited,
    Ready,
    Refused,
}
// The offer survives an unhinted answer and the following head and body.
#[derive(Debug)]
struct Hint {
    value: Option<api::Hint>,
    offer: HintState,
}
#[derive(Debug)]
struct Body {
    until: Time,
    held: Hint,
    mac: Hmac,
    signature: [u8; 64],
    forge: u16,
    decoder: Option<decode::Decoder>,
    decoded: Option<Delivery>,
    malformed: bool,
    ended: bool,
}
#[derive(Debug)]
pub struct Hooked {
    socket: Token,
    pub(super) deadline: Time,
    http: http::Server,
    events: Queue<http::Event>,
    below: Queue<Down>,
    json: Tokenizer,
    json_events: Queue<json::Event>,
    json_down: Queue<Down>,
    state: State,
}
impl Hooked {
    #[cfg(test)]
    pub fn new(socket: Token, env: &Env<Limits>) -> Self {
        Self::accepted(socket, env.now, env)
    }
    pub fn accepted(socket: Token, accepted: Time, env: &Env<Limits>) -> Self {
        let mut c = Self {
            socket,
            deadline: accepted,
            http: http::Server::new(&limits::hook_http(&env.limits)),
            events: Queue::uncapped(),
            below: Queue::uncapped(),
            json: Tokenizer::new(&limits::hook_documents(&env.limits).json),
            json_events: Queue::uncapped(),
            json_down: Queue::uncapped(),
            state: State::Head {
                until: accepted.saturating_add(env.limits.hook_deadline),
                held: Hint { value: None, offer: HintState::Awaited },
            },
        };
        c.down(env, http::Request::Next);
        c.follow();
        c
    }
    fn follow(&mut self) {
        self.deadline = match &self.state {
            State::Head { until, .. }
            | State::Body(Body { until, .. })
            | State::Answering { until, .. }
            | State::Closing(Closing::Pending { until }) => *until,
            State::Closing(Closing::Sent | Closing::Aborted) => self.deadline,
        };
    }
    pub fn sent(&self) -> bool {
        match &self.state {
            State::Head { .. } | State::Body(_) | State::Answering { .. } | State::Closing(Closing::Pending { .. }) => {
                false
            }
            State::Closing(Closing::Sent | Closing::Aborted) => true,
        }
    }
    pub fn fault(&mut self) {
        self.state = State::Closing(Closing::Pending { until: self.deadline });
        self.follow();
    }
    pub fn abort(&mut self, io: &mut Queue<skein_io::Request>) {
        match self.state {
            State::Closing(Closing::Aborted) => return,
            State::Head { .. }
            | State::Body(_)
            | State::Answering { .. }
            | State::Closing(Closing::Pending { .. } | Closing::Sent) => {}
        }
        self.state = State::Closing(Closing::Aborted);
        self.follow();
        io.push(skein_io::Request::Abort { entity: self.socket });
    }
    fn down(&mut self, env: &Env<Limits>, request: http::Request) {
        let env = env.with(limits::hook_http(&env.limits));
        http::down(&mut self.http, &env, request, &mut self.events, &mut self.below);
    }
    pub fn stream(&mut self, env: &Env<Limits>, event: Up) {
        let open = match self.state {
            State::Head { .. } | State::Body(_) => true,
            State::Answering { next, .. } => next != Some(http::Reuse::Close),
            State::Closing(_) => false,
        };
        if open {
            let env = env.with(limits::hook_http(&env.limits));
            http::up(&mut self.http, &env, event, &mut self.events, &mut self.below);
        }
    }
    fn json_request(&mut self, env: &Env<Limits>, request: json::Request) {
        let env = env.with(limits::hook_documents(&env.limits).json);
        json::down(&mut self.json, &env, request, &mut self.json_events, &mut self.json_down);
    }
    fn read(&mut self, env: &Env<Limits>) {
        let read = Read::Arrived { max: limits::hook_http(&env.limits).read };
        self.down(env, http::Request::Body(Down::Demand { read, room: 0 }));
    }
    fn respond(&mut self, env: &Env<Limits>, status: u16) {
        let response = http::Response { status, headers: Box::new([]), body: http::Body::None, close: status != 204 };
        self.down(env, http::Request::Respond(response));
    }
    fn refuse(&mut self, env: &Env<Limits>, status: u16, until: Time, held: Hint) -> State {
        // Respond before Discard: a refusal never demands any body bytes.
        self.respond(env, status);
        self.down(env, http::Request::Discard);
        State::Answering { until, held, next: None }
    }
    fn call(
        &mut self,
        env: &Env<Limits>,
        forges: &skein_lib::List<ForgeEntry>,
        call: http::Call,
        until: Time,
        held: Hint,
    ) -> State {
        let forge = delivery::hook_forge(&call.target);
        let mut secret = None;
        for entry in forges {
            let configured = &entry.base.forge;
            if Some(configured.number) == forge {
                secret = match &configured.access {
                    Some(access) => access.webhook_secret.as_ref(),
                    None => None,
                };
                break;
            }
        }
        let Some(secret) = secret else {
            return self.refuse(env, 404, until, held);
        };
        if call.method != Method::Post {
            return self.refuse(env, 405, until, held);
        }
        let Some(signature) = signature(call.header(delivery::SIGNATURE_HEADER)) else {
            return self.refuse(env, 403, until, held);
        };
        let decoder = match call.header(delivery::EVENT_HEADER) {
            Some(event) => decode::decoder(event, &limits::hook_documents(&env.limits)),
            None => None,
        };
        self.json = Tokenizer::new(&limits::hook_documents(&env.limits).json);
        if decoder.is_some() {
            self.json_request(env, json::Request::Next);
        } else {
            self.read(env);
        }
        State::Body(Body {
            until,
            held,
            forge: forge.expect("configured forge"),
            mac: Hmac::new(secret.expose()),
            signature,
            decoder,
            decoded: None,
            malformed: false,
            ended: false,
        })
    }
    fn body(&mut self, env: &Env<Limits>, event: Up, mut body: Body) -> State {
        match &event {
            Up::Bytes(bytes) => body.mac.update(bytes),
            Up::End => body.ended = true,
            Up::Failed(_) => return State::Closing(Closing::Pending { until: body.until }),
            Up::Room => unreachable!("request body is read only"),
        }
        if body.decoder.is_some() {
            let env = env.with(limits::hook_documents(&env.limits).json);
            json::up(&mut self.json, &env, event, &mut self.json_events, &mut self.json_down);
        } else if !body.ended {
            self.read(env);
        }
        State::Body(body)
    }
    fn verified(&mut self, env: &Env<Limits>, word: &[u8], mut body: Body) -> State {
        let digest = body.mac.finish();
        let mut signature = [0; 64];
        for ([high, low], byte) in signature.as_chunks_mut::<2>().0.iter_mut().zip(digest) {
            *high = hex_digit(byte / 16);
            *low = hex_digit(byte);
        }
        let status = if !sha256::equal(&signature, &body.signature) {
            403
        } else if body.malformed {
            400
        } else {
            204
        };
        if status == 204 {
            body.held.value = match body.decoded {
                Some(delivery) => hint::hint(body.forge, word, &delivery),
                None => None,
            };
        }
        self.respond(env, status);
        State::Answering { until: body.until, held: body.held, next: None }
    }
    fn held(&mut self) -> Option<&mut Hint> {
        match &mut self.state {
            State::Head { held, .. } | State::Body(Body { held, .. }) | State::Answering { held, .. } => Some(held),
            State::Closing(_) => None,
        }
    }
    pub fn refused(&mut self, hint: api::Hint) {
        if let Some(held) = self.held() {
            held.value = Some(hint);
            held.offer = HintState::Refused;
        }
    }
    pub fn offer(&mut self) {
        if let Some(held) = self.held()
            && held.offer == HintState::Refused
        {
            held.offer = HintState::Ready;
        }
    }
    pub fn pending(&self) -> bool {
        let (hint, work) = match &self.state {
            State::Head { held, .. } => (held, false),
            State::Body(body) => (&body.held, body.ended && body.decoder.is_none()),
            State::Answering { held, next, .. } => {
                (held, *next == Some(http::Reuse::Close) || *next == Some(http::Reuse::Keep) && held.value.is_none())
            }
            State::Closing(_) => return !self.sent(),
        };
        work || !self.below.is_empty()
            || !self.json_events.is_empty()
            || !self.json_down.is_empty()
            || hint.offer == HintState::Ready && hint.value.is_some()
            || !self.events.is_empty() && (hint.value.is_none() || hint.offer != HintState::Refused)
    }
    pub fn pump(
        &mut self,
        owner: Token,
        env: &Env<Limits>,
        forges: &skein_lib::List<ForgeEntry>,
        word: &[u8],
        up: &mut Queue<Event>,
        io: &mut Queue<skein_io::Request>,
    ) {
        if self.sent() {
            return;
        }
        for _ in 0_u32..64 {
            if let Some(down) = self.below.pop() {
                io.push(skein_io::Request::Stream { stream: self.socket, down });
                continue;
            }
            if let Some(held) = self.held()
                && held.offer == HintState::Ready
                && held.value.is_some()
            {
                held.offer = HintState::Awaited;
                let hint = held.value.take().expect("ready hint");
                let mut configured = false;
                for entry in forges {
                    if entry.base.forge.number == hint.repository.forge {
                        configured = true;
                        break;
                    }
                }
                if configured {
                    up.push(Event::Hint(Hinted { connection: owner, hint }));
                }
                continue;
            }
            let state = core::mem::replace(&mut self.state, State::Closing(Closing::Aborted));
            let mut progress = true;
            self.state = match state {
                State::Closing(Closing::Pending { .. }) => {
                    io.push(skein_io::Request::Close { entity: self.socket });
                    State::Closing(Closing::Sent)
                }
                State::Body(mut body) if !self.json_events.is_empty() => {
                    let event = self.json_events.pop().expect("queued JSON event");
                    let decoder = body.decoder.as_mut().expect("decoding body");
                    match decode::step(decoder, &limits::hook_documents(&env.limits), &event) {
                        decode::Step::Demand(request) => self.json_request(env, request),
                        decode::Step::Done(delivery) => {
                            body.decoded = Some(delivery);
                            body.decoder = None;
                        }
                        decode::Step::Malformed(_) => {
                            body.malformed = true;
                            body.decoder = None;
                            if !body.ended {
                                self.read(env);
                            }
                        }
                    }
                    State::Body(body)
                }
                state @ (State::Head { .. } | State::Body(_) | State::Answering { .. } | State::Closing(_))
                    if !self.json_down.is_empty() =>
                {
                    let down = self.json_down.pop().expect("queued JSON demand");
                    self.down(env, http::Request::Body(down));
                    state
                }
                State::Body(body) if body.ended && body.decoder.is_none() => self.verified(env, word, body),
                state @ (State::Head { .. } | State::Body(_) | State::Answering { .. } | State::Closing(_))
                    if !self.events.is_empty() =>
                {
                    let event = self.events.pop().expect("queued HTTP event");
                    let (state, advanced) = self.event(env, forges, state, event);
                    progress = advanced;
                    state
                }
                State::Answering { until, held, next: Some(http::Reuse::Keep) } if held.value.is_none() => {
                    self.down(env, http::Request::Next);
                    State::Head { until, held }
                }
                State::Answering { next: Some(http::Reuse::Close), .. } => {
                    io.push(skein_io::Request::Close { entity: self.socket });
                    State::Closing(Closing::Sent)
                }
                state @ (State::Head { .. } | State::Body(_) | State::Answering { .. } | State::Closing(_)) => {
                    self.state = state;
                    self.follow();
                    break;
                }
            };
            self.follow();
            if !progress || self.sent() {
                break;
            }
        }
    }
    fn event(
        &mut self,
        env: &Env<Limits>,
        forges: &skein_lib::List<ForgeEntry>,
        state: State,
        event: http::Event,
    ) -> (State, bool) {
        let state = match event {
            http::Event::Call(call) => match state {
                State::Head { until, held } if held.value.is_none() => self.call(env, forges, call, until, held),
                state @ (State::Head { .. } | State::Body(_) | State::Answering { .. } | State::Closing(_)) => {
                    self.events.push(http::Event::Call(call));
                    return (state, false);
                }
            },
            http::Event::Body(event) => match state {
                State::Body(body) => self.body(env, event, body),
                state @ (State::Head { .. } | State::Answering { .. } | State::Closing(_)) => match event {
                    Up::End => state,
                    Up::Failed(_) => State::Closing(Closing::Pending { until: self.deadline }),
                    Up::Bytes(_) | Up::Room => unreachable!("body delivered without an authenticated head"),
                },
            },
            http::Event::Done(reuse) => {
                let held = match state {
                    State::Head { held, .. } | State::Body(Body { held, .. }) | State::Answering { held, .. } => held,
                    State::Closing(_) => unreachable!("Done precedes closing"),
                };
                State::Answering {
                    until: env.now.saturating_add(env.limits.hook_deadline),
                    held: Hint { offer: HintState::Ready, ..held },
                    next: Some(reuse),
                }
            }
            http::Event::Closed => state,
            http::Event::Failed(_) | http::Event::Refused(_) | http::Event::Reply(_) | http::Event::Ended => {
                State::Closing(Closing::Pending { until: self.deadline })
            }
        };
        (state, true)
    }
}

fn signature(header: Option<&[u8]>) -> Option<[u8; 64]> {
    let bytes = <[u8; 64]>::try_from(header?).ok()?;
    for byte in &bytes {
        if !byte.is_ascii_digit() && !(b'a'..=b'f').contains(byte) {
            return None;
        }
    }
    Some(bytes)
}
