//! One missing call's wire spelling, decoding and terminal git diagnostic.
use crate::{
    Answer,
    connection::{Failed, Reply},
};
use skein_fake_forge_domain::api;
use skein_fake_forge_protocol::{
    Limits,
    git::{self, Call, Name, client},
};
use skein_http::{Header, Method, client as http};
use skein_json::tokenizer;
use skein_lib::{
    Env, Queue, Secret, Time, Wall,
    stream::{Down, Held},
};

#[derive(Debug)]
pub(crate) struct Exchange {
    call: Call,
    remote: Vec<u8>,
    host: Vec<u8>,
    pub address: std::net::SocketAddr,
}
pub(crate) type Received = (Call, Result<git::Answer, api::Error>, bool);
impl Exchange {
    pub fn new(call: Call, remote: Vec<u8>) -> Result<Self, Answer> {
        let rest = remote.strip_prefix(b"http://").or_else(|| remote.strip_prefix(b"https://")).unwrap_or(&remote);
        let host = rest.split(|b| *b == b'/').next().unwrap_or(b"").to_vec();
        let address = std::str::from_utf8(&host).ok().and_then(|host| host.parse().ok());
        match address {
            Some(address) => Ok(Self { call, remote, host, address }),
            None => {
                Err(fatal(format!("unable to access '{}': Could not resolve host: {}", text(&remote), text(&host))))
            }
        }
    }
    pub fn request(&self, authorization: Option<&Secret>, limits: &Limits) -> Result<(http::Call, Box<[u8]>), Answer> {
        let encoded = client::encode(&self.call, limits).map_err(|_| broken("request"))?;
        let mut headers = vec![
            Header { name: b"Host".as_slice().into(), value: self.host.clone().into() },
            Header { name: b"Content-Type".as_slice().into(), value: b"application/json".as_slice().into() },
        ];
        if self.call.name().repository()
            && let Some(authorization) = authorization
        {
            headers.push(Header { name: b"Authorization".as_slice().into(), value: authorization.expose().into() });
        }
        let length = u64::try_from(encoded.body.len()).expect("bounded JSON body");
        Ok((
            http::Call {
                method: Method::Post,
                target: encoded.target,
                headers: headers.into(),
                body: http::Body::Length(length),
                close: false,
            },
            encoded.body,
        ))
    }
    pub fn finish(self, reply: Result<Reply, Failed>, limits: &Limits) -> Result<Received, Answer> {
        let reply = match reply {
            Ok(reply) => reply,
            Err(Failed) if can_fault(self.call.name()) => return Ok((self.call, Err(api::Error::Unreachable), true)),
            Err(Failed) => {
                return Err(fatal(format!(
                    "unable to access '{}': Failed to connect to {} port {}: Couldn't connect to server",
                    text(&self.remote),
                    self.address.ip(),
                    self.address.port()
                )));
            }
        };
        let decoded = decode(self.call.name(), &reply, limits);
        let Some((answer, named)) = decoded else {
            return Err(self.no_route());
        };
        if !named && matches!(reply.status, 404 | 405) {
            return Err(self.no_route());
        }
        // An absent branch is branch()'s ordinary None, needed before a creation.
        let absent = self.call.name() == Name::Branch && answer == Err(api::Error::Missing(api::What::Branch));
        if answer.is_err() && !can_fault(self.call.name()) && !absent {
            return Err(fatal(format!(
                "unable to access '{}': The requested URL returned error: {}",
                text(&self.remote),
                reply.status
            )));
        }
        if matches!(self.call.name(), Name::Merge | Name::CommitTree) && answer == Ok(git::Answer::Stored(None)) {
            return Err(self.no_route());
        }
        Ok((self.call, answer, reply.status != 401 || named))
    }
    fn no_route(&self) -> Answer {
        fatal(format!("{} serves no fake git route", text(&self.host)))
    }
}
fn can_fault(name: Name) -> bool {
    matches!(name, Name::Clone | Name::Fetch | Name::Create | Name::Push)
}
fn fatal(mut message: String) -> Answer {
    message.insert_str(0, "fatal: ");
    message.push('\n');
    Answer { exit: 128, stdout: Vec::new(), stderr: message.into_bytes() }
}
pub(crate) fn broken(rule: &str) -> Answer {
    Answer { exit: 125, stdout: Vec::new(), stderr: format!("temper fake git: broken: {rule}\n").into_bytes() }
}
fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}
fn decode(name: Name, reply: &Reply, limits: &Limits) -> Option<(Result<git::Answer, api::Error>, bool)> {
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.documents.json };
    let mut json = tokenizer::Tokenizer::new(&env.limits);
    let mut decoder = client::decoder(name, reply.status, Time::ZERO, limits);
    let mut above = Queue::with_capacity(2);
    let mut below = Queue::with_capacity(2);
    let mut held = Held::new(reply.body.clone().into());
    tokenizer::down(&mut json, &env, client::first(&decoder), &mut above, &mut below);
    // A token/demand consumes bytes or advances a finite JSON container.
    for _ in 0..reply.body.len().saturating_mul(4).saturating_add(32) {
        if let Some(event) = above.pop() {
            match client::step(&mut decoder, limits, &event) {
                client::Step::Demand(request) => tokenizer::down(&mut json, &env, request, &mut above, &mut below),
                client::Step::Decoded(answer) => return Some((answer, client::named(&decoder))),
                client::Step::Malformed => return None,
            }
        } else if let Some(Down::Demand { read, .. }) = below.pop() {
            let event = held.answer(read)?;
            tokenizer::up(&mut json, &env, event, &mut above, &mut below);
        } else {
            return None;
        }
    }
    None
}
