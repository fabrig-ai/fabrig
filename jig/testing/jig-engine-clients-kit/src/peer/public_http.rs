//! HTTP bytes through the mapping and origin policy, with scripted component answers
//! (jig/docs/design/protocol/client/default-http.md, 5 and jig/docs/design/protocol/client/public-reads.md).
use super::{Exchange, ExchangeState, Peer, Token, fixtures, http, mapping};
use jig_protocol_engine_clients::{FromTransport, ToTransport};
use jig_test_service::server::Routed;
use skein_http::{Header, Method, server};
use skein_lib::{Queue, Time};
use skein_world::stream::Wire;

impl Peer {
    /// Sends an HTTP call over wires seeded 1 and 2, leaving the component scripted.
    pub fn public_http(
        &mut self,
        method: Method,
        target: &[u8],
        headers: Box<[Header]>,
    ) -> (Token, Vec<FromTransport>) {
        let mut exchange = Exchange {
            owner: None,
            client: http::Client::new(&fixtures::client_limits()),
            events: Queue::with_capacity(8),
            below: Queue::with_capacity(8),
            to_engine: Wire::new(1),
            to_client: Wire::new(2),
            sent_engine: 0,
            sent_client: 0,
            upload: None,
            body: Vec::new(),
            status: None,
            done: false,
            streaming: false,
            paused: false,
            pause_after: None,
            state: ExchangeState::Active,
        };
        exchange.down(http::Request::Call(http::Call {
            method,
            target: Box::from(target),
            headers,
            body: http::Body::None,
            close: false,
        }));
        let socket = Token::new(self.next_socket);
        self.next_socket += 1;
        self.sockets.insert(socket, exchange);
        self.server.up(
            skein_io::Event::Accepted { owner: Token::new(0), socket, peer: fixtures::address() },
            &mut self.server_out,
        );
        (socket, self.public_http_pump())
    }

    /// Supplies a scripted component answer at the mapping's monotonic clock.
    pub fn public_http_answer(&mut self, now: Time, answer: ToTransport) {
        self.env.now = now;
        self.mount_event(mapping::Event::Component(answer));
        assert!(self.public_http_pump().is_empty(), "wire seeds 1,2: no extra component call");
    }

    /// The complete HTTP answer bytes, including the server's policy and framing.
    #[must_use]
    pub fn public_http_bytes(&self, socket: Token) -> &[u8] {
        let exchange = self.sockets.get(&socket).expect("wire seeds 1,2: socket");
        assert!(exchange.done, "wire seeds 1,2: answer complete");
        &exchange.to_engine.received
    }

    fn public_http_route(&mut self, routed: Routed) {
        match routed {
            Routed::Io(request) => self.io(request),
            Routed::Call { call, head } => {
                if self.mount.room_for(&head) > 0 {
                    self.mount_event(mapping::Event::Call { call, head });
                } else {
                    self.refused.insert(call, ()).expect("bounded server refusals");
                    self.server.answer(call, server::Request::Discard, &mut self.server_out);
                    self.server.answer(call, server::Request::Respond(self.mount.refusal(&head)), &mut self.server_out);
                }
            }
            Routed::Http { call, event } => {
                if self.refused.get(&call).is_some() {
                    match event {
                        server::Event::Done(_) | server::Event::Failed(_) | server::Event::Closed => {
                            self.refused.remove(&call);
                            self.server.done(call, &mut self.server_out);
                        }
                        server::Event::Call(_)
                        | server::Event::Ended
                        | server::Event::Body(_)
                        | server::Event::Reply(_)
                        | server::Event::Refused(_) => {}
                    }
                } else {
                    self.mount_event(mapping::Event::Http { call, event });
                }
            }
        }
    }

    fn public_http_pump(&mut self) -> Vec<FromTransport> {
        let mut messages = Vec::new();
        let mut quiet = 0;
        for _ in 0..4096 {
            self.server.clock(self.env.now, self.env.wall);
            self.server.fire(&mut self.server_out);
            let mut active = self.mount.up(&self.mount_env(), &mut self.mount_out);
            while let Some(request) = self.mount_out.pop() {
                active = true;
                match request {
                    mapping::Request::Http { call, request } => self.server.answer(call, request, &mut self.server_out),
                    mapping::Request::Component(message) => messages.push(message),
                }
            }
            active |= self.server.advance(&mut self.server_out);
            if let Some(routed) = self.server_out.pop() {
                active = true;
                self.public_http_route(routed);
            }
            let mut ups = Vec::new();
            for exchange in self.sockets.values_mut() {
                if let Some(owner) = exchange.owner {
                    let (up, changed) = exchange.advance();
                    active |= changed;
                    if let Some(up) = up {
                        active = true;
                        ups.push((owner, up));
                    }
                }
            }
            for (owner, up) in ups {
                self.server.up(skein_io::Event::Stream { owner, up }, &mut self.server_out);
            }
            quiet = if active { 0 } else { quiet + 1 };
            if quiet == 16 {
                return messages;
            }
        }
        panic!("wire seeds 1,2: HTTP mapping did not settle");
    }
}
