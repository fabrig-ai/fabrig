//! Kernel and hosted glue for reading a list (smith/docs/design/protocol/agent.md, 6.1).
//! Keeps its I/O queues and one Exchange; holds the credential until the request is written.
//! `Process::new` adopts the supplied faces; `run_decoded` supplies the native kernel.

#[cfg(unix)]
use std::{io::Write, path::Path};

use skein_io::{
    self as io, Event, Io, Request,
    kernel::{Complete, Exit, Fd, Submit},
};
use skein_lib::{
    Duration, Env, List, Queue, Reader, Time, Token, Wall, Writer,
    stream::{Down, Read, Up},
};
use skein_llm_connection::{
    Deadlines,
    list::{Exchange, ExchangeLimits, ListAnswer as Answer},
};
use skein_shell::Host;
use smith_channel::{self as wire, CEILINGS};

#[cfg(unix)]
use crate::config;

/// Select one standalone list read instead of a run.
pub const FLAG: &str = "--list";
const QUEUE: u32 = 64;
const SOCKET: Token = Token::new(100);

/// Owned descriptors transferred by native startup or the hosted factory.
#[derive(Debug)]
pub struct Faces {
    pub stdin: Fd,
    pub stdout: Fd,
    pub stderr: Option<Fd>,
    pub signal: Fd,
}

/// One bounded ask, one exchange and one answer, driven until every face settles.
pub struct Process {
    io: Io,
    endpoints: smith_protocol_llm::Endpoints,
    input: Token,
    output: Token,
    signal: Token,
    ask: Vec<u8>,
    reading: bool,
    exchange: Option<Exchange>,
    answer: Option<Box<[u8]>>,
    answered: bool,
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
    events: Queue<Event>,
    requests: Queue<Request>,
}

impl std::fmt::Debug for Process {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("Process").field("answered", &self.answered).finish_non_exhaustive()
    }
}

fn ask_bound() -> u32 {
    u32::try_from(wire::ListAsk::encoded_bound(&CEILINGS).expect("list ask bound")).expect("ask fits u32")
}

pub(crate) fn limits() -> io::Limits {
    io::Limits {
        sockets: 5,
        names: 0,
        intake: 8192,
        receive: 8192,
        output: u32::try_from(wire::ListAnswer::encoded_bound(&CEILINGS).expect("answer bound"))
            .expect("answer fits u32"),
        refusals: 1,
        sends: 1,
        accepts: 1,
        backlog: 1,
        close_timeout: Duration::from_secs(1),
        retry: Duration::from_millis(10),
    }
}

/// Kernel ring size for four faces and one connection.
#[must_use]
pub fn operations() -> u32 {
    io::operations(&limits()).expect("list operation bound")
}

impl Process {
    /// Resolve startup endpoints without opening a machine, event sink or run.
    #[cfg(unix)]
    pub fn new(decoded: config::Decoded, faces: Faces) -> Result<Self, String> {
        let endpoints = decoded.list_endpoints()?;
        Ok(Self::with_endpoints(endpoints, faces))
    }

    /// Host a list read with the caller's resolved agent endpoints and descriptors.
    #[must_use]
    #[expect(clippy::needless_pass_by_value, reason = "startup transfers descriptor ownership to the process")]
    pub fn with_endpoints(endpoints: smith_protocol_llm::Endpoints, faces: Faces) -> Self {
        let mut io = Io::new(&limits());
        let input = io.adopt_read_pipe(faces.stdin).expect("list stdin slot");
        let output = io.adopt_write_pipe(faces.stdout, None).expect("list stdout slot");
        let signal = io.adopt_signals(faces.signal).expect("list signal slot");
        let mut requests = Queue::uncapped();
        if let Some(fd) = faces.stderr {
            let error = io.adopt_write_pipe(fd, None).expect("list stderr slot");
            requests.push(Request::Close { entity: error });
        }
        Self {
            io,
            endpoints,
            input,
            output,
            signal,
            ask: Vec::new(),
            reading: false,
            exchange: None,
            answer: None,
            answered: false,
            completions: Queue::with_capacity(QUEUE),
            submissions: Queue::with_capacity(QUEUE),
            events: Queue::uncapped(),
            requests,
        }
    }

