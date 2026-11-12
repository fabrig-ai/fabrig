//! Forge HTTP in the session's existing pump; only this adapter reads the header.
use super::super::{Event, forge::RESPONSE_BYTES};
use skein_http::{self as http, client};
use skein_io as io;
use skein_lib::{
    Env, Queue, Time, Token, Wall,
    stream::{Down, Read, Up},
};
use std::{collections::VecDeque, fs, net::SocketAddr, path::Path};
const LIMITS: client::Limits = client::Limits { request: 16384, head: 16384, headers: 64, read: 16384, send: 16384 };

pub(in crate::drive) fn read_log(path: &Path) -> Result<Box<[u8]>, String> {
    fs::read(path).map(Vec::into_boxed_slice).map_err(|why| format!("{}: {why}", path.display()))
}
pub(in crate::drive) struct Prepared {
    pub call: client::Call,
    pub address: SocketAddr,
    body: Box<[u8]>,
    token: Box<[u8]>,
}
pub(in crate::drive) fn prepare(
    api: &[u8],
    header: &Path,
    method: http::Method,
    path: &[u8],
    body: Box<[u8]>,
) -> Result<Prepared, String> {
    let api = std::str::from_utf8(api).map_err(|_| "forge API is not UTF-8")?;
    let authority = api
        .strip_prefix("http://")
        .and_then(|rest| rest.strip_suffix("/api/v1"))
        .ok_or("forge API needs http://ADDRESS/api/v1")?;
    let address = authority.parse().map_err(|_| "forge API needs an IP address and port")?;
    let line = fs::read(header).map_err(|why| format!("{}: {why}", header.display()))?;
    let line = line.strip_suffix(b"\n").unwrap_or(&line);
    let token = line
        .strip_prefix(b"Authorization: token ")
        .filter(|value| !value.is_empty() && value.iter().all(u8::is_ascii_graphic))
        .ok_or_else(|| format!("{}: expected one Authorization: token header line", header.display()))?;
    let call = client::Call {
        method,
        target: [b"/api/v1".as_slice(), path].concat().into(),
        headers: Box::new([
            http::Header { name: b"Host".as_slice().into(), value: authority.as_bytes().into() },
            http::Header {
                name: b"Authorization".as_slice().into(),
                value: [b"token ".as_slice(), token].concat().into(),
            },
            http::Header { name: b"Content-Type".as_slice().into(), value: b"application/json".as_slice().into() },
        ]),
        body: if body.is_empty() {
            client::Body::None
        } else {
            client::Body::Length(u64::try_from(body.len()).expect("body length"))
        },
        close: true,
    };
    Ok(Prepared { call, address, body, token: token.into() })
}
/// One HTTP exchange, driven by the same io reports as the children.
/// Closing sockets remain here until their terminal report, even after cancellation.
pub(in crate::drive) struct Connection {
    owner: Token,
    socket: Option<Token>,
    closing: bool,
    pub settled: bool,
    cancelled: bool,
    call: Option<client::Call>,
    client: client::Client,
    above: Queue<client::Event>,
    below: Queue<Down>,
    upload: VecDeque<Box<[u8]>>,
    status: Option<u16>,
    body: Vec<u8>,
    token: Box<[u8]>,
    result: Option<Event>,
}
impl Connection {
    pub fn start(owner: Token, prepared: Prepared) -> (Self, Vec<io::Request>) {
        let request = io::Request::Connect { owner, addr: io::Addr::Ip(prepared.address) };
        (
            Self {
                owner,
                socket: None,
                closing: false,
                settled: false,
                cancelled: false,
                call: Some(prepared.call),
                client: client::Client::new(&LIMITS),
                above: Queue::uncapped(),
                below: Queue::uncapped(),
                upload: prepared.body.chunks(16384).map(Box::from).collect(),
                status: None,
                body: vec![],
                token: prepared.token,
                result: None,
            },
            vec![request],
        )
    }
    pub fn owns(&self, owner: Token) -> bool {
        self.owner == owner
    }
    pub fn cancel(&mut self) -> Vec<io::Request> {
        self.cancelled = true;
        self.result = None;
        self.close()
    }
    fn close(&mut self) -> Vec<io::Request> {
        if !self.closing
            && let Some(entity) = self.socket
        {
            self.closing = true;
            return vec![io::Request::Close { entity }];
        }
        vec![]
    }
    pub fn event(&mut self, event: io::Event, now: Time) -> (Option<Event>, Vec<io::Request>) {
        let env = Env { now, wall: Wall::EPOCH, limits: LIMITS };
        let mut requests = vec![];
        match event {
            io::Event::Connecting { socket, .. } => {
                self.socket = Some(socket);
                if self.cancelled {
                    requests.extend(self.close());
                }
            }
            io::Event::Connected { .. } if !self.cancelled => {
                let call = self.call.take().expect("one forge call");
                client::down(&mut self.client, &env, client::Request::Call(call), &mut self.above, &mut self.below);
                if !self.upload.is_empty() {
                    self.upload_demand(&env);
                }
            }
            io::Event::Stream { up, .. } if !self.closing => {
                client::up(&mut self.client, &env, up, &mut self.above, &mut self.below);
            }
            io::Event::Failed { error, .. } => {
                if !self.cancelled {
                    self.result = Some(Event::ForgeFailure(format!("forge HTTP connection: {error:?}")));
                }
                if self.socket.is_none() {
                    self.settled = true;
                }
            }
            io::Event::Closed { .. } => {
                self.socket = None;
                self.settled = true;
                if !self.cancelled && self.result.is_none() {
                    self.result = Some(Event::ForgeFailure("forge HTTP connection closed".into()));
                }
            }
            _ => {}
        }
        while let Some(event) = self.above.pop() {
            if self.cancelled || self.result.is_some() {
                continue;
            }
            match event {
                client::Event::Response(response) => {
                    self.status = Some(response.status);
                    self.body_demand(&env);
                }
                client::Event::Upload(Up::Room) => {
                    let bytes = self.upload.pop_front().expect("upload room requested");
                    client::down(
                        &mut self.client,
                        &env,
                        client::Request::Upload(Down::Send(bytes)),
                        &mut self.above,
                        &mut self.below,
                    );
                    if self.upload.is_empty() {
                        client::down(
                            &mut self.client,
                            &env,
                            client::Request::Upload(Down::Finish),
                            &mut self.above,
                            &mut self.below,
                        );
                    } else {
                        self.upload_demand(&env);
                    }
                }
                client::Event::Body(Up::Bytes(bytes)) => {
                    let refused = self.status.is_some_and(|status| !(200..300).contains(&status));
                    let cap = if refused { 4096 } else { usize::try_from(RESPONSE_BYTES).expect("response bound") };
                    let left = cap.saturating_sub(self.body.len());
                    self.body.extend_from_slice(bytes.get(..left).unwrap_or(&bytes));
                    if refused && self.body.len() == cap {
                        self.response();
                    } else if bytes.len() > left {
                        self.result = Some(Event::ForgeFailure("forge response exceeds byte bound".into()));
                    } else {
                        self.body_demand(&env);
                    }
                }
                client::Event::Done(_) => self.response(),
                client::Event::Failed(why) => self.result = Some(Event::ForgeFailure(format!("forge HTTP: {why:?}"))),
                client::Event::Body(Up::Failed(why)) => {
                    self.result = Some(Event::ForgeFailure(format!("forge HTTP body: {why:?}")));
                }
                client::Event::Upload(Up::Failed(_)) | client::Event::Body(Up::End) | client::Event::Closed => {}
                client::Event::Upload(Up::Bytes(_) | Up::End) | client::Event::Body(Up::Room) => {
                    unreachable!("HTTP stream direction")
                }
            }
        }
        if self.result.is_some() || self.cancelled {
            requests.extend(self.close());
        }
        while let Some(down) = self.below.pop() {
            if !self.closing {
                requests.push(io::Request::Stream { stream: self.socket.expect("forge socket"), down });
            }
        }
        (if self.settled { self.result.take() } else { None }, requests)
    }
    fn upload_demand(&mut self, env: &Env<client::Limits>) {
        let room = u32::try_from(self.upload.front().expect("upload bytes").len()).expect("upload chunk");
        client::down(
            &mut self.client,
            env,
            client::Request::Upload(Down::Demand { read: Read::Nothing, room }),
            &mut self.above,
            &mut self.below,
        );
    }
    fn body_demand(&mut self, env: &Env<client::Limits>) {
        client::down(
            &mut self.client,
            env,
            client::Request::Body(Down::Demand { read: Read::Arrived { max: LIMITS.read }, room: 0 }),
            &mut self.above,
            &mut self.below,
        );
    }
    fn response(&mut self) {
        let bytes = redact(&self.body, &self.token);
        self.result = Some(Event::ForgeResponse { status: self.status.expect("response head"), body: bytes });
    }
}
fn redact(body: &[u8], token: &[u8]) -> Box<[u8]> {
    // JSON escapes can hide a credential on the wire and reveal it in an answer.
    // Such a document is withheld whole before the policy can decode its fields.
    let json = skein_json::tokenizer::Limits {
        depth: 32,
        string: RESPONSE_BYTES,
        number: 32,
        chunk: 4096,
        length: RESPONSE_BYTES,
    };
    if let Ok(doc) =
        skein_json::parse(body, &json, &skein_json::document::Limits { tokens: RESPONSE_BYTES, text: RESPONSE_BYTES })
        && (0..doc.len())
            .filter_map(|index| doc.token(index).and_then(|record| doc.text(record)))
            .any(|text| text.windows(token.len()).any(|part| part == token))
    {
        return b"[redacted]".as_slice().into();
    }
    let mut out = vec![];
    let mut rest = body;
    while let Some(at) = rest.windows(token.len()).position(|part| part == token) {
        out.extend_from_slice(&rest[..at]);
        out.extend_from_slice(b"[redacted]");
        rest = &rest[at + token.len()..];
    }
    out.extend_from_slice(rest);
    out.into()
}
