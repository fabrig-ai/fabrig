//! One cat-file response at a time, then the bounded JSON machine (hearth/docs/design/domain/render/README.md, 2.2).
use crate::{
    Asked, Current, Limits, Mode, builder,
    cache::Cache,
    config,
    git::Child,
    git_machine::{batch, oid::Oid},
    render::{
        self,
        vocabulary::{Blob, Decoded},
    },
    translate,
};
use alloc::boxed::Box;
use skein_io as io;
use skein_lib::{
    Env, Queue, Time, Token, Writer,
    stream::{self, Down, Held, Up},
};
#[derive(Debug)]
struct Decode {
    kind: render::vocabulary::As,
    held: Held,
    machine: config::json::Decoder,
    above: Queue<config::json::Event>,
    below: Queue<Down>,
}
impl Decode {
    fn new(bytes: Box<[u8]>, kind: render::vocabulary::As, env: &Env<Limits>) -> Self {
        let mut decode = Self {
            kind,
            held: Held::new(bytes),
            machine: config::json::Decoder::new(&env.limits.config),
            above: Queue::uncapped(),
            below: Queue::uncapped(),
        };
        let env = Env { now: env.now, wall: env.wall, limits: env.limits.config };
        config::json::down(
            &mut decode.machine,
            &env,
            config::json::Request::Decode,
            &mut decode.above,
            &mut decode.below,
        );
        decode
    }
    fn resume(&mut self, env: &Env<Limits>) -> Option<Result<Decoded, ()>> {
        if let Some(event) = self.above.pop() {
            let result = match event {
                config::json::Event::Decoded(value) => match self.kind {
                    render::vocabulary::As::Document => match config::document::document(value) {
                        Ok(document) => Ok(Decoded::Document(translate::documents::document(document))),
                        Err(error) => Err(error),
                    },
                    render::vocabulary::As::Declaration => {
                        match config::declaration::declaration(value, &env.limits.config) {
                            Ok(declaration) => match translate::declarations::declaration(declaration, &env.limits) {
                                Ok(declaration) => Ok(Decoded::Declaration(declaration)),
                                Err(translate::declarations::Refusal { at, fault }) => {
                                    Ok(Decoded::Refused { at, fault, also: None })
                                }
                            },
                            Err(error) => Err(error),
                        }
                    }
                    render::vocabulary::As::Link
                    | render::vocabulary::As::Template
                    | render::vocabulary::As::Cached => {
                        unreachable!("static bytes are never decoded")
                    }
                },
                config::json::Event::Failed(error) => Err(error),
                config::json::Event::Closed => unreachable!("decoder is not closed"),
            };
            return Some(match result {
                Ok(decoded) => Ok(decoded),
                Err(config::Error { at, fault }) => match translate::errors::fault(fault, &env.limits.config) {
                    Some((fault, also)) => Ok(Decoded::Refused { at: translate::documents::position(at), fault, also }),
                    None => Err(()),
                },
            });
        }
        if let Some(Down::Demand { read, room: 0 }) = self.below.pop() {
            let event = self.held.answer(read).expect("JSON has an active read");
            let env = Env { now: env.now, wall: env.wall, limits: env.limits.config };
            config::json::up(&mut self.machine, &env, event, &mut self.above, &mut self.below);
        }
        None
    }
}
#[derive(Debug)]
#[expect(clippy::large_enum_variant, reason = "the settled terminal stays inline without another allocation")]
enum Arrival {
    Streaming,
    Waiting,
    Settled(Option<builder::vocabulary::Event>),
}
#[derive(Debug)]
enum Holding {
    Body(Writer),
    Decoding(Decode),
    Decoded(Decoded),
    Caching { cache: Cache, arrival: Arrival },
    Spent,
}
impl Holding {
    fn whole(self, kind: render::vocabulary::As, env: &Env<Limits>) -> Self {
        match self {
            Self::Caching { mut cache, arrival } => {
                cache.finish();
                Self::Caching { cache, arrival }
            }
            Self::Body(body) => {
                let bytes = body.finish();
                match kind {
                    render::vocabulary::As::Link => Self::Decoded(Decoded::Link(bytes)),
                    render::vocabulary::As::Template => Self::Decoded(compile_template(&bytes, &env.limits)),
                    render::vocabulary::As::Document | render::vocabulary::As::Declaration => {
                        Self::Decoding(Decode::new(bytes, kind, env))
                    }
                    render::vocabulary::As::Cached => unreachable!("cached bytes stream"),
                }
            }
            Self::Decoding(_) | Self::Decoded(_) | Self::Spent => unreachable!("whole blob"),
        }
    }
    fn cache(&self) -> Option<&Cache> {
        match self {
            Self::Caching { cache, .. } => Some(cache),
            Self::Body(_) | Self::Decoding(_) | Self::Decoded(_) | Self::Spent => None,
        }
    }
    fn cache_mut(&mut self) -> Option<&mut Cache> {
        match self {
            Self::Caching { cache, .. } => Some(cache),
            Self::Body(_) | Self::Decoding(_) | Self::Decoded(_) | Self::Spent => None,
        }
    }
    fn ready(&self) -> bool {
        match self {
            Self::Decoded(_) => true,
            Self::Caching { cache, arrival } => match arrival {
                Arrival::Settled(_) => cache.done(),
                Arrival::Streaming | Arrival::Waiting => false,
            },
            Self::Body(_) | Self::Decoding(_) | Self::Spent => false,
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Feed {
    Due { sent: usize },
    Asked { sent: usize },
    Room { sent: usize },
    Finished,
}
#[derive(Debug)]
pub(crate) struct Read {
    root: skein_io::kernel::Fd,
    rng: skein_lib::Rng,
    current: Holding,
    request: Token,
    blobs: Box<[Blob]>,
    index: usize,
    feed: Feed,
    batch: batch::Batch,
    above: Queue<batch::Event>,
    below: Queue<Down>,
    failed: bool,
    began: Time,
    ended: render::errors::Ended,
}
impl Read {
    pub fn new(request: Token, blobs: Box<[Blob]>, env: &Env<Limits>, root: skein_io::kernel::Fd, seed: u64) -> Self {
        assert!(!blobs.is_empty(), "Read asks for blobs");
        let mut read = Self {
            root,
            rng: skein_lib::Rng::new(seed),
            current: Holding::Spent,
            request,
            blobs,
            index: 0,
            feed: Feed::Due { sent: 0 },
            batch: batch::Batch::START,
            above: Queue::uncapped(),
            below: Queue::uncapped(),
            failed: false,
            began: env.now,
            ended: render::errors::Ended::Stream,
        };
        read.start(env);
        read
    }
    fn start(&mut self, env: &Env<Limits>) {
        self.began = env.now;
        let blob = self.blobs.get(self.index).expect("current blob");
        self.current = match blob.kind {
            render::vocabulary::As::Document
            | render::vocabulary::As::Declaration
            | render::vocabulary::As::Template
            | render::vocabulary::As::Link => {
                Holding::Body(Writer::new(usize::try_from(blob.size).expect("render checked document bound")))
            }
            render::vocabulary::As::Cached => {
                Holding::Caching { cache: Cache::new(self.root, &mut self.rng), arrival: Arrival::Streaming }
            }
        };
        let request =
            batch::Request::Read { oid: Oid::parse(&blob.oid.0).expect("listed oid"), size: blob.size, max: blob.size };
        self.down(env, request);
    }
    fn down(&mut self, env: &Env<Limits>, request: batch::Request) {
        let env = Env { now: env.now, wall: env.wall, limits: env.limits.git };
        batch::down(&mut self.batch, &env, request, &mut self.above, &mut self.below);
    }
    pub fn up(&mut self, env: &Env<Limits>, event: Up) {
        let env = Env { now: env.now, wall: env.wall, limits: env.limits.git };
        batch::up(&mut self.batch, &env, event, &mut self.above, &mut self.below);
    }
    pub fn input_room(&mut self) {
        self.feed = match self.feed {
            Feed::Asked { sent } => Feed::Room { sent },
            Feed::Due { .. } | Feed::Room { .. } | Feed::Finished => unreachable!("one room demand"),
        };
    }
    pub fn input(&mut self, child: &Child, below: &mut Queue<Asked>) -> bool {
        let Some(input) = child.input.token() else {
            return false;
        };
        let down = match self.feed {
            Feed::Due { sent } => match self.blobs.get(sent) {
                Some(blob) => {
                    self.feed = Feed::Asked { sent };
                    Down::Demand {
                        read: stream::Read::Nothing,
                        room: u32::try_from(blob.oid.0.len()).expect("bounded oid").checked_add(1).expect("line fits"),
                    }
                }
                None => {
                    self.feed = Feed::Finished;
                    Down::Finish
                }
            },
            Feed::Room { sent } => {
                let blob = self.blobs.get(sent).expect("room for current blob");
                self.feed = Feed::Due { sent: sent.checked_add(1).expect("bounded blobs") };
                Down::Send(batch::request_line(Oid::parse(&blob.oid.0).expect("listed oid")))
            }
            Feed::Asked { .. } | Feed::Finished => return false,
        };
        below.push(Asked::Io(io::Request::Stream { stream: input, down }));
        true
    }
    pub fn work_pending(&self) -> bool {
        match &self.current {
            Holding::Caching { cache, arrival } => {
                let waiting = match arrival {
                    Arrival::Waiting => true,
                    Arrival::Streaming | Arrival::Settled(_) => false,
                };
                if cache.work_pending() || (cache.done() && !waiting) {
                    return true;
                }
                if cache.busy() {
                    return false;
                }
            }
            Holding::Decoding(_) | Holding::Decoded(_) => return true,
            Holding::Body(_) | Holding::Spent => {}
        }
        !self.above.is_empty()
            || !self.below.is_empty()
            || match self.feed {
                Feed::Due { .. } | Feed::Room { .. } => true,
                Feed::Asked { .. } | Feed::Finished => false,
            }
    }
    pub fn take_request(&mut self) -> Option<Down> {
        if let Some(cache) = self.current.cache()
            && cache.busy()
        {
            return None;
        }
        self.below.pop()
    }
    pub fn close_cache(&mut self, below: &mut Queue<Asked>) -> bool {
        let Some(cache) = self.current.cache_mut() else {
            return true;
        };
        if cache.abort() {
            return true;
        }
        if let Some(request) = cache.take_request() {
            below.push(Asked::File(request));
        }
        false
    }
    pub fn cache_work_pending(&self) -> Option<bool> {
        let cache = self.current.cache()?;
        if cache.done() || cache.failed() {
            return None;
        }
        Some(!cache.closing() || cache.work_pending())
    }
    pub fn ended(&self) -> render::errors::Ended {
        self.ended.clone()
    }
    pub fn failed(&self) -> bool {
        self.failed
            || (match self.current.cache() {
                Some(cache) => cache.failed(),
                None => false,
            })
    }
    pub fn request(&self) -> Token {
        self.request
    }
    pub fn ready(&self) -> bool {
        self.current.ready() && self.index.checked_add(1) == Some(self.blobs.len())
    }
    pub fn take_answer(&mut self) -> render::vocabulary::Event {
        let blob = self.blobs.get(self.index).expect("current blob");
        assert!(self.current.ready(), "decoded blob");
        match core::mem::replace(&mut self.current, Holding::Spent) {
            Holding::Decoded(read) => {
                render::vocabulary::Event::Read { request: self.request, oid: blob.oid.clone(), read }
            }
            Holding::Body(_) | Holding::Decoding(_) | Holding::Caching { .. } | Holding::Spent => {
                unreachable!("decoded blob")
            }
        }
    }
    pub fn temporary(&self) -> Box<[u8]> {
        self.current.cache().expect("arrival temporary").name()
    }
    pub fn settle(&mut self, event: builder::vocabulary::Event) -> Option<builder::vocabulary::Event> {
        let arrival = match &mut self.current {
            Holding::Caching { arrival, .. } => arrival,
            Holding::Body(_) | Holding::Decoding(_) | Holding::Decoded(_) | Holding::Spent => {
                unreachable!("settling a cached arrival")
            }
        };
        *arrival = Arrival::Settled(None);
        if self.index.checked_add(1) == Some(self.blobs.len()) {
            *arrival = Arrival::Settled(Some(event));
            None
        } else {
            Some(event)
        }
    }
    pub fn take_terminal(&mut self) -> Option<builder::vocabulary::Event> {
        match &mut self.current {
            Holding::Caching { arrival, .. } => match arrival {
                Arrival::Settled(terminal) => terminal.take(),
                Arrival::Streaming | Arrival::Waiting => None,
            },
            Holding::Body(_) | Holding::Decoding(_) | Holding::Decoded(_) | Holding::Spent => None,
        }
    }
    pub fn file_up(&mut self, event: skein_io::file::Event) {
        self.current.cache_mut().expect("cache event").up(event);
    }
    pub fn resume(
        &mut self,
        env: &Env<Limits>,
        above: &mut Queue<builder::vocabulary::Event>,
        below: &mut Queue<Asked>,
    ) {
        match &mut self.current {
            Holding::Caching { cache, arrival } => {
                if let Some(request) = cache.take_request() {
                    below.push(Asked::File(request));
                    return;
                }
                if cache.written() {
                    self.down(env, batch::Request::Next);
                    return;
                }
                if cache.done() {
                    match arrival {
                        Arrival::Streaming => {
                            let blob = self.blobs.get(self.index).expect("arriving blob");
                            above.push(builder::vocabulary::Event::Arrived {
                                request: self.request,
                                oid: blob.oid.clone(),
                                size: blob.size,
                                last: self.index.checked_add(1) == Some(self.blobs.len()),
                                arrival: Token::new(10),
                                sha256: cache.digest(),
                                took: env.now.saturating_since(self.began),
                            });
                            *arrival = Arrival::Waiting;
                        }
                        Arrival::Settled(_) => {
                            if self.index.checked_add(1) != Some(self.blobs.len()) {
                                self.index = self.index.checked_add(1).expect("bounded blobs");
                                self.start(env);
                            }
                        }
                        Arrival::Waiting => {}
                    }
                    return;
                }
                if !cache.readable() {
                    return;
                }
            }
            Holding::Decoding(decode) => {
                if let Some(result) = decode.resume(env) {
                    self.current = match result {
                        Ok(decoded) => Holding::Decoded(decoded),
                        Err(()) => {
                            self.failed = true;
                            Holding::Spent
                        }
                    };
                }
                return;
            }
            Holding::Body(_) | Holding::Decoded(_) | Holding::Spent => {}
        }
        if let Some(event) = self.above.pop() {
            match event {
                batch::Event::Piece(bytes) => match &mut self.current {
                    Holding::Caching { cache, .. } => cache.piece(bytes),
                    Holding::Body(body) => {
                        body.put(&bytes).expect("batch bounded piece");
                        self.down(env, batch::Request::Next);
                    }
                    Holding::Decoding(_) | Holding::Decoded(_) | Holding::Spent => {
                        unreachable!("current blob buffer")
                    }
                },
                batch::Event::Whole { .. } => {
                    let current = core::mem::replace(&mut self.current, Holding::Spent);
                    let kind = self.blobs.get(self.index).expect("blob").kind;
                    self.current = current.whole(kind, env);
                }
                batch::Event::Missing { oid } => {
                    self.failed = true;
                    self.ended = render::errors::Ended::Missing {
                        oid: render::revision::Oid(skein_lib::bytes::copy_of(oid.hex())),
                    };
                }
                batch::Event::Failed(error) => {
                    self.failed = true;
                    self.ended = match error {
                        batch::Error::Stream(_) => render::errors::Ended::Stream,
                        batch::Error::Malformed
                        | batch::Error::Other { .. }
                        | batch::Error::Size { .. }
                        | batch::Error::TooLarge { .. }
                        | batch::Error::Truncated => render::errors::Ended::Output,
                    };
                }
                batch::Event::Closed => unreachable!("batch is not closed"),
            }
        } else if self.current.ready() && !self.ready() {
            above.push(builder::vocabulary::Event::Render(self.take_answer()));
            self.index = self.index.checked_add(1).expect("bounded blobs");
            self.start(env);
        }
    }
}

pub(super) fn current(current: &mut Current) -> Option<&mut Read> {
    let (_, mode) = current.git_mut()?;
    match mode {
        Mode::Read(read) => Some(read),
        Mode::Rev { .. } | Mode::Listing { .. } | Mode::Hunks { .. } => None,
    }
}

fn compile_template(bytes: &[u8], limits: &Limits) -> Decoded {
    use hearth_protocol_template::template;
    match template::compile(bytes, &limits.template) {
        Ok(value) => Decoded::Template(translate::instructions::template(value)),
        Err(error) => {
            let (line, column) = template::place(bytes, error.at);
            Decoded::Refused {
                at: render::revision::Position { line, column },
                fault: render::errors::Decode::Syntax(translate::errors::syntax(error.kind)),
                also: None,
            }
        }
    }
}
