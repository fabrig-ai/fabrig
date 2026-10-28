use skein_io as io;
use skein_lib::{List, Map, Time, Token, Wall};

/// A forge's component face, with each connection's demand answered through Wire.
pub struct ForgeFace {
    peers: Map<Token, Peer>,
    selected: u16,
    answers: List<skein_forge_connection::Answered>,
    capture_answers: bool,
    now: Time,
    worker_owner: Option<Token>,
    hooks: Option<(Token, Token)>,
}
struct Peer {
    number: u16,
    socket: Token,
    wire: skein_world::stream::Wire,
}
impl ForgeFace {
    pub fn new() -> Self {
        Self {
            peers: Map::with_capacity(8),
            selected: 1,
            answers: List::with_capacity(8),
            capture_answers: false,
            now: Time::from_nanos(1),
            worker_owner: None,
            hooks: None,
        }
    }
    pub fn connected(&mut self, service: &mut crate::Service) {
        for _ in 0_u32..256 {
            self.turn(service);
            if self.sent().ends_with(b"\r\n\r\n") {
                return;
            }
        }
        panic!("start request sent");
    }
    pub fn sent(&self) -> &[u8] {
        for (_, peer) in &self.peers {
            if peer.number == self.selected {
                return &peer.wire.received;
            }
        }
        &[]
    }
    pub fn answer(&mut self, service: &mut crate::Service, bytes: &[u8]) {
        let mut owner = None;
        for (token, peer) in &self.peers {
            if peer.number == self.selected {
                owner = Some(*token);
                break;
            }
        }
        self.peers.get_mut(&owner.expect("selected forge connected")).unwrap().wire.write(bytes);
        for _ in 0_u32..512 {
            self.turn(service);
        }
    }
    pub fn select(&mut self, number: u16) {
        self.selected = number;
    }
    pub fn capture_answers(&mut self) {
        self.capture_answers = true;
    }
    pub fn take_answers(&mut self) -> Box<[skein_forge_connection::Answered]> {
        core::mem::replace(&mut self.answers, List::with_capacity(8)).into_boxed()
    }
    pub fn refuse(&mut self, service: &mut crate::Service) {
        for (owner, peer) in &self.peers {
            if peer.number == self.selected {
                service
                    .engine
                    .as_mut()
                    .expect("engine")
                    .forge
                    .up(io::Event::Failed { owner: *owner, error: io::Error::Refused });
                break;
            }
        }
    }
    pub fn at(&mut self, service: &mut crate::Service, now: Time) {
        self.now = now;
        self.turn(service);
    }
    pub fn turn(&mut self, service: &mut crate::Service) {
        let engine = service.engine.as_mut().expect("engine");
        for _ in 0_u32..engine.clients.io.len() {
            match engine.clients.io.pop().unwrap() {
                io::Request::Listen { owner, addr } => {
                    assert!(engine.clients.up(io::Event::Listening { owner, listener: Token::new(77), addr }));
                }
                io::Request::Close { .. } | io::Request::Abort { .. } => {
                    assert!(engine.clients.up(io::Event::Closed { owner: Token::new(crate::tokens::CLIENTS) }));
                }
                io::Request::Connect { .. }
                | io::Request::Bind { .. }
                | io::Request::Reject { .. }
                | io::Request::Stream { .. }
                | io::Request::Output { .. }
                | io::Request::Spawn { .. }
                | io::Request::Signal { .. }
                | io::Request::Usage { .. }
                | io::Request::Window { .. } => panic!("unexpected client listener request"),
            }
        }
        for _ in 0_u32..engine.workers.io.len() {
            match engine.workers.io.pop().unwrap() {
                io::Request::Listen { owner, addr } => {
                    self.worker_owner = Some(owner);
                    assert!(engine.workers.up(
                        self.now,
                        io::Event::Listening { owner, listener: Token::new(78), addr },
                        &mut engine.events
                    ));
                }
                io::Request::Close { .. } | io::Request::Abort { .. } => {
                    assert!(engine.workers.up(
                        self.now,
                        io::Event::Closed { owner: self.worker_owner.unwrap() },
                        &mut engine.events
                    ));
                }
                io::Request::Connect { .. }
                | io::Request::Bind { .. }
                | io::Request::Reject { .. }
                | io::Request::Stream { .. }
                | io::Request::Output { .. }
                | io::Request::Spawn { .. }
                | io::Request::Signal { .. }
                | io::Request::Usage { .. }
                | io::Request::Window { .. } => panic!("unexpected worker listener request"),
            }
        }
        for _ in 0..engine.forge.above.len() {
            let event = engine.forge.above.pop().unwrap();
            if self.capture_answers
                && let skein_forge_connection::Event::Answered(answer) = event
            {
                self.answers.push(answer).unwrap();
            } else {
                engine.forge.above.push(event);
            }
        }
        crate::engine::iterate(engine, self.now, Wall::EPOCH);
        self.forge_io(engine);
    }
    fn forge_io(&mut self, engine: &mut crate::engine::Engine) {
        for _ in 0_u32..engine.forge.io.len() {
            match engine.forge.io.pop().unwrap() {
                io::Request::Listen { owner, addr } => {
                    let listener = Token::new(79);
                    self.hooks = Some((owner, listener));
                    engine.forge.up(io::Event::Listening { owner, listener, addr });
                }
                io::Request::Connect { owner, addr } => {
                    let number = match addr {
                        io::Addr::Ip(address) => address.port().checked_sub(2999).expect("fixture forge port"),
                        io::Addr::InProcess(_) => panic!("forge socket"),
                    };
                    let socket = Token::new(owner.raw().checked_add(99).unwrap());
                    assert!(
                        self.peers
                            .insert(owner, Peer { number, socket, wire: skein_world::stream::Wire::new(7) })
                            .is_ok()
                    );
                    engine.forge.up(io::Event::Connecting { owner, socket });
                    engine.forge.up(io::Event::Connected { owner });
                }
                io::Request::Stream { stream, down } => {
                    let mut owner = None;
                    for (token, peer) in &self.peers {
                        if peer.socket == stream {
                            owner = Some(*token);
                            break;
                        }
                    }
                    self.peers.get_mut(&owner.expect("stream peer")).unwrap().wire.take(down);
                }
                io::Request::Close { entity } | io::Request::Abort { entity } => {
                    if let Some((owner, listener)) = self.hooks
                        && entity == listener
                    {
                        self.hooks = None;
                        engine.forge.up(io::Event::Closed { owner });
                        continue;
                    }
                    let mut owner = None;
                    for (token, peer) in &self.peers {
                        if peer.socket == entity || *token == entity {
                            owner = Some(*token);
                            break;
                        }
                    }
                    let owner = owner.expect("closing peer");
                    self.peers.remove(&owner);
                    engine.forge.up(io::Event::Closed { owner });
                }
                io::Request::Bind { .. }
                | io::Request::Reject { .. }
                | io::Request::Output { .. }
                | io::Request::Spawn { .. }
                | io::Request::Signal { .. }
                | io::Request::Usage { .. }
                | io::Request::Window { .. } => panic!("unexpected forge request"),
            }
        }
        let mut owners = List::with_capacity(self.peers.len());
        for (owner, _) in &self.peers {
            owners.push(*owner).unwrap();
        }
        for owner in &owners {
            let peer = self.peers.get_mut(owner).unwrap();
            if let Some(up) = peer.wire.answer() {
                engine.forge.up(io::Event::Stream { owner: *owner, up });
            }
        }
    }

