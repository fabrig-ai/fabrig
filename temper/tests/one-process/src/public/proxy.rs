//! The caching proxy's contract (temper/docs/design/protocol/sign-in.md, 2.4).
use super::{
    http::{self, Answer, header},
    wire::{Event, Wire},
};
use skein_http::Method;
use skein_io::kernel::{Complete, Submit};
use skein_lib::{Queue, Time, Wall};
use skein_shell::Host;
use std::{
    collections::BTreeMap,
    net::{IpAddr, SocketAddr},
};

pub(crate) struct ProxyConfig {
    pub upstreams: Vec<Upstream>,
    pub per_address: PerAddress,
    pub public_in_flight: u32,
    pub public_pool: u32,
    pub other_pool: u32,
}
pub(crate) struct Upstream {
    pub names: Vec<Box<[u8]>>,
    pub address: SocketAddr,
}
pub(crate) struct PerAddress {
    pub reads_per_second: u32,
    pub reads_in_flight: u32,
    pub anonymous_per_second: u32,
}
#[derive(Clone, Copy, Default, Debug)]
pub(crate) struct ProxyCounts {
    pub upstream_public: u32,
    pub upstream_other: u32,
    pub served_from_cache: u32,
    pub refused_host: u32,
    pub busy: u32,
}
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Key {
    upstream: usize,
    host: Box<[u8]>,
    path: Box<[u8]>,
}
struct Copy {
    answer: Answer,
    received: Time,
    age: u64,
    max_age: u64,
}
#[derive(Default)]
struct Address {
    second: u64,
    reads: u32,
    anonymous: u32,
    in_flight: u32,
}
struct Flight {
    downstream: u64,
    address: IpAddr,
    public: bool,
    key: Key,
}
pub(crate) struct Proxy {
    pub(super) config: ProxyConfig,
    pub counts: ProxyCounts,
    pub forwarded: Vec<Box<[u8]>>,
    wire: Wire,
    copies: BTreeMap<Key, Copy>,
    addresses: BTreeMap<IpAddr, Address>,
    flights: BTreeMap<u64, Flight>,
    public: u32,
    other: u32,
}
impl Proxy {
    pub fn new(address: SocketAddr, config: ProxyConfig) -> Self {
        Self {
            config,
            counts: ProxyCounts::default(),
            forwarded: Vec::new(),
            wire: Wire::new(Some(address)),
            copies: BTreeMap::new(),
            addresses: BTreeMap::new(),
            flights: BTreeMap::new(),
            public: 0,
            other: 0,
        }
    }
    pub fn ready(&self) -> bool {
        self.wire.ready
    }
    pub fn stop(&mut self) {
        self.wire.stop();
    }
    pub fn idle(&self) -> bool {
        self.wire.idle()
    }
    fn busy(&mut self, downstream: u64) {
        self.counts.busy += 1;
        let mut answer = Answer::empty(503);
        let mut headers = answer.headers.into_vec();
        headers.push(header(b"Retry-After", b"1"));
        answer.headers = headers.into();
        self.wire.respond(downstream, answer.wire());
    }
    fn request(&mut self, id: u64, peer: SocketAddr, bytes: &[u8], now: Time) {
        let (call, end) = http::request(bytes).expect("complete downstream request");
        let host = call.header(b"host").expect("HTTP/1.1 host");
        let Some(upstream) =
            self.config.upstreams.iter().position(|u| u.names.iter().any(|name| name.as_ref() == host))
        else {
            self.counts.refused_host += 1;
            self.wire.respond(id, Answer::empty(421).wire());
            return;
        };
        let public = call.method == Method::Get && call.target.starts_with(b"/jig/v1/public/");
        let credential = call.headers.iter().any(|h| h.is(b"cookie") || h.is(b"authorization"));
        let address = self.addresses.entry(peer.ip()).or_default();
        let second = now.as_nanos() / 1_000_000_000;
        if second != address.second {
            address.second = second;
            address.reads = 0;
            address.anonymous = 0;
        }
        if public {
            if address.reads >= self.config.per_address.reads_per_second
                || address.in_flight >= self.config.per_address.reads_in_flight
                || self.public >= self.config.public_in_flight
            {
                self.busy(id);
                return;
            }
            address.reads += 1;
        } else if !credential {
            if address.anonymous >= self.config.per_address.anonymous_per_second {
                self.busy(id);
                return;
            }
            address.anonymous += 1;
        }
        let key = Key { upstream, host: host.into(), path: call.target.clone() };
        if public && let Some(copy) = self.copies.get(&key) {
            let age = copy.age + now.as_nanos().saturating_sub(copy.received.as_nanos()) / 1_000_000_000;
            if age < copy.max_age {
                let mut answer = copy.answer.clone();
                let mut headers: Vec<_> = answer.headers.into_vec().into_iter().filter(|h| !h.is(b"age")).collect();
                headers.push(header(b"Age", age.to_string().as_bytes()));
                answer.headers = headers.into();
                self.counts.served_from_cache += 1;
                self.wire.respond(id, answer.wire());
                return;
            }
        }
        self.copies.remove(&key);
        if (public && self.public >= self.config.public_pool) || (!public && self.other >= self.config.other_pool) {
            self.busy(id);
            return;
        }
        let mut forward = call.method.as_bytes().to_vec();
        forward.extend_from_slice(b" ");
        forward.extend_from_slice(&call.target);
        forward.extend_from_slice(b" HTTP/1.1\r\n");
        for h in &call.headers {
            let lower = h.name.to_ascii_lowercase();
            if public
                && (h.is(b"cookie")
                    || h.is(b"authorization")
                    || h.is(b"forwarded")
                    || lower.starts_with(b"x-forwarded-"))
            {
                continue;
            }
            forward.extend_from_slice(&h.name);
            forward.extend_from_slice(b": ");
            forward.extend_from_slice(&h.value);
            forward.extend_from_slice(b"\r\n");
        }
        forward.extend_from_slice(b"\r\n");
        forward.extend_from_slice(
            &bytes[end
                - match call.body {
                    skein_http::server::Body::None => 0,
                    skein_http::server::Body::Length(n) => usize::try_from(n).expect("bounded request body"),
                    skein_http::server::Body::Chunked => unreachable!("world requests use Content-Length"),
                }..end],
        );
        self.forwarded.push(forward.clone().into());
        let connection = self.wire.start(self.config.upstreams[upstream].address, call.method, forward.into());
        if public {
            self.public += 1;
            self.addresses.get_mut(&peer.ip()).expect("client address").in_flight += 1;
            self.counts.upstream_public += 1;
        } else {
            self.other += 1;
            self.counts.upstream_other += 1;
        }
        self.flights.insert(connection, Flight { downstream: id, address: peer.ip(), public, key });
    }
    fn answered(&mut self, id: u64, answer: Option<Answer>, now: Time) {
        let flight = self.flights.remove(&id).expect("upstream flight");
        if flight.public {
            self.public -= 1;
            self.addresses.get_mut(&flight.address).expect("client address").in_flight -= 1;
        } else {
            self.other -= 1;
        }
        let answer = answer.unwrap_or_else(|| Answer::empty(502));
        if flight.public
            && let Some((age, max_age)) = cacheable(&answer)
        {
            self.copies.insert(flight.key, Copy { answer: answer.clone(), received: now, age, max_age });
        }
        self.wire.respond(flight.downstream, answer.wire());
    }
}
fn cacheable(answer: &Answer) -> Option<(u64, u64)> {
    if !matches!(answer.status, 200 | 404) {
        return None;
    }
    let controls: Vec<_> = answer.headers.iter().filter(|h| h.is(b"cache-control")).collect();
    let [control] = controls.as_slice() else {
        return None;
    };
    let value = std::str::from_utf8(&control.value).ok()?;
    let mut public = false;
    let mut revalidate = false;
    let mut max_age = None;
    for directive in value.split(',').map(str::trim) {
        if directive == "public" {
            public = true;
        } else if directive == "must-revalidate" {
            revalidate = true;
        } else {
            let age = directive.strip_prefix("max-age=")?;
            if max_age.is_some() {
                return None;
            }
            max_age = age.parse::<u64>().ok();
        }
    }
    if !public || !revalidate {
        return None;
    }
    let max_age = max_age?;
    let age = answer.header(b"age").map_or(Some(0), |v| std::str::from_utf8(v).ok()?.parse::<u64>().ok())?;
    (age < max_age).then_some((age, max_age))
}
impl Host for Proxy {
    fn iterate(&mut self, now: Time, _: Wall) {
        self.wire.iterate();
        while let Some(event) = self.wire.events.pop_front() {
            match event {
                Event::Request { id, peer, bytes } => self.request(id, peer, &bytes, now),
                Event::Answer { id, answer } => self.answered(id, answer, now),
            }
        }
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.wire.completions
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.wire.submissions
    }
    fn work_pending(&self, _: Time) -> bool {
        !self.wire.completions.is_empty() || !self.wire.submissions.is_empty()
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        None
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        None
    }
    fn is_empty(&self) -> bool {
        self.wire.empty()
    }
    fn operations(&self) -> u32 {
        256
    }
}
