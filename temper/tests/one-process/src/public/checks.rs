//! Independent proxy assertions (temper/docs/design/protocol/sign-in.md, 2.4–2.5).
use super::{
    http::{Answer, header},
    programs::{Program, Upstream},
    proxy::{PerAddress, Proxy, ProxyConfig, Upstream as Route},
    visitor::Visitor,
};
use skein_http::Method;
use skein_lib::Time;
use skein_shell::Host;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

pub(super) const PROXY: SocketAddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 42000));
const UPSTREAM: SocketAddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 42001));
const SECOND: SocketAddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 42002));
const PATH: &[u8] = b"/jig/v1/public/0/1/1";
pub(super) fn config(address: SocketAddr) -> ProxyConfig {
    ProxyConfig {
        upstreams: vec![
            Route { names: vec![b"one.example".as_slice().into(), b"alias.example".as_slice().into()], address },
            Route { names: vec![b"two.example".as_slice().into()], address: SECOND },
        ],
        per_address: PerAddress { reads_per_second: 100, reads_in_flight: 32, anonymous_per_second: 32 },
        public_in_flight: 32,
        public_pool: 32,
        other_pool: 4,
    }
}
fn document(status: u16, control: &[u8], age: Option<&[u8]>, body: &[u8]) -> Answer {
    let mut headers = vec![
        header(b"Content-Length", body.len().to_string().as_bytes()),
        header(b"Cache-Control", control),
        header(b"X-Content-Type-Options", b"nosniff"),
    ];
    if let Some(age) = age {
        headers.push(header(b"Age", age));
    }
    Answer { status, headers: headers.into(), body: body.into() }
}
pub(super) fn no_cors(answer: &Answer) {
    assert!(
        answer.headers.iter().all(|h| !h.name.to_ascii_lowercase().starts_with(b"access-control-")),
        "no CORS header: {answer:?}"
    );
}