    pub fn pending(service: &crate::Service) -> u32 {
        service.engine.as_ref().expect("engine").forge.above.len()
    }
    pub fn returned(service: &mut crate::Service, answer: skein_forge_connection::Answered) {
        let temper_engine_domain::Event::Forge(event) = crate::forge::event(answer) else {
            panic!("forge event");
        };
        crate::route::request(service.engine.as_mut().expect("engine"), temper_engine_domain::Request::Busy(event));
    }
}
/// Install a real listener connection and return its first verified hint.
pub fn hook(service: &mut crate::Service) -> skein_forge_connection::hooks::Hinted {
    use skein_lib::{Secret, Writer};
    const SECRET: &[u8] = b"fixture-hook-secret";
    const BODY: &[u8] = b"{\"repository\":{\"id\":2},\"number\":7}";
    let config = super::engine();
    let mut found = jig_protocol_config::Problems::empty();
    let mut settings =
        crate::forge::settings(&config.forges, super::forge_runtime(), None, &mut found).expect("settings");
    settings.forges.get_mut(0).expect("forge").access.as_mut().expect("access").webhook_secret =
        Some(Secret::new(Box::from(SECRET)));
    settings.webhooks =
        Some(skein_forge_connection::hooks::Webhooks { listener: io::Addr::Ip(([127, 0, 0, 1], 7200).into()) });
    let layer = &mut service.engine.as_mut().expect("engine").forge;
    layer.component = skein_forge_connection::Component::new(
        settings,
        &layer.env.limits,
        &crate::forge::shared(&config.limits, &config.jig.limits.io),
    )
    .expect("component");
    while layer.io.pop().is_some() {}
    layer.component.begin(&layer.env, &mut layer.above, &mut layer.io);
    let listener = Token::new(1_u64 << 59_u32);
    let mut owner = None;
    let mut wire = skein_world::stream::Wire::new(7);
    let mut mac = skein_lib::sha256::Hmac::new(SECRET);
    mac.update(BODY);
    let mut head = Writer::new(180);
    head.put(b"POST /hooks/1 HTTP/1.1\r\nHost: loopback\r\nContent-Length: 34\r\nX-Forgejo-Event: pull_request\r\nX-Forgejo-Signature: ").expect("head");
    for byte in mac.finish() {
        for nibble in [byte / 16, byte % 16] {
            head.put(&[b"0123456789abcdef"[usize::from(nibble)]]).expect("signature");
        }
    }
    head.put(b"\r\n\r\n").expect("head");
    wire.write(&head.finish());
    wire.write(BODY);
    layer.up(io::Event::Listening {
        owner: listener,
        listener: Token::new(88),
        addr: io::Addr::Ip(([127, 0, 0, 1], 7200).into()),
    });
    layer.up(io::Event::Accepted {
        owner: listener,
        socket: Token::new(89),
        peer: io::Addr::Ip(([127, 0, 0, 1], 9000).into()),
    });
    for _ in 0_u32..2048 {
        while let Some(request) = layer.io.pop() {
            match request {
                io::Request::Bind { owner: bound, .. } => owner = Some(bound),
                io::Request::Stream { stream, down } if stream == Token::new(89) => wire.take(down),
                io::Request::Listen { .. } | io::Request::Connect { .. } | io::Request::Stream { .. } => {}
                io::Request::Close { .. }
                | io::Request::Abort { .. }
                | io::Request::Reject { .. }
                | io::Request::Output { .. }
                | io::Request::Spawn { .. }
                | io::Request::Signal { .. }
                | io::Request::Usage { .. }
                | io::Request::Window { .. } => panic!("hook delivery only"),
            }
        }
        if let Some(up) = wire.answer() {
            layer.up(io::Event::Stream { owner: owner.expect("bound hook"), up });
        }
        if let Some(skein_forge_connection::Event::Hint(hint)) = layer.above.pop() {
            return hint;
        }
    }
    panic!("verified hint");
}
