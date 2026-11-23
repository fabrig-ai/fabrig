//! Bounded exchanges and kept answer headers (skein/docs/design/ui/web/effects.md, 1).
use skein_lib::usize_of;
use std::collections::BTreeMap;
#[cfg(target_arch = "wasm32")]
pub mod web;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Cache {
    Browser,
    NoStore,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Exchange {
    pub id: u32,
    pub path: Box<[u8]>,
    pub method: String,
    pub headers: Vec<(String, String)>,
    pub body: Box<[u8]>,
    pub bound: u32,
    /// None answers whole; Some bounds each piece and must be positive.
    pub pieces: Option<u32>,
    pub cache: Cache,
    pub keep: &'static [&'static str],
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Report {
    Answered { exchange: u32, status: u16, body: Box<[u8]>, headers: Vec<(String, String)> },
    Failed { exchange: u32 },
    Opened { exchange: u32, status: u16, headers: Vec<(String, String)> },
    Piece { exchange: u32, bytes: Box<[u8]> },
    Ended { exchange: u32 },
    StreamOpened,
    StreamRefused { status: u16 },
    StreamPiece { bytes: Box<[u8]> },
    StreamEnded,
}
struct Stream {
    ticket: u64,
    pieces: Pieces,
}
struct Pieces {
    bound: usize,
    held: Box<[u8]>,
    at: usize,
}
struct Flight {
    id: u32,
    bound: u32,
    body: Vec<u8>,
    keep: Vec<String>,
    received: usize,
    pieces: Option<Pieces>,
}
impl Pieces {
    fn new(bound: u32) -> Self {
        assert!(bound > 0, "a piece holds a byte");
        Self { bound: usize_of(bound), held: Box::from([]), at: 0 }
    }
    fn hold(&mut self, bytes: Box<[u8]>) {
        assert!(self.at == self.held.len(), "read on after the last piece");
        self.held = bytes;
        self.at = 0;
    }
    fn next(&mut self) -> Option<Box<[u8]>> {
        if self.at == self.held.len() {
            return None;
        }
        let end = self.at.saturating_add(self.bound).min(self.held.len());
        let bytes = self.held.get(self.at..end).expect("part within held piece").into();
        self.at = end;
        if end == self.held.len() {
            self.held = Box::from([]);
            self.at = 0;
        }
        Some(bytes)
    }
}
impl Flight {
    fn headers(&self, headers: &[(String, String)]) -> Option<Vec<(String, String)>> {
        let mut kept = Vec::new();
        for name in &self.keep {
            if let Some((_, value)) = headers.iter().find(|(key, _)| key.eq_ignore_ascii_case(name)) {
                if value.len() > usize_of(self.bound) {
                    return None;
                }
                kept.push((name.clone(), value.clone()));
            }
        }
        Some(kept)
    }
}
pub struct Fetch {
    flights: BTreeMap<u64, Flight>,
    capacity: usize,
    serial: u64,
    stream: Option<Stream>,
}
impl Fetch {
    #[must_use]
    pub fn new(capacity: u32) -> Self {
        Self { flights: BTreeMap::new(), capacity: usize_of(capacity), serial: 0, stream: None }
    }
    /// A ticket remains unique after the application abandons and reuses an id.
    pub fn begin(&mut self, id: u32, bound: u32, keep: &[&str]) -> Option<u64> {
        if self.flights.len() == self.capacity || self.flights.values().any(|flight| flight.id == id) {
            return None;
        }
        self.serial = self.serial.checked_add(1).expect("exchange tickets fit");
        self.flights.insert(
            self.serial,
            Flight {
                id,
                bound,
                body: Vec::new(),
                keep: keep.iter().map(|name| name.to_ascii_lowercase()).collect(),
                received: 0,
                pieces: None,
            },
        );
        Some(self.serial)
    }
    pub fn piece(&mut self, ticket: u64, piece: &[u8]) -> Option<Report> {
        let flight = self.flights.get_mut(&ticket)?;
        if piece.len() > usize_of(flight.bound).saturating_sub(flight.received) {
            return self.fail(ticket);
        }
        flight.received = flight.received.checked_add(piece.len()).expect("body within bound");
        flight.body.extend_from_slice(piece);
        None
    }
    #[must_use]
    pub fn remaining(&self, ticket: u64) -> Option<u32> {
        let flight = self.flights.get(&ticket)?;
        flight.bound.checked_sub(u32::try_from(flight.received).expect("body within bound"))
    }
    pub fn finish(&mut self, ticket: u64, status: u16, headers: &[(String, String)]) -> Option<Report> {
        let flight = self.flights.remove(&ticket)?;
        let Some(kept) = flight.headers(headers) else {
            return Some(Report::Failed { exchange: flight.id });
        };
        Some(Report::Answered { exchange: flight.id, status, body: flight.body.into_boxed_slice(), headers: kept })
    }
    /// Open a live answer in pieces, retaining its place until end, failure or abandoning.
    pub fn open_exchange(
        &mut self,
        ticket: u64,
        bound: u32,
        status: u16,
        headers: &[(String, String)],
    ) -> Option<Report> {
        let flight = self.flights.get_mut(&ticket)?;
        let Some(headers) = flight.headers(headers) else {
            return self.fail(ticket);
        };
        flight.pieces = Some(Pieces::new(bound));
        Some(Report::Opened { exchange: flight.id, status, headers })
    }
    #[must_use]
    pub fn exchange_ticket(&self, id: u32) -> Option<u64> {
        self.flights.iter().find_map(|(ticket, flight)| (flight.id == id).then_some(*ticket))
    }
    pub fn exchange_piece(&mut self, ticket: u64, bytes: Box<[u8]>) -> Option<Report> {
        let flight = self.flights.get_mut(&ticket)?;
        let pieces = flight.pieces.as_mut()?;
        if bytes.len() > usize_of(flight.bound).saturating_sub(flight.received) {
            return self.fail(ticket);
        }
        flight.received = flight.received.checked_add(bytes.len()).expect("body within bound");
        pieces.hold(bytes);
        self.read_on_exchange(ticket)
    }
    pub fn read_on_exchange(&mut self, ticket: u64) -> Option<Report> {
        let flight = self.flights.get_mut(&ticket)?;
        flight.pieces.as_mut()?.next().map(|bytes| Report::Piece { exchange: flight.id, bytes })
    }
    pub fn end_exchange(&mut self, ticket: u64) -> Option<Report> {
        let flight = self.flights.remove(&ticket)?;
        Some(Report::Ended { exchange: flight.id })
    }
    pub fn fail(&mut self, ticket: u64) -> Option<Report> {
        self.flights.remove(&ticket).map(|flight| Report::Failed { exchange: flight.id })
    }
    pub fn abandon(&mut self, id: u32) -> Option<u64> {
        let ticket = self.exchange_ticket(id)?;
        self.flights.remove(&ticket);
        Some(ticket)
    }
    pub fn open_stream(&mut self, bound: u32) -> u64 {
        assert!(bound > 0, "a stream piece holds a byte");
        self.serial = self.serial.checked_add(1).expect("stream tickets fit");
        self.stream = Some(Stream { ticket: self.serial, pieces: Pieces::new(bound) });
        self.serial
    }
    #[must_use]
    pub fn stream_ticket(&self) -> Option<u64> {
        self.stream.as_ref().map(|stream| stream.ticket)
    }
    pub fn stream_piece(&mut self, ticket: u64, bytes: Box<[u8]>) -> Option<Report> {
        let stream = self.stream.as_mut().filter(|stream| stream.ticket == ticket)?;
        stream.pieces.hold(bytes);
        self.read_on()
    }
    pub fn read_on(&mut self) -> Option<Report> {
        let stream = self.stream.as_mut()?;
        stream.pieces.next().map(|bytes| Report::StreamPiece { bytes })
    }
    pub fn end_stream(&mut self, ticket: u64) -> Option<Report> {
        if self.stream_ticket() != Some(ticket) {
            return None;
        }
        self.stream = None;
        Some(Report::StreamEnded)
    }
    /// The stream's opening status; its body follows as bounded pieces.
    pub fn refuse_stream(&mut self, ticket: u64, status: u16) -> Option<Report> {
        (self.stream_ticket() == Some(ticket)).then_some(Report::StreamRefused { status })
    }
}