    fn answer(&mut self, answer: Answer) {
        let answer = to_wire(answer).unwrap_or_else(|()| failed(wire::ListFailed::Unreadable));
        let mut writer = Writer::new(usize::try_from(answer.measure()).expect("bounded answer"));
        answer.encode(&mut writer).expect("measured answer");
        self.answer = Some(writer.finish());
        self.ask.clear();
        self.answered = true;
        self.requests.push(Request::Abort { entity: self.input });
        self.requests.push(Request::Abort { entity: self.signal });
        self.requests.push(Request::Stream {
            stream: self.output,
            down: Down::Demand { read: Read::Nothing, room: answer.measure() },
        });
    }

    fn start(&mut self, now: Time, wall: Wall) {
        let ask = wire::ListAsk::decode(&CEILINGS, &mut Reader::new(&self.ask));
        self.ask.clear();
        let Ok(ask) = ask else {
            self.answer(Answer::Unreadable);
            return;
        };
        let ask = ask.into_parts();
        let Some(options) = self.endpoints.resolve(smith_domain::llm::Endpoint(ask.endpoint)) else {
            self.answer(Answer::Unreadable);
            return;
        };
        let endpoint = self.endpoints.destinations().get(options.index).expect("resolved destination");
        let credential = ask.credential.map(|value| {
            let value = value.into_parts();
            let bytes = value.bytes.into_exposed();
            match value.scheme {
                wire::Scheme::Bearer => skein_llm::Credential::bearer(bytes),
                wire::Scheme::Key => skein_llm::Credential::key(bytes),
            }
        });
        let mut exchange = Exchange::new(
            SOCKET,
            endpoint,
            options.provider,
            credential,
            Deadlines { connect: Some(options.connect), handshake: Some(options.handshake), ..Deadlines::none() },
            &ExchangeLimits { wall, ..ExchangeLimits::DEFAULT },
        );
        exchange.start(now, &mut self.requests);
        self.exchange = Some(exchange);
    }

    fn event(&mut self, event: Event, now: Time, wall: Wall) {
        if event.owner() == Some(SOCKET) {
            if let Some(exchange) = &mut self.exchange {
                exchange.up(event, now, &mut self.requests);
            }
            return;
        }
        match event {
            Event::Stream { owner, up } if owner == self.input && !self.answered => match up {
                Up::Bytes(bytes) => {
                    self.reading = false;
                    if skein_lib::len_u64(self.ask.len()).saturating_add(skein_lib::len_u64(bytes.len()))
                        > u64::from(ask_bound())
                    {
                        self.answer(Answer::Unreadable);
                    } else {
                        self.ask.extend_from_slice(&bytes);
                    }
                }
                Up::End => self.start(now, wall),
                Up::Failed(_) => self.answer(Answer::Unreadable),
                Up::Room => unreachable!("stdin is read-only"),
            },
            Event::Stream { owner, up } if owner == self.output => match up {
                Up::Room => {
                    if let Some(bytes) = self.answer.take() {
                        self.requests.push(Request::Stream { stream: owner, down: Down::Send(bytes) });
                        self.requests.push(Request::Close { entity: owner });
                    }
                }
                Up::End | Up::Failed(_) => {
                    self.answer = None;
                    self.requests.push(Request::Abort { entity: owner });
                }
                Up::Bytes(_) => unreachable!("stdout is write-only"),
            },
            Event::Shutdown { .. } => {
                if let Some(exchange) = &mut self.exchange {
                    exchange.cancel(&mut self.requests);
                } else if !self.answered {
                    self.answer(Answer::Deadline);
                }
            }
            Event::Listening { .. }
            | Event::Accepted { .. }
            | Event::Connecting { .. }
            | Event::Connected { .. }
            | Event::Stream { .. }
            | Event::Output { .. }
            | Event::Spawned { .. }
            | Event::Exited { .. }
            | Event::Usage { .. }
            | Event::Resized
            | Event::Window { .. }
            | Event::Failed { .. }
            | Event::Closed { .. } => {}
        }
    }
}

fn failed(failure: wire::ListFailed) -> wire::ListAnswer {
    wire::ListAnswer::Failed(
        wire::ListFailure::new(&CEILINGS, wire::ListFailureParts { failure }).expect("fixed failure"),
    )
}

