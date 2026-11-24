//! The example's HTTP/SSE fan-out over scripted streams (skein/docs/design/ui/web/testing.md, 3).
use super::{Addr, BTreeMap, Bundle, EVENT_BYTES, LISTENER, Request, Time, Token, Wall, Web, io, io_limits, stream};
use skein_fake_person::{Find, Next, Person, Step, TreeFace};
use skein_lib::base64::{self, Alphabet};
use skein_lib::{Duration, Intake, Queue, Reader, bytes};
use skein_ui::{Role, View};
use skein_ui_example_view::records::{Add, CEILINGS, Label, Note, Notes, Record};
use skein_ui_example_view::{Duties, Place, Report, State, limits};

const FIRST_READER: Token = Token::new(10);
const SECOND_READER: Token = Token::new(11);
const EMPTY_LIST: Token = Token::new(12);
const WRITER: Token = Token::new(13);
const RETRY: Token = Token::new(14);
const FINAL_LIST: Token = Token::new(15);
const ADD_KEY: u64 = 42;
const TITLE: &[u8] = b"Groceries";
const GET_LIST: &[u8] = b"GET /api/notes HTTP/1.1\r\nHost: notes\r\n\r\n";
const GET_EVENTS: &[u8] = b"GET /api/events HTTP/1.1\r\nHost: notes\r\n\r\n";
const OPENING: &[u8] = b":\n\n";

#[test]
fn one_add_and_its_retry_reach_both_readers_once() {
    let mut web = Streams::new();
    let readers = [FIRST_READER, SECOND_READER];
    for reader in readers {
        web.request(reader, GET_EVENTS);
        assert_eq!(web.peer(reader).event_body(), OPENING, "opening comment for {reader:?}");
    }
    web.request(EMPTY_LIST, GET_LIST);
    let empty = web.peer(EMPTY_LIST).body();
    assert!(notes(empty).notes.is_empty(), "the readers start from an empty list");
    let mut views = readers.map(|_| Reading::new(empty));

    let add = Add {
        key: ADD_KEY,
        title: TITLE.into(),
        body: Box::from([]),
        due: Box::from([]),
        minutes: 0,
        label: Label::Idea,
    };
    let body = add.encode(&CEILINGS).expect("fixture add fits");
    let head = format!(
        "POST /api/notes HTTP/1.1\r\nHost: notes\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n",
        body.len()
    );
    let post = [head.as_bytes(), &body].concat();
    web.request(WRITER, &post);
    let delivered = readers.map(|reader| web.peer(reader).event_body());
    web.request(RETRY, &post);
    web.request(FINAL_LIST, GET_LIST);

    let Add { title, body, due, minutes, label, .. } = add;
    let expected = Note { number: 1, title, body, due, minutes, label };
    assert_eq!(notes(web.peer(FINAL_LIST).body()).notes.as_slice(), std::slice::from_ref(&expected));
    for writer in [WRITER, RETRY] {
        assert_eq!(note(web.peer(writer).body()), expected, "note returned to {writer:?}");
    }
    for ((reader, view), body) in readers.into_iter().zip(&mut views).zip(delivered) {
        let data = body
            .strip_prefix(OPENING)
            .and_then(|body| body.strip_prefix(b"data: "))
            .and_then(|body| body.strip_suffix(b"\n\n"))
            .expect("comment followed by a data event");
        let record = base64::decode(Alphabet::Standard, data, EVENT_BYTES)
            .expect("the first add sends exactly one base64 data event");
        assert_eq!(note(&record), expected, "note delivered to {reader:?}");
        assert_eq!(web.peer(reader).event_body(), body, "the retry sends nothing to {reader:?}");
        view.sees_note(&body);
    }
}

fn notes(body: &[u8]) -> Notes {
    Notes::decode(&CEILINGS, &mut Reader::new(body)).expect("list reply decodes")
}

fn note(body: &[u8]) -> Note {
    Note::decode(&CEILINGS, &mut Reader::new(body)).expect("note decodes")
}

struct Reading {
    state: State,
    duties: Duties,
}

impl Reading {
    fn new(empty: &[u8]) -> Self {
        let mut reading = Self {
            state: State::start(1, b"/", &[], 0, b"en"),
            duties: Duties {
                writes: Queue::uncapped(),
                removes: Queue::uncapped(),
                exchanges: Queue::uncapped(),
                show: None,
                stream: None,
            },
        };
        reading.report(Report::StreamOpened);
        let list = reading.duties.exchanges.pop().expect("stream opening requests the list");
        reading.report(Report::Answered { exchange: list.id, status: 200, body: empty.into() });
        reading
    }

    fn report(&mut self, report: Report) {
        self.state.step(report, Time::ZERO, Wall::EPOCH, &mut self.duties);
    }

