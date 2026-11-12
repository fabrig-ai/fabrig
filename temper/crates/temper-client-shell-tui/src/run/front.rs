//! The run's children and pipes (temper/docs/design/client/tui/box.md, The box's start and Its exit).
use super::Front;
use crate::host::Stop;
use skein_io::{
    self as io,
    kernel::{Exit, Pipe, Signal, Spawn, Target, Way},
};
use skein_lib::{Queue, Secret, Token, stream};
use temper_client_tui::{BoxEnd, BoxLine, BoxStage};

pub(super) const FETCH: Token = Token::new(u64::MAX - 1);
pub(super) const BOX: Token = Token::new(u64::MAX - 2);
const TAIL: usize = 8192;
const LINK: usize = 4096;

pub(super) enum Report {
    Stage(BoxStage),
    Line(BoxLine),
    Link { engine: io::Addr, address: Box<[u8]>, link: Secret },
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Fetch,
    Box,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Source {
    Fetch,
    Error,
    Output,
    Link,
    Lines,
}
struct Read {
    token: Token,
    source: Source,
    reading: bool,
    closing: bool,
    skipping: bool,
}

pub(super) struct Child {
    settings: Front,
    active: Option<Kind>,
    wanted: Option<Kind>,
    process: Option<Token>,
    life: Option<Token>,
    reads: Vec<Read>,
    exit: Option<Exit>,
    box_exit: Option<Exit>,
    found: Option<Box<[u8]>>,
    error: Vec<u8>,
    tail: Vec<u8>,
    link: Vec<u8>,
    linked: bool,
    failure: Option<Box<[u8]>>,
    stop: Option<Stop>,
    reaping: bool,
    ended: Option<Exit>,
    closing: bool,
    pub(super) requests: Queue<io::Request>,
    pub(super) reports: Queue<Report>,
}
impl Child {
    pub(super) fn new(settings: Front) -> Self {
        Self {
            settings,
            active: None,
            wanted: Some(Kind::Fetch),
            process: None,
            life: None,
            reads: Vec::new(),
            exit: None,
            box_exit: None,
            found: None,
            error: Vec::new(),
            tail: Vec::new(),
            link: Vec::new(),
            linked: false,
            failure: None,
            stop: None,
            reaping: false,
            ended: None,
            closing: false,
            requests: Queue::uncapped(),
            reports: Queue::uncapped(),
        }
    }
    fn spawn(&self, kind: Kind) -> Spawn {
        let (program, args, pipes) = match kind {
            Kind::Fetch => (
                self.settings.executable.clone(),
                vec![
                    Box::from(b"internal".as_slice()),
                    Box::from(b"fetch-box".as_slice()),
                    Box::from(b"--cache".as_slice()),
                    self.settings.directories.cache.clone(),
                ],
                vec![
                    Pipe { child: 0, way: Way::Null, parent: None },
                    Pipe { child: 1, way: Way::Out, parent: None },
                    Pipe { child: 2, way: Way::Out, parent: None },
                ],
            ),
            Kind::Box => {
                let mut args = vec![Box::from(b"--front".as_slice())];
                args.extend(self.settings.arguments.iter().cloned());
                (
                    self.found.clone().expect("found box"),
                    args,
                    (0..5)
                        .map(|child| Pipe { child, way: if child == 0 { Way::In } else { Way::Out }, parent: None })
                        .collect(),
                )
            }
        };
        Spawn {
            program,
            args: args.into(),
            env: self.settings.environment.clone(),
            root: self.settings.directory,
            dir: Box::from(b".".as_slice()),
            pipes: pipes.into(),
        }
    }
    pub(super) fn login(&self, name: &[u8]) -> Option<Spawn> {
        let program = self.found.clone()?;
        let mut args = vec![Box::from(b"program-login".as_slice())];
        args.extend(self.settings.directories.flags());
        args.push(name.into());
        Some(Spawn {
            program,
            args: args.into(),
            env: self.settings.environment.clone(),
            root: self.settings.directory,
            dir: Box::from(b".".as_slice()),
            pipes: (0..3).map(|child| Pipe { child, way: Way::Inherit, parent: None }).collect(),
        })
    }
    pub(super) fn progress(&mut self) {
        if let Some(kind) = self.wanted.take() {
            if self.stop.is_some() {
                self.end(Exit::Code(0));
            } else {
                let owner = if kind == Kind::Fetch { FETCH } else { BOX };
                let spawn = self.spawn(kind);
                self.active = Some(kind);
                self.exit = None;
                self.reaping = false;
                self.requests.push(io::Request::SpawnAlone { owner, spawn });
                if kind == Kind::Box {
                    self.reports.push(Report::Stage(BoxStage::Starting));
                }
            }
        }
        for read in &mut self.reads {
            if read.reading || read.closing {
                continue;
            }
            read.reading = true;
            let demand = match read.source {
                Source::Fetch | Source::Lines => stream::Read::Scan {
                    until: stream::Delimiter::LF,
                    max: u32::try_from(BoxLine::MOST).expect("line bound"),
                },
                Source::Error | Source::Output | Source::Link => stream::Read::Arrived { max: 8192 },
            };
            self.requests
                .push(io::Request::Stream { stream: read.token, down: stream::Down::Demand { read: demand, room: 0 } });
        }
        if self.exit.is_some() && self.reads.is_empty() && !self.reaping {
            self.reaping = true;
            if let Some(entity) = self.life.take() {
                self.requests.push(io::Request::Abort { entity });
            }
            if let Some(entity) = self.process {
                self.requests.push(io::Request::Abort { entity });
            }
        }
    }
    pub(super) fn event(&mut self, event: &io::Event) -> bool {
        match event {
            io::Event::Spawned { owner, child, pipes } if *owner == FETCH || *owner == BOX => {
                self.process = Some(*child);
                let sources: &[Source] = if *owner == FETCH {
                    &[Source::Fetch, Source::Error]
                } else {
                    self.life = pipes.first().copied();
                    &[Source::Output, Source::Output, Source::Link, Source::Lines]
                };
                let offset = usize::from(*owner == BOX);
                for (token, source) in pipes.iter().skip(offset).zip(sources) {
                    self.reads.push(Read {
                        token: *token,
                        source: *source,
                        reading: false,
                        closing: false,
                        skipping: false,
                    });
                }
                if self.closing {
                    self.close();
                } else if self.stop.is_some() {
                    self.stop_child();
                }
                true
            }
            io::Event::Exited { owner, exit } if *owner == FETCH || *owner == BOX => {
                self.exit = Some(*exit);
                if *owner == BOX {
                    self.box_exit = Some(*exit);
                }
                true
            }
            io::Event::Failed { owner, error } if *owner == FETCH || *owner == BOX => {
                self.fail(
                    format!("cannot start {}: {error:?}", if *owner == FETCH { "fetch-box" } else { "temper-box" })
                        .into_bytes(),
                );
                true
            }
            io::Event::Closed { owner } if *owner == FETCH || *owner == BOX || self.process == Some(*owner) => {
                self.process = None;
                let kind = self.active.take();
                if self.stop.is_some() {
                    self.end(self.box_exit.unwrap_or(Exit::Code(0)));
                } else if kind == Some(Kind::Fetch) {
                    if self.exit == Some(Exit::Code(0)) && self.found.is_some() && self.failure.is_none() {
                        self.wanted = Some(Kind::Box);
                    } else if self.failure.is_none() {
                        self.fail(if self.error.is_empty() {
                            b"temper-box was not found".to_vec()
                        } else {
                            self.error.clone()
                        });
                    }
                } else if let Some(exit) = self.box_exit {
                    self.end(exit);
                }
                true
            }
            io::Event::Stream { owner, up } if self.reads.iter().any(|read| read.token == *owner) => {
                self.stream(*owner, up);
                true
            }
            io::Event::Stream { owner, .. } if self.life == Some(*owner) => true,
            io::Event::Closed { owner } if self.reads.iter().any(|read| read.token == *owner) => {
                self.reads.retain(|read| read.token != *owner);
                true
            }
            io::Event::Closed { owner } if self.life == Some(*owner) => {
                self.life = None;
                true
            }
            io::Event::Shutdown { .. }
            | io::Event::Failed { .. }
            | io::Event::Spawned { .. }
            | io::Event::Exited { .. }
            | io::Event::Closed { .. }
            | io::Event::Stream { .. }
            | io::Event::Resized
            | io::Event::Output { .. }
            | io::Event::Listening { .. }
            | io::Event::Accepted { .. }
            | io::Event::Connecting { .. }
            | io::Event::Connected { .. }
            | io::Event::Usage { .. }
            | io::Event::Window { .. } => false,
        }
    }
    fn stream(&mut self, token: Token, up: &stream::Up) {
        let read = self.reads.iter_mut().find(|read| read.token == token).expect("owned pipe");
        read.reading = false;
        let source = read.source;
        match up {
            stream::Up::Bytes(bytes) => match source {
                Source::Error => {
                    let room = TAIL.saturating_sub(self.error.len());
                    self.error.extend(bytes.iter().take(room));
                }
                Source::Output => {
                    self.tail.extend_from_slice(bytes);
                    let remove = self.tail.len().saturating_sub(TAIL);
                    self.tail.drain(..remove);
                }
                Source::Link => {
                    if self.failure.is_none() && self.stop.is_none() {
                        let room = LINK.saturating_add(1).saturating_sub(self.link.len());
                        self.link.extend(bytes.iter().take(room));
                        if self.link.len() > LINK {
                            self.no_link();
                        }
                    }
                }
                Source::Fetch | Source::Lines => {
                    if let Some(line) = bytes.strip_suffix(b"\n") {
                        let whole = !read.skipping;
                        read.skipping = false;
                        if whole {
                            if source == Source::Fetch {
                                self.fetched(line);
                            } else if let Some(line) = BoxLine::read(line) {
                                self.reports.push(Report::Line(line));
                            }
                        }
                    } else {
                        read.skipping = true;
                    }
                }
            },
            stream::Up::End | stream::Up::Failed(_) => {
                read.closing = true;
                self.requests.push(io::Request::Abort { entity: token });
                if source == Source::Link && !self.linked && self.failure.is_none() && self.stop.is_none() {
                    if matches!(up, stream::Up::End) {
                        self.handed();
                    } else {
                        self.no_link();
                    }
                }
            }
            stream::Up::Room => unreachable!("front pipes are read only"),
        }
    }
    fn fetched(&mut self, line: &[u8]) {
        if self.stop.is_some() {
            return;
        }
        if let Some(path) = line.strip_prefix(b"found ").filter(|path| path.starts_with(b"/") && !path.contains(&0)) {
            self.found = Some(path.into());
        } else if let Some(words) =
            line.strip_prefix(b"fetched temper-box ").and_then(|words| words.strip_suffix(b" bytes"))
        {
            let parts: Vec<_> = words.split(|byte| *byte == b' ').collect();
            let number = |bytes: &[u8]| std::str::from_utf8(bytes).ok()?.parse::<u64>().ok();
            let progress = match parts.as_slice() {
                [done] => number(done).map(|done| (done, None)),
                [done, b"of", total] => number(done).zip(number(total)).map(|(done, total)| (done, Some(total))),
                _ => None,
            };
            if let Some((done, total)) = progress {
                self.reports.push(Report::Stage(BoxStage::Fetching { done, total }));
            }
        }
    }
    fn handed(&mut self) {
        let Some((engine, address)) = authority(&self.link) else {
            self.no_link();
            return;
        };
        self.linked = true;
        self.reports.push(Report::Stage(BoxStage::Running));
        self.reports.push(Report::Link { engine, address, link: Secret::new(core::mem::take(&mut self.link).into()) });
    }
    fn no_link(&mut self) {
        self.link.clear();
        self.fail(b"temper-box handed no link".to_vec());
        // A failed start is not the person's stop: its end leaves the TUI open.
        if let Some(entity) = self.life.take() {
            self.requests.push(io::Request::Abort { entity });
        }
    }
    fn fail(&mut self, why: Vec<u8>) {
        if self.failure.is_none() {
            let first = why.split(|byte| *byte == b'\n').next().unwrap_or(&[]);
            self.reports.push(Report::Stage(BoxStage::Failed { why: first.iter().take(512).copied().collect() }));
            self.failure = Some(why.into());
        }
    }
    fn end(&mut self, exit: Exit) {
        if self.ended.is_none() {
            self.ended = Some(exit);
            let end = match exit {
                Exit::Code(code) => BoxEnd::Exited { code },
                Exit::Signal(signal) => BoxEnd::Killed { signal: u8::try_from(signal).expect("signal fits u8") },
            };
            self.reports.push(Report::Stage(BoxStage::Ended { end }));
        }
    }
    pub(super) fn stop(&mut self, stop: Stop) {
        self.stop = Some(if self.stop == Some(Stop::Second) { Stop::Second } else { stop });
        self.stop_child();
        if self.active.is_none() {
            self.wanted = None;
            // Failed fetches have already been reaped, so a quit has nothing to wait for.
            self.end(self.box_exit.unwrap_or(Exit::Code(0)));
        }
    }
    fn stop_child(&mut self) {
        if (self.active == Some(Kind::Fetch) || self.stop == Some(Stop::Second))
            && let Some(child) = self.process
        {
            self.requests.push(io::Request::Signal { child, signal: Signal::Terminate, to: Target::Child });
        }
        if let Some(entity) = self.life.take() {
            self.requests.push(io::Request::Abort { entity });
        }
    }
    pub(super) fn leave(&mut self) {
        self.wanted = None;
        self.stop = Some(Stop::First);
        self.stop_child();
    }
    pub(super) fn close(&mut self) {
        self.closing = true;
        self.wanted = None;
        if let Some(entity) = self.life.take() {
            self.requests.push(io::Request::Abort { entity });
        }
        if let Some(entity) = self.process {
            self.requests.push(io::Request::Abort { entity });
        }
    }
    pub(super) fn pending(&self) -> bool {
        self.wanted.is_some()
            || !self.requests.is_empty()
            || self.reads.iter().any(|read| !read.reading && !read.closing)
    }
    pub(super) fn exit(&self) -> (u8, Box<[u8]>) {
        if self.stop.is_some() && self.failure.is_none() && self.box_exit.unwrap_or(Exit::Code(0)) == Exit::Code(0) {
            return (0, Box::new([]));
        }
        let mut words = b"temper: ".to_vec();
        let ended = self.box_exit.filter(|exit| *exit != Exit::Code(0) || self.failure.is_none());
        let code = if let Some(exit) = ended {
            let (line, code) = match exit {
                Exit::Code(code) => (format!("temper-box exited {code}\n"), code),
                Exit::Signal(signal) => (format!("temper-box killed by signal {signal}\n"), 1),
            };
            words.extend_from_slice(line.as_bytes());
            code
        } else {
            let why = self.failure.as_deref().unwrap_or(b"temper-box handed no link");
            // fetch-box already writes the binary's prefix on its stderr.
            words.extend_from_slice(why.strip_prefix(b"temper: ").unwrap_or(why));
            if !words.ends_with(b"\n") {
                words.push(b'\n');
            }
            1
        };
        words.extend_from_slice(&self.tail);
        if ended.is_some() {
            if !words.ends_with(b"\n") {
                words.push(b'\n');
            }
            words.extend_from_slice(b"Log: ");
            words.extend_from_slice(&self.settings.log);
            words.push(b'\n');
        }
        (code, words.into())
    }
}

fn authority(link: &[u8]) -> Option<(io::Addr, Box<[u8]>)> {
    if link.is_empty() || link.len() > LINK {
        return None;
    }
    let url = std::str::from_utf8(link).ok()?.strip_prefix("http://")?;
    let (authority, _) = url.split_once('/')?;
    let address = crate::start::address(format!("http://{authority}").as_bytes()).ok()?;
    if address.port() == 0 {
        return None;
    }
    Some((io::Addr::Ip(address), authority.as_bytes().into()))
}

#[cfg(test)]
mod tests;