fn to_wire(answer: Answer) -> Result<wire::ListAnswer, ()> {
    let failure = match answer {
        Answer::Listed(rows) => {
            let mut values = List::with_capacity(CEILINGS.list_rows_rows);
            for row in rows {
                let mut efforts = List::with_capacity(CEILINGS.list_row_efforts);
                for effort in row.efforts {
                    efforts.push(effort).map_err(|_| ())?;
                }
                values
                    .push(
                        wire::ListRow::new(
                            &CEILINGS,
                            wire::ListRowParts {
                                name: row.name,
                                shown: row.shown,
                                window: row.window,
                                output: row.output,
                                efforts,
                            },
                        )
                        .map_err(|_| ())?,
                    )
                    .map_err(|_| ())?;
            }
            return Ok(wire::ListAnswer::Listed(
                wire::ListRows::new(&CEILINGS, wire::ListRowsParts { rows: values }).map_err(|_| ())?,
            ));
        }
        Answer::Refused { status } => wire::ListFailed::Refused(
            wire::ListRefused::new(&CEILINGS, wire::ListRefusedParts { status }).expect("fixed status"),
        ),
        Answer::Unreachable => wire::ListFailed::Unreachable,
        Answer::Unreadable => wire::ListFailed::Unreadable,
        Answer::Deadline => wire::ListFailed::Deadline,
    };
    Ok(failed(failure))
}

impl Host for Process {
    fn iterate(&mut self, now: Time, wall: Wall) {
        let env = Env { now, wall, limits: limits() };
        io::stage::up(&mut self.io, &env, &mut self.completions, &mut self.events, &mut self.submissions, now);
        while let Some(event) = self.events.pop() {
            self.event(event, now, wall);
        }
        if let Some(exchange) = &mut self.exchange {
            exchange.fire(now, &mut self.requests);
            if let Some(answer) = exchange.answer() {
                self.exchange = None;
                self.answer(answer);
            }
        } else if !self.answered && !self.reading {
            self.reading = true;
            let room = ask_bound().saturating_sub(u32::try_from(self.ask.len()).expect("bounded ask"));
            self.requests.push(Request::Stream {
                stream: self.input,
                down: Down::Demand {
                    read: Read::Arrived { max: room.saturating_add(1).min(limits().receive) },
                    room: 0,
                },
            });
        }
        io::stage::down(&mut self.io, &env, &mut self.requests, &mut self.submissions);
        self.io.reclaim();
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submissions
    }
    fn work_pending(&self, _now: Time) -> bool {
        self.io.is_ready() || !self.completions.is_empty() || !self.events.is_empty() || !self.requests.is_empty()
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.io.next_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.exchange.as_ref().and_then(Exchange::next_deadline)
    }
    fn is_empty(&self) -> bool {
        self.answered
            && self.io.is_empty()
            && self.exchange.is_none()
            && self.answer.is_none()
            && self.completions.is_empty()
            && self.submissions.is_empty()
            && self.events.is_empty()
            && self.requests.is_empty()
    }
    fn exit(&self) -> Option<Exit> {
        self.is_empty().then_some(Exit::Code(0))
    }
    fn operations(&self) -> u32 {
        operations()
    }
}

/// Drive one list invocation with native signals and kernel; startup failures go to stderr.
#[cfg(unix)]
pub fn run_decoded(decoded: config::Decoded, mut errors: Box<dyn Write>) -> Result<Exit, String> {
    let result = (|| {
        let signal =
            skein_shell::open_termination_signals().map_err(|error| format!("termination signals: {error:?}"))?;
        let mut process = Process::new(decoded, Faces { stdin: Fd::new(0), stdout: Fd::new(1), stderr: None, signal })?;
        let mut kernel = skein_shell::Kernel::open(skein_shell::Config { operations: operations() })
            .map_err(|error| format!("kernel: {error:?}"))?;
        Ok(skein_shell::drive(&mut kernel, &skein_shell::Clock::new(), &mut process))
    })();
    if let Err(why) = &result {
        let _written = writeln!(errors, "smith-agent: {why}");
    }
    result
}

/// Read the same bounded configuration as a run before reading any ask.
#[cfg(unix)]
pub fn run(path: &Path, mut errors: Box<dyn Write>) -> Result<Exit, String> {
    match config::read_decoded(path) {
        Ok(decoded) => run_decoded(decoded, errors),
        Err(why) => {
            let _written = writeln!(errors, "smith-agent: {why}");
            Err(why)
        }
    }
}
