use jig_client_protocol_wire::v22 as wire;
use skein_io as io;
use skein_lib::{Decimal, Reader, Time, Token, Wall, Writer};
use temper_engine_service::Engine;
const REQUEST_AT: Time = Time::from_nanos(1);
const REQUEST_WALL: Wall = Wall::from_nanos(1);
pub const SOCKET: Token = Token::new(100);
const LISTENER: Token = Token::new(99);

pub struct Peer {
    wire: skein_world::stream::Wire,
    socket: Token,
    owner: Option<Token>,
    closed: u32,
    body_demand: bool,
}
impl Peer {
    pub fn new(engine: &mut Engine, method: &[u8], path: &[u8], body: &[u8], bearer: Option<&[u8]>) -> Self {
        let header =
            b" HTTP/1.1\r\nHost: 127.0.0.1:7100\r\nConnection: close\r\nContent-Type: application/octet-stream\r\n";
        let content = b"Content-Length: ";
        let length = Decimal::of(skein_lib::len_u64(body.len()));
        let authorization = match bearer {
            Some(value) => b"Authorization: Bearer ".len().checked_add(value.len()).unwrap().checked_add(2).unwrap(),
            None => 0,
        };
        let size = method
            .len()
            .checked_add(1)
            .unwrap()
            .checked_add(path.len())
            .unwrap()
            .checked_add(header.len())
            .unwrap()
            .checked_add(content.len())
            .unwrap()
            .checked_add(length.as_bytes().len())
            .unwrap()
            .checked_add(4)
            .unwrap()
            .checked_add(body.len())
            .unwrap()
            .checked_add(authorization)
            .unwrap();
        let mut bytes = Writer::new(size);
        bytes.put(method).unwrap();
        bytes.put(b" ").unwrap();
        bytes.put(path).unwrap();
        bytes
            .put(
                b" HTTP/1.1\r\nHost: 127.0.0.1:7100\r\nConnection: close\r\nContent-Type: application/octet-stream\r\n",
            )
            .unwrap();
        if let Some(bearer) = bearer {
            bytes.put(b"Authorization: Bearer ").unwrap();
            bytes.put(bearer).unwrap();
            bytes.put(b"\r\n").unwrap();
        }
        bytes.put(b"Content-Length: ").unwrap();
        bytes.put(Decimal::of(skein_lib::len_u64(body.len())).as_bytes()).unwrap();
        bytes.put(b"\r\n\r\n").unwrap();
        bytes.put(body).unwrap();
        Self::raw(engine, &bytes.finish())
    }
    pub fn raw(engine: &mut Engine, bytes: &[u8]) -> Self {
        Self::raw_on(engine, bytes, SOCKET)
    }
    pub fn raw_on(engine: &mut Engine, bytes: &[u8], socket: Token) -> Self {
        let mut wire = skein_world::stream::Wire::new(7);
        wire.write(bytes);
        assert!(engine.client_event(io::Event::Accepted {
            owner: Token::new(crate::tokens::SERVER),
            socket,
            peer: address(),
        }));
        Self { wire, socket, owner: None, closed: 0, body_demand: false }
    }

    pub fn pump(&mut self, engine: &mut Engine, decide: bool) {
        self.pump_at(engine, decide, REQUEST_AT, REQUEST_WALL);
    }
    pub fn pump_at(&mut self, engine: &mut Engine, decide: bool, now: Time, wall: Wall) {
        self.pump_until(engine, decide, now, wall, |peer, _| peer.streaming());
    }
    fn pump_until(
        &mut self,
        engine: &mut Engine,
        decide: bool,
        now: Time,
        wall: Wall,
        ready: fn(&Self, &Engine) -> bool,
    ) {
        for _ in 0_u32..256 {
            if decide {
                engine.iterate(now, wall);
            } else {
                engine.advance_clients(now, wall);
            }
            for _ in 0..engine.client_requests().len() {
                match engine.client_requests().pop().unwrap() {
                    io::Request::Listen { owner, addr } => {
                        assert!(engine.client_event(io::Event::Listening { owner, listener: LISTENER, addr }));
                    }
                    io::Request::Bind { socket, owner } => {
                        assert_eq!(socket, self.socket);
                        self.owner = Some(owner);
                    }
                    io::Request::Stream { stream, down } => {
                        assert_eq!(stream, self.socket);
                        self.body_demand = match &down {
                            skein_lib::stream::Down::Demand { read: skein_lib::stream::Read::Fill(_), .. } => true,
                            skein_lib::stream::Down::Demand { .. }
                            | skein_lib::stream::Down::Send(_)
                            | skein_lib::stream::Down::Finish => false,
                        };
                        self.wire.take(down);
                    }
                    io::Request::Close { .. } | io::Request::Abort { .. } => {
                        self.closed = self.closed.checked_add(1).unwrap();
                        assert!(engine.client_event(io::Event::Closed { owner: self.owner.unwrap() }));
                    }
                    io::Request::Connect { .. }
                    | io::Request::Reject { .. }
                    | io::Request::Output { .. }
                    | io::Request::Spawn { .. }
                    | io::Request::SpawnAlone { .. }
                    | io::Request::Signal { .. }
                    | io::Request::Usage { .. }
                    | io::Request::Window { .. } => panic!("unexpected client io"),
                }
            }
            if self.closed > 0 || ready(self, engine) {
                break;
            }
            if let Some(owner) = self.owner
                && let Some(up) = self.wire.answer()
            {
                assert!(engine.client_event(io::Event::Stream { owner, up }));
            }
        }
    }

    pub fn received(&self) -> &[u8] {
        &self.wire.received
    }
    pub fn sign_in_answer(&self, engine: &Engine) -> wire::SignInAnswer {
        let at = skein_lib::bytes::find(&self.wire.received, b"\r\n\r\n").unwrap().checked_add(4).unwrap();
        wire::SignInAnswer::decode(engine.client_wire(), &mut Reader::new(self.wire.received.get(at..).unwrap()))
            .unwrap()
    }
    pub fn streaming(&self) -> bool {
        skein_lib::bytes::find(&self.wire.received, b"text/event-stream").is_some()
    }
}
pub fn address() -> io::Addr {
    io::Addr::Ip((::core::net::Ipv4Addr::LOCALHOST, 7100).into())
}
pub fn link_body(engine: &Engine, link: &[u8]) -> Box<[u8]> {
    use skein_lib::base64::{self, Alphabet};
    let at = skein_lib::bytes::find(link, b"secret=").unwrap().checked_add(7).unwrap();
    let secret = base64::decode(Alphabet::Url, link.get(at..).unwrap(), 32).unwrap();
    let record =
        wire::LinkSignIn::new(engine.client_wire(), wire::LinkSignInParts { secret: skein_lib::Secret::new(secret) })
            .unwrap();
    let mut bytes = Writer::new(usize::try_from(record.measure()).unwrap());
    record.encode(&mut bytes).unwrap();
    bytes.finish()
}

pub fn bearer(value: &[u8]) -> Box<[u8]> {
    use skein_lib::base64::{self, Alphabet};
    let mut bytes = Writer::new(
        usize::try_from(base64::encoded_len(Alphabet::Url, u32::try_from(value.len()).unwrap()).unwrap()).unwrap(),
    );
    base64::encode(Alphabet::Url, value, &mut bytes).unwrap();
    bytes.finish()
}