struct Judge {
    seed: u64,
    name: String,
    stage: usize,
    calls: Vec<u64>,
    wake: Option<Time>,
    stopped: bool,
    act_ready: bool,
}
impl Judge {
    fn send(&mut self, visitor: &mut Visitor, host: &[u8], path: &[u8]) {
        self.calls.push(visitor.send(PROXY, Method::Get, path, &[(b"Host", host)]));
    }
    fn act_inner(
        &mut self,
        now: Time,
        proxy: &mut Proxy,
        visitor: &mut Visitor,
        upstream: &mut Upstream,
        second: &mut Upstream,
    ) {
        if !proxy.ready() || !upstream.ready() || !second.ready() || self.stopped {
            return;
        }
        if self.wake.is_some_and(|wake| now < wake) {
            return;
        }
        self.wake = None;
        if self.calls.iter().any(|id| !visitor.answers.contains_key(id)) {
            return;
        }
        match self.name.as_str() {
            "the_proxy_keeps_age_and_serves_nothing_stale"
            | "a_stopped_upstream_serves_copies_until_their_freshness_ends" => self.age(now, proxy, visitor, upstream),
            "the_proxy_keeps_only_public_200_and_404" => self.policy(proxy, visitor, upstream),
            "two_deployments_behind_one_proxy_keep_their_own_copies" => self.keys(proxy, visitor),
            "a_refused_host_is_never_answered_from_the_cache" => self.host(proxy, visitor),
            other => panic!("unknown proxy walk {other}"),
        }
        if self.stage == usize::MAX && proxy.idle() && upstream.idle() && second.idle() {
            proxy.stop();
            upstream.stop();
            second.stop();
            self.stopped = true;
        }
    }
    fn last<'a>(&self, visitor: &'a Visitor) -> &'a Answer {
        visitor
            .answers
            .get(self.calls.last().expect("sent call"))
            .expect("completed call")
            .as_ref()
            .expect("answered call")
    }
    fn age(&mut self, now: Time, proxy: &Proxy, visitor: &mut Visitor, upstream: &mut Upstream) {
        match self.stage {
            0 => {
                upstream.answer = document(200, b"public, max-age=3, must-revalidate", Some(b"1"), b"first");
                self.send(visitor, b"one.example", PATH);
                self.stage = 1;
            }
            1 => {
                assert_eq!(self.last(visitor).body.as_ref(), b"first");
                self.wake = Some(Time::from_nanos(now.as_nanos() + 1_000_000_000));
                self.stage = 2;
            }
            2 => {
                self.send(visitor, b"one.example", PATH);
                self.stage = 3;
            }
            3 => {
                assert_eq!(self.last(visitor).header(b"Age"), Some(b"2".as_slice()));
                assert_eq!(proxy.counts.upstream_public, 1);
                upstream.stop();
                self.send(visitor, b"one.example", PATH);
                self.stage = 4;
            }
            4 => {
                assert_eq!(self.last(visitor).body.as_ref(), b"first");
                self.wake = Some(Time::from_nanos(now.as_nanos() + 1_000_000_000));
                self.stage = 5;
            }
            5 => {
                self.send(visitor, b"one.example", PATH);
                self.stage = 6;
            }
            6 => {
                assert_eq!(self.last(visitor).status, 502, "no stale copy on upstream failure");
                assert_eq!(proxy.counts.upstream_public, 2);
                assert_eq!(proxy.counts.served_from_cache, 2);
                self.stage = usize::MAX;
            }
            usize::MAX => {}
            other => panic!("age stage {other}"),
        }
    }
    fn policy(&mut self, proxy: &Proxy, visitor: &mut Visitor, upstream: &mut Upstream) {
        const ROWS: &[(u16, &[u8], u32)] = &[
            (200, b"public, max-age=2, must-revalidate", 1),
            (404, b"public, max-age=2, must-revalidate", 1),
            (503, b"public, max-age=2, must-revalidate", 2),
            (200, b"no-store", 2),
            (200, b"public, max-age=2", 2),
            (200, b"private, max-age=2, must-revalidate", 2),
            (200, b"public, max-age=0, must-revalidate", 2),
            (201, b"public, max-age=2, must-revalidate", 2),
            (200, b"public, max-age=2, must-revalidate, no-store", 2),
        ];
        let row = self.stage / 3;
        if row >= ROWS.len() {
            self.stage = usize::MAX;
            return;
        }
        let (status, control, reached) = ROWS[row];
        let path = format!("/jig/v1/public/0/1/{row}");
        match self.stage % 3 {
            0 => {
                upstream.answer = document(status, control, None, b"policy");
                self.send(visitor, b"one.example", path.as_bytes());
            }
            1 => {
                assert_eq!(self.last(visitor).status, status);
                self.send(visitor, b"one.example", path.as_bytes());
            }
            2 => {
                assert_eq!(self.last(visitor).status, status);
                let expected: u32 = ROWS[..=row].iter().map(|r| r.2).sum();
                assert_eq!(
                    proxy.counts.upstream_public, expected,
                    "cache row {row}: expected {reached} upstream calls"
                );
            }
            other => panic!("policy stage {other}"),
        }
        self.stage += 1;
    }
    fn keys(&mut self, proxy: &Proxy, visitor: &mut Visitor) {
        match self.stage {
            0 => self.send(visitor, b"one.example", PATH),
            1 | 3 => {
                assert_eq!(self.last(visitor).body.as_ref(), b"first");
                self.send(visitor, b"two.example", PATH);
            }
            2 => {
                assert_eq!(self.last(visitor).body.as_ref(), b"second");
                self.send(visitor, b"one.example", PATH);
            }
            4 => {
                assert_eq!(self.last(visitor).body.as_ref(), b"second");
                assert_eq!(proxy.counts.upstream_public, 2);
                assert_eq!(proxy.counts.served_from_cache, 2);
                self.send(visitor, b"alias.example", PATH);
            }
            5 => {
                assert_eq!(proxy.counts.upstream_public, 3, "validated Host is part of the key even on one upstream");
                self.stage = usize::MAX;
                return;
            }
            usize::MAX => return,
            other => panic!("key stage {other}"),
        }
        self.stage += 1;
    }
    fn host(&mut self, proxy: &Proxy, visitor: &mut Visitor) {
        match self.stage {
            0 => self.send(visitor, b"one.example", PATH),
            1 => {
                assert_eq!(self.last(visitor).status, 200);
                self.send(visitor, b"refused.example", PATH);
            }
            2 => {
                assert_eq!(self.last(visitor).status, 421);
                no_cors(self.last(visitor));
                assert_eq!(proxy.counts.refused_host, 1);
                assert_eq!(proxy.counts.upstream_public, 1);
                assert_eq!(proxy.counts.served_from_cache, 0);
                self.stage = usize::MAX;
                return;
            }
            usize::MAX => return,
            other => panic!("host stage {other}"),
        }
        self.stage += 1;
    }
}
impl skein_world::Referee<Program> for Judge {
    fn observe(&mut self, _: Time, procs: &[Program]) {
        let [Program::Proxy(proxy), Program::Visitor(visitor), Program::Upstream(upstream), Program::Upstream(second)] =
            procs
        else {
            panic!("proxy composition")
        };
        self.act_ready = !self.stopped
            && proxy.ready()
            && upstream.ready()
            && second.ready()
            && self.calls.iter().all(|id| visitor.answers.contains_key(id));
    }
    fn act(&mut self, now: Time, procs: &mut [Program]) {
        let [Program::Proxy(proxy), Program::Visitor(visitor), Program::Upstream(upstream), Program::Upstream(second)] =
            procs
        else {
            panic!("proxy program composition")
        };
        self.act_inner(now, proxy, visitor, upstream, second);
    }
    fn next_deadline(&self) -> Option<Time> {
        self.wake
            .or_else(|| self.act_ready.then_some(Time::ZERO))
            .or_else(|| (!self.stopped).then_some(Time::from_nanos(10_000_000_000)))
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (!self.stopped && now >= Time::from_nanos(10_000_000_000))
            .then(|| format!("proxy walk {}, stage {}, seed {}", self.name, self.stage, self.seed))
    }
    fn passed(&self) -> bool {
        self.stopped
    }
}
pub(super) fn run(seed: u64, name: &str) {
    let machine = temper_playground::machine::Machine::new(temper_playground::machine::fresh());
    let mut world =
        skein_world::World::new_controlled(seed, skein_sim::Config::calm(), skein_world::Memory::Unchecked, |_| {
            Judge { seed, name: name.into(), stage: 0, calls: Vec::new(), wake: None, stopped: false, act_ready: false }
        })
        .with_machine(machine);
    world.spawn(|| Program::Proxy(Box::new(Proxy::new(PROXY, config(UPSTREAM)))));
    world.spawn(|| Program::Visitor(Box::new(Visitor::new())));
    world.spawn(|| {
        Program::Upstream(Box::new(Upstream::new(
            UPSTREAM,
            document(200, b"public, max-age=2, must-revalidate", None, b"first"),
        )))
    });
    world.spawn(|| {
        Program::Upstream(Box::new(Upstream::new(
            SECOND,
            document(200, b"public, max-age=2, must-revalidate", None, b"second"),
        )))
    });
    let outcome = world.run();
    assert!(outcome.procs.iter().all(Host::is_empty), "proxy world drains, seed {seed}");
}