    fn sees_note(&mut self, body: &[u8]) {
        self.report(Report::StreamPiece { bytes: body.into() });
        let limits = limits::limits();
        let mut view = View::new(&limits);
        self.state.build(&mut view, &mut Queue::with_capacity(limits.patches));
        let mut face = TreeFace::new(view.tree());
        let link = Find::exactly_named(Role::Link, TITLE);
        let script = [
            Step::See { find: link.clone(), within: Duration::ZERO },
            Step::Gone { find: link.nth(1), within: Duration::ZERO },
        ];
        let steps = script.len();
        let mut reader = Person::<Place>::new(Box::new(script));
        for _ in 0..steps {
            assert_eq!(reader.next(&mut face, Time::ZERO), Next::Wait { until: None });
        }
        assert_eq!(reader.next(&mut face, Time::ZERO), Next::Done);
    }
}

// The fixture ends at the server's io requests: it binds accepted sockets,
// meets reads from complete requests, grants write room and captures wire bytes.
struct Streams {
    server: Web,
    peers: BTreeMap<Token, Peer>,
}

struct Peer {
    owner: Option<Token>,
    receive: Intake,
    sent: Vec<u8>,
    closed: bool,
}

impl Streams {
    fn new() -> Self {
        let bundle = Bundle::new(Box::from([]), "0123456789abcdef".into(), BTreeMap::new()).expect("fixture bundle");
        let mut server = Web::new(([127, 0, 0, 1], 0).into(), bundle, Time::ZERO, Wall::EPOCH);
        server.requests.clear(); // No kernel listener: the peers supply accepts.
        Self { server, peers: BTreeMap::new() }
    }

    fn request(&mut self, socket: Token, request: &[u8]) {
        let mut receive = Intake::with_capacity(io_limits().intake);
        receive.append(request).expect("one fixture request fits");
        self.peers.insert(socket, Peer { owner: None, receive, sent: Vec::new(), closed: false });
        self.server.event(io::Event::Accepted { owner: LISTENER, socket, peer: Addr::Ip(([127, 0, 0, 1], 0).into()) });
        while let Some(request) = self.server.requests.pop() {
            match request {
                Request::Bind { socket, owner } => {
                    self.peers.get_mut(&socket).expect("accepted peer").owner = Some(owner);
                }
                Request::Close { entity } => {
                    let peer = self.peers.get_mut(&entity).expect("closing peer");
                    peer.closed = true;
                    self.server.event(io::Event::Closed { owner: peer.owner.expect("bound peer") });
                }
                Request::Stream { stream, down } => {
                    let peer = self.peers.get_mut(&stream).expect("known peer");
                    assert!(!peer.closed, "no stream request after close");
                    match down {
                        stream::Down::Demand { read, room } => {
                            let answer = peer
                                .receive
                                .meet(read)
                                .map(stream::Up::Bytes)
                                .or_else(|| (room > 0).then_some(stream::Up::Room));
                            if let Some(up) = answer {
                                self.server.event(io::Event::Stream { owner: peer.owner.expect("bound peer"), up });
                            }
                        }
                        stream::Down::Send(bytes) => peer.sent.extend(bytes),
                        stream::Down::Finish => unreachable!("HTTP uses the connection close"),
                    }
                }
                other @ (Request::Listen { .. }
                | Request::Connect { .. }
                | Request::Reject { .. }
                | Request::Output { .. }
                | Request::Spawn { .. }
                | Request::SpawnAlone { .. }
                | Request::Signal { .. }
                | Request::Usage { .. }
                | Request::Window { .. }
                | Request::Abort { .. }) => unreachable!("unexpected fake stream request: {other:?}"),
            }
        }
    }

    fn peer(&self, socket: Token) -> &Peer {
        self.peers.get(&socket).expect("fixture peer")
    }
}

impl Peer {
    fn body(&self) -> &[u8] {
        assert!(self.sent.starts_with(b"HTTP/1.1 200 "), "successful reply: {}", self.sent.escape_ascii());
        split(&self.sent, b"\r\n\r\n").1
    }

    fn event_body(&self) -> Vec<u8> {
        assert!(!self.closed, "the event stream is still open");
        let mut wire = self.body();
        let mut body = Vec::new();
        while !wire.is_empty() {
            let (size, rest) = split(wire, b"\r\n");
            let size = usize::from_str_radix(std::str::from_utf8(size).expect("chunk size is ASCII"), 16)
                .expect("chunk has a hex size");
            assert_ne!(size, 0, "open streams have no last chunk");
            let (piece, trailer) = rest.split_at_checked(size).expect("complete chunk");
            body.extend_from_slice(piece);
            wire = trailer.strip_prefix(b"\r\n").expect("chunk ends with CRLF");
        }
        body
    }
}

fn split<'a>(bytes: &'a [u8], delimiter: &[u8]) -> (&'a [u8], &'a [u8]) {
    let at = bytes::find(bytes, delimiter).expect("HTTP framing delimiter");
    let (before, rest) = bytes.split_at(at);
    (before, rest.strip_prefix(delimiter).expect("delimiter found"))
}
