//! The mapping's calls carried by skein's transport (protocol/browser/README.md, 10).
use jig_client_protocol_http::Call;
use skein_ui_web::{fetch, store, transport};

/// A step's HTTP and small-store duties, before touching the platform.
#[derive(Debug)]
pub enum Asked {
    Exchange {
        id: u32,
        call: Call,
        bound: u32,
        pieces: Option<u32>,
        cache: fetch::Cache,
        keep: &'static [&'static str],
    },
    OpenStream {
        call: Call,
        piece: u32,
    },
    CloseStream,
    Save {
        key: Box<[u8]>,
        value: Box<[u8]>,
    },
    Erase {
        key: Box<[u8]>,
    },
}
/// A platform report to translate into the application's host event.
#[derive(Debug, PartialEq, Eq)]
pub enum Heard {
    Answered { exchange: u32, status: u16, body: Box<[u8]>, headers: Vec<(String, String)> },
    Begun { exchange: u32, status: u16 },
    Part { exchange: u32, bytes: Box<[u8]> },
    Done { exchange: u32 },
    Failed { exchange: u32 },
    StreamOpened,
    StreamRefused { status: u16 },
    Piece { bytes: Box<[u8]> },
    StreamEnded,
    Stored { key: Box<[u8]> },
    NotStored { key: Box<[u8]> },
}
/// Live exchange answers, cut by the mapping (jig/docs/design/protocol/client/default-http.md, 6).
#[derive(Default)]
pub struct Exchanges {
    flights: std::collections::BTreeMap<u32, Answer>,
}
struct Answer {
    exchange: skein_lib::Token,
    frames: Option<jig_client_protocol_http::Frames>,
}
impl Exchanges {
    /// None receives an ordinary answer whole; Some receives a batch in pieces.
    pub fn insert(&mut self, id: u32, exchange: skein_lib::Token, limits: Option<&jig_client_protocol_http::Limits>) {
        let frames = limits.map(|limits| jig_client_protocol_http::Frames::new(exchange, limits));
        assert!(self.flights.insert(id, Answer { exchange, frames }).is_none(), "fetch id is unique");
    }
    pub fn heard(&mut self, id: u32, report: Heard, out: &mut skein_lib::Queue<jig_client_protocol_http::Heard>) {
        use jig_client_protocol_http::{Heard as Above, answered};
        let Some(answer) = self.flights.get_mut(&id) else { return };
        let terminal = match report {
            Heard::Answered { status, body, .. } => {
                if let Some(frames) = &mut answer.frames {
                    frames.opened(status, out);
                    if frames.active() {
                        frames.body(&body, out);
                    }
                    if frames.active() {
                        frames.ended(out);
                    }
                } else {
                    out.push(answered(answer.exchange, status, body));
                }
                true
            }
            Heard::Failed { .. } => {
                out.push(Above::Lost { exchange: answer.exchange });
                true
            }
            Heard::Begun { status, .. } => {
                if let Some(frames) = &mut answer.frames {
                    frames.opened(status, out);
                    !frames.active()
                } else {
                    out.push(Above::Lost { exchange: answer.exchange });
                    true
                }
            }
            Heard::Part { bytes, .. } => {
                if let Some(frames) = &mut answer.frames {
                    frames.body(&bytes, out);
                    !frames.active()
                } else {
                    out.push(Above::Lost { exchange: answer.exchange });
                    true
                }
            }
            Heard::Done { .. } => {
                if let Some(frames) = &mut answer.frames {
                    frames.ended(out);
                } else {
                    out.push(Above::Lost { exchange: answer.exchange });
                }
                true
            }
            Heard::StreamOpened
            | Heard::StreamRefused { .. }
            | Heard::Piece { .. }
            | Heard::StreamEnded
            | Heard::Stored { .. }
            | Heard::NotStored { .. } => unreachable!("only exchange reports name a flight"),
        };
        if terminal {
            self.flights.remove(&id);
        }
    }
}

/// Hand the entire step to the transport, which writes before sending.
#[must_use]
pub fn requests(asked: Vec<Asked>) -> transport::Requests {
    let mut requests = transport::Requests::default();
    for ask in asked {
        match ask {
            Asked::Save { key, value } => requests.writes.push(store::Record { key, value }),
            Asked::Erase { key } => requests.removes.push(key),
            Asked::CloseStream => requests.stream = Some(transport::StreamDuty::Close),
            Asked::OpenStream { call, piece } => {
                requests.stream =
                    Some(transport::StreamDuty::Open { path: call.path, headers: headers(call.headers), bound: piece });
            }
            Asked::Exchange { id, call, bound, pieces, cache, keep } => {
                let headers =
                    headers(call.headers).into_iter().map(|(name, value)| (text(name), text(value))).collect();
                requests.exchanges.push(fetch::Exchange {
                    id,
                    path: call.path,
                    method: text(call.method.as_bytes().into()),
                    headers,
                    body: call.body,
                    bound,
                    pieces,
                    cache,
                    keep,
                });
            }
        }
    }
    requests
}
fn headers(headers: skein_lib::List<skein_http::Header>) -> Vec<transport::Header> {
    let mut result = vec![(Box::from(b"Jig-Client".as_slice()), Box::from(b"browser".as_slice()))];
    for header in headers.into_boxed().into_vec() {
        result.push((header.name, header.value));
    }
    result
}
fn text(bytes: Box<[u8]>) -> String {
    String::from_utf8(bytes.into_vec()).expect("the HTTP mapping gives UTF-8")
}
/// The next exchange read is asked after its opening or piece has been stepped (protocol/browser/loop.md, 1.2).
#[must_use]
pub fn read_on_exchange(report: &fetch::Report) -> Option<u32> {
    match report {
        fetch::Report::Opened { exchange, .. } | fetch::Report::Piece { exchange, .. } => Some(*exchange),
        fetch::Report::Answered { .. }
        | fetch::Report::Failed { .. }
        | fetch::Report::Ended { .. }
        | fetch::Report::StreamOpened
        | fetch::Report::StreamRefused { .. }
        | fetch::Report::StreamPiece { .. }
        | fetch::Report::StreamEnded => None,
    }
}

#[must_use]
pub fn fetched(report: fetch::Report) -> Heard {
    match report {
        fetch::Report::Answered { exchange, status, body, headers } => {
            Heard::Answered { exchange, status, body, headers }
        }
        fetch::Report::Failed { exchange } => Heard::Failed { exchange },
        fetch::Report::Opened { exchange, status, headers: _ } => Heard::Begun { exchange, status },
        fetch::Report::Piece { exchange, bytes } => Heard::Part { exchange, bytes },
        fetch::Report::Ended { exchange } => Heard::Done { exchange },
        fetch::Report::StreamOpened => Heard::StreamOpened,
        fetch::Report::StreamRefused { status } => Heard::StreamRefused { status },
        fetch::Report::StreamPiece { bytes } => Heard::Piece { bytes },
        fetch::Report::StreamEnded => Heard::StreamEnded,
    }
}
#[must_use]
pub fn carried(report: transport::Carried) -> Heard {
    match report {
        transport::Carried::Written { key } => Heard::Stored { key },
        transport::Carried::Refused { key } => Heard::NotStored { key },
        transport::Carried::Failed { exchange } => Heard::Failed { exchange },
    }
}
