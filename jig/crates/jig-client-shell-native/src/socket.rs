//! One socket, optionally carrying TLS (protocol/client/shells.md, 1).
use crate::Engine;
use skein_io as io;
use skein_lib::{
    Env, Queue, Token,
    stream::{Down, Up},
};
use skein_tls::client as tls;
use std::collections::VecDeque;

#[derive(Clone, Copy)]
enum Phase {
    Connecting,
    Handshaking,
    Ready,
    Closing,
    Ending,
}

pub(crate) struct Socket {
    // Kept through graceful close so host shutdown can still abort the socket.
    pub entity: Option<Token>,
    pub started: bool,
    pub closed: bool,
    pub retry: bool,
    pub attempt: usize,
    phase: Phase,
    pub below: VecDeque<Down>,
    pub above: Queue<tls::Event>,
    cipher: Queue<Down>,
    tls: Option<tls::Client>,
}
impl Socket {
    pub fn new(engine: &Engine, limits: &tls::Limits) -> Self {
        Self {
            entity: None,
            started: false,
            closed: false,
            retry: false,
            attempt: 0,
            phase: Phase::Connecting,
            below: VecDeque::new(),
            above: Queue::uncapped(),
            cipher: Queue::uncapped(),
            tls: match engine {
                Engine::Plain(_) => None,
                Engine::Tls(tls) => Some(tls::Client::new(&tls.config, tls.name.clone(), limits)),
            },
        }
    }
    pub fn next(&mut self, engine: &Engine, limits: &tls::Limits) {
        let attempt = self.attempt.checked_add(1).expect("bounded addresses");
        let below = std::mem::take(&mut self.below);
        *self = Self::new(engine, limits);
        self.attempt = attempt;
        self.below = below;
    }
    pub fn ready(&self) -> bool {
        matches!(self.phase, Phase::Ready)
    }
    pub fn readied(&mut self) {
        self.phase = Phase::Ready;
    }
    pub fn connected(&mut self, env: &Env<tls::Limits>) {
        if matches!(self.phase, Phase::Closing | Phase::Ending) {
            return;
        }
        self.phase = Phase::Handshaking;
        match &mut self.tls {
            Some(client) => tls::down(client, env, tls::Request::Handshake, &mut self.above, &mut self.cipher),
            None => self.readied(),
        }
    }
    pub fn up(&mut self, engine: &Engine, env: &Env<tls::Limits>, up: Up) {
        if matches!(self.phase, Phase::Closing | Phase::Ending) {
            return;
        }
        if self.tls.is_some() && !self.ready() && matches!(up, Up::Failed(_) | Up::End) {
            self.phase = Phase::Ending;
            self.cipher.clear();
            self.above.clear();
            self.retry = engine.address(self.attempt.checked_add(1).expect("bounded addresses")).is_some();
            if !self.retry {
                self.above.push(tls::Event::Stream(Up::Failed(skein_lib::stream::Fault::Reset)));
            }
            return;
        }
        match &mut self.tls {
            Some(client) => tls::up(client, env, up, &mut self.above, &mut self.cipher),
            None => self.above.push(tls::Event::Stream(up)),
        }
    }
    pub fn pending(&self) -> bool {
        !self.above.is_empty()
            || !self.cipher.is_empty()
            || self.ready() && !self.below.is_empty()
            || matches!(self.phase, Phase::Ending) && self.entity.is_some()
            || self.below.iter().any(|down| matches!(down, Down::Finish))
    }
    pub fn pump(&mut self, env: &Env<tls::Limits>, requests: &mut Queue<io::Request>) {
        if matches!(self.phase, Phase::Closing) {
            return;
        }
        let finish = self.below.iter().any(|down| matches!(down, Down::Finish));
        if matches!(self.phase, Phase::Ending) || !self.ready() && finish {
            if finish {
                self.retry = false;
                self.below.clear();
            }
            self.phase = Phase::Ending;
            self.cipher.clear();
            if let Some(entity) = self.entity.take() {
                requests.push(io::Request::Abort { entity });
            }
            return;
        }
        if matches!(self.phase, Phase::Connecting) {
            return;
        }
        let stream = self.entity.expect("connected socket");
        while self.ready() {
            let Some(down) = self.below.pop_front() else { break };
            match (&mut self.tls, down) {
                (Some(client), Down::Finish) => {
                    tls::down(client, env, tls::Request::Close, &mut self.above, &mut self.cipher);
                    self.phase = Phase::Closing;
                    break;
                }
                (Some(client), down @ (Down::Demand { .. } | Down::Send(_))) => {
                    tls::down(client, env, tls::Request::Stream(down), &mut self.above, &mut self.cipher);
                }
                (None, Down::Finish) => {
                    requests.push(io::Request::Close { entity: stream });
                    self.phase = Phase::Closing;
                    break;
                }
                (None, down @ (Down::Demand { .. } | Down::Send(_))) => {
                    requests.push(io::Request::Stream { stream, down });
                }
            }
        }
        while let Some(down) = self.cipher.pop() {
            requests.push(io::Request::Stream { stream, down });
        }
        if matches!(self.phase, Phase::Closing) && self.tls.is_some() {
            requests.push(io::Request::Close { entity: stream });
        }
    }
}
