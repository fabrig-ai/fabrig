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
pub(super) fn no_cors(answer: &Answer) -> Result<(), String> {
    if !(answer.headers.iter().all(|h| !h.name.to_ascii_lowercase().starts_with(b"access-control-"))) {
        return Err(format!("no CORS header: {answer:?}"));
    }

    Ok(())
}

struct Judge {
    finding: Option<String>,
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
    ) -> Result<(), String> {
        if !proxy.ready() || !upstream.ready() || !second.ready() || self.stopped {
            return Ok(());
        }
        if self.wake.is_some_and(|wake| now < wake) {
            return Ok(());
        }
        self.wake = None;
        if !matches!(self.stage, 8 | 9)
            && self.name == "the_proxy_keeps_age_and_serves_nothing_stale"
            && self.calls.iter().any(|id| !visitor.answers.contains_key(id))
        {
            return Ok(());
        }
        if self.name != "the_proxy_keeps_age_and_serves_nothing_stale"
            && self.calls.iter().any(|id| !visitor.answers.contains_key(id))
        {
            return Ok(());
        }
        match self.name.as_str() {
            "the_proxy_keeps_age_and_serves_nothing_stale"
            | "a_stopped_upstream_serves_copies_until_their_freshness_ends" => {
                self.age(now, proxy, visitor, upstream)?;
            }
            "the_proxy_keeps_only_public_200_and_404" => self.policy(proxy, visitor, upstream)?,
            "two_deployments_behind_one_proxy_keep_their_own_copies" => self.keys(proxy, visitor)?,
            "a_refused_host_is_never_answered_from_the_cache" => self.host(proxy, visitor)?,
            other => return Err(format!("unknown proxy walk {other}")),
        }
        if self.stage == usize::MAX && proxy.idle() && upstream.idle() && second.idle() {
            proxy.stop();
            upstream.stop();
            second.stop();
            self.stopped = true;
        }

        Ok(())
    }
    fn last<'a>(&self, visitor: &'a Visitor) -> Result<&'a Answer, String> {
        Ok({
            visitor
                .answers
                .get(self.calls.last().ok_or_else(|| String::from("sent call"))?)
                .ok_or_else(|| String::from("completed call"))?
                .as_ref()
                .ok_or_else(|| String::from("answered call"))?
        })
    }
    #[expect(clippy::too_many_lines, reason = "the ordered referee checks return their findings at each step")]
    fn age(&mut self, now: Time, proxy: &Proxy, visitor: &mut Visitor, upstream: &mut Upstream) -> Result<(), String> {
        match self.stage {
            0 => {
                upstream.answer = document(200, b"public, max-age=3, must-revalidate", Some(b"1"), b"first");
                self.send(visitor, b"one.example", PATH);
                self.stage = 1;
            }
            1 => {
                match (&(self.last(visitor)?.body.as_ref()), &(b"first")) {
                    (left, right) if left != right => {
                        let finding = String::from("assertion `left == right` failed");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                self.wake = Some(Time::from_nanos(now.as_nanos() + 1_000_000_000));
                self.stage = 2;
            }
            2 => {
                self.send(visitor, b"one.example", PATH);
                self.stage = 3;
            }
            3 => {
                match (&(self.last(visitor)?.header(b"Age")), &(Some(b"2".as_slice()))) {
                    (left, right) if left != right => {
                        let finding = String::from("assertion `left == right` failed");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                match (&(proxy.counts.upstream_public), &(1)) {
                    (left, right) if left != right => {
                        let finding = String::from("assertion `left == right` failed");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                if self.name == "the_proxy_keeps_age_and_serves_nothing_stale" {
                    upstream.hold = true;
                    self.wake = Some(Time::from_nanos(now.as_nanos() + 1_000_000_000));
                    self.stage = 7;
                } else {
                    upstream.stop();
                    self.send(visitor, b"one.example", PATH);
                    self.stage = 4;
                }
            }
            4 => {
                match (&(self.last(visitor)?.body.as_ref()), &(b"first")) {
                    (left, right) if left != right => {
                        let finding = String::from("assertion `left == right` failed");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                self.wake = Some(Time::from_nanos(now.as_nanos() + 1_000_000_000));
                self.stage = 5;
            }
            5 => {
                self.send(visitor, b"one.example", PATH);
                self.stage = 6;
            }
            6 => {
                match (&(self.last(visitor)?.status), &(502)) {
                    (left, right) if left != right => {
                        let finding = String::from("no stale copy on upstream failure");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                match (&(proxy.counts.upstream_public), &(2)) {
                    (left, right) if left != right => {
                        let finding = String::from("assertion `left == right` failed");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                match (&(proxy.counts.served_from_cache), &(2)) {
                    (left, right) if left != right => {
                        let finding = String::from("assertion `left == right` failed");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                self.stage = usize::MAX;
            }
            7 => {
                self.send(visitor, b"one.example", PATH);
                self.stage = 8;
            }
            8 => {
                if upstream.requests.len() != 2 {
                    return Ok(());
                }
                if visitor.answers.contains_key(self.calls.last().ok_or_else(|| String::from("refresh request"))?) {
                    return Err(String::from("no stale copy while refreshing"));
                }
                self.send(visitor, b"one.example", PATH);
                self.stage = 9;
            }
            9 => {
                if upstream.requests.len() != 3 {
                    return Ok(());
                }
                for id in self.calls.iter().rev().take(2) {
                    if visitor.answers.contains_key(id) {
                        return Err(String::from("no stale copy for either refresher"));
                    }
                }
                match (&(proxy.counts.served_from_cache), &(1)) {
                    (left, right) if left != right => {
                        let finding = String::from("assertion `left == right` failed");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                upstream.release();
                self.stage = 10;
            }
            10 => {
                match (&(proxy.counts.upstream_public), &(3)) {
                    (left, right) if left != right => {
                        let finding = String::from("refreshes are foreground reads");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                match (&(self.last(visitor)?.status), &(200)) {
                    (left, right) if left != right => {
                        let finding = String::from("assertion `left == right` failed");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                self.stage = usize::MAX;
            }
            usize::MAX => {}
            other => return Err(format!("age stage {other}")),
        }

        Ok(())
    }
    fn policy(&mut self, proxy: &Proxy, visitor: &mut Visitor, upstream: &mut Upstream) -> Result<(), String> {
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
            (200, b"public, max-age=2, must-revalidate, stale-while-revalidate=2", 2),
            (200, b"public, max-age=2, must-revalidate, stale-if-error=2", 2),
        ];
        if self.stage == usize::MAX {
            return Ok(());
        }
        let row = self.stage / 3;
        if row >= ROWS.len() {
            let extra = row - ROWS.len();
            if extra == 2 {
                self.stage = usize::MAX;
                return Ok(());
            }
            if self.stage % 3 < 2 {
                upstream.answer = document(200, b"public, max-age=2, must-revalidate", None, b"other route");
                self.calls.push(visitor.send(
                    PROXY,
                    if extra == 0 { Method::Post } else { Method::Get },
                    if extra == 0 { PATH } else { b"/jig/v1/read" },
                    &[(b"Host", b"one.example")],
                ));
            } else {
                match (
                    &(proxy.counts.upstream_other),
                    &(u32::try_from((extra + 1) * 2).map_err(|why| format!("other requests: {why:?}"))?),
                ) {
                    (left, right) if left != right => {
                        let finding = String::from("neither a public POST nor another GET is cached");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
            }
            self.stage += 1;
            return Ok(());
        }
        let (status, control, reached) = ROWS[row];
        let path = format!("/jig/v1/public/0/1/{row}");
        match self.stage % 3 {
            0 => {
                upstream.answer = document(status, control, None, b"policy");
                self.send(visitor, b"one.example", path.as_bytes());
            }
            1 => {
                match (&(self.last(visitor)?.status), &(status)) {
                    (left, right) if left != right => {
                        let finding = String::from("assertion `left == right` failed");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                self.send(visitor, b"one.example", path.as_bytes());
            }
            2 => {
                match (&(self.last(visitor)?.status), &(status)) {
                    (left, right) if left != right => {
                        let finding = String::from("assertion `left == right` failed");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                let expected: u32 = ROWS[..=row].iter().map(|r| r.2).sum();
                match (&(proxy.counts.upstream_public), &(expected)) {
                    (left, right) if left != right => {
                        let finding = format!("cache row {row}: expected {reached} upstream calls");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
            }
            other => return Err(format!("policy stage {other}")),
        }
        self.stage += 1;

        Ok(())
    }
    fn keys(&mut self, proxy: &mut Proxy, visitor: &mut Visitor) -> Result<(), String> {
        match self.stage {
            0 => self.send(visitor, b"one.example", PATH),
            1 | 3 => {
                match (&(self.last(visitor)?.body.as_ref()), &(b"first")) {
                    (left, right) if left != right => {
                        let finding = String::from("assertion `left == right` failed");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                self.send(visitor, b"two.example", PATH);
            }
            2 => {
                match (&(self.last(visitor)?.body.as_ref()), &(b"second")) {
                    (left, right) if left != right => {
                        let finding = String::from("assertion `left == right` failed");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                self.send(visitor, b"one.example", PATH);
            }
            4 => {
                match (&(self.last(visitor)?.body.as_ref()), &(b"second")) {
                    (left, right) if left != right => {
                        let finding = String::from("assertion `left == right` failed");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                match (&(proxy.counts.upstream_public), &(2)) {
                    (left, right) if left != right => {
                        let finding = String::from("assertion `left == right` failed");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                match (&(proxy.counts.served_from_cache), &(2)) {
                    (left, right) if left != right => {
                        let finding = String::from("assertion `left == right` failed");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                self.send(visitor, b"alias.example", PATH);
            }
            5 => match (&(proxy.counts.upstream_public), &(3)) {
                (left, right) if left != right => {
                    let finding = String::from("validated Host is part of the key even on one upstream");
                    return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                }
                _ => {}
            },
            6..=8 => self.send(
                visitor,
                b"one.example",
                match self.stage {
                    6 => b"/jig/v1/public/0/1/%31",
                    7 => b"/jig/v1/public/0/1/./1",
                    _ => b"/jig/v1/public/0/1%2F1",
                },
            ),
            9 => {
                match (&(proxy.counts.upstream_public), &(6)) {
                    (left, right) if left != right => {
                        let finding =
                            String::from("raw paths are neither decoded nor normalized into the canonical cached path");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                proxy.config.upstreams[0].names.retain(|name| name.as_ref() != b"one.example");
                proxy.config.upstreams[1].names.push(b"one.example".as_slice().into());
                self.send(visitor, b"one.example", PATH);
            }
            10 => {
                match (&(self.last(visitor)?.body.as_ref()), &(b"second")) {
                    (left, right) if left != right => {
                        let finding =
                            String::from("the same Host and path on another upstream cannot reuse its old copy");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                match (&(proxy.counts.upstream_public), &(7)) {
                    (left, right) if left != right => {
                        let finding = String::from("assertion `left == right` failed");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                self.stage = usize::MAX;
                return Ok(());
            }
            usize::MAX => return Ok(()),
            other => return Err(format!("key stage {other}")),
        }
        self.stage += 1;

        Ok(())
    }
    fn host(&mut self, proxy: &mut Proxy, visitor: &mut Visitor) -> Result<(), String> {
        match self.stage {
            0 => self.send(visitor, b"one.example", PATH),
            1 => {
                match (&(self.last(visitor)?.status), &(200)) {
                    (left, right) if left != right => {
                        let finding = String::from("assertion `left == right` failed");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                proxy.config.upstreams[0].names.retain(|name| name.as_ref() != b"one.example");
                self.send(visitor, b"one.example", PATH);
            }
            2 => {
                match (&(self.last(visitor)?.status), &(421)) {
                    (left, right) if left != right => {
                        let finding = String::from("assertion `left == right` failed");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                no_cors(self.last(visitor)?)?;
                match (&(proxy.counts.refused_host), &(1)) {
                    (left, right) if left != right => {
                        let finding = String::from("assertion `left == right` failed");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                match (&(proxy.counts.upstream_public), &(1)) {
                    (left, right) if left != right => {
                        let finding = String::from("assertion `left == right` failed");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                match (&(proxy.counts.served_from_cache), &(0)) {
                    (left, right) if left != right => {
                        let finding = String::from("assertion `left == right` failed");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                self.stage = usize::MAX;
                return Ok(());
            }
            usize::MAX => return Ok(()),
            other => return Err(format!("host stage {other}")),
        }
        self.stage += 1;

        Ok(())
    }
}
impl Judge {
    fn act_checked(&mut self, now: Time, procs: &mut [Program]) -> Result<(), String> {
        let [Program::Proxy(proxy), Program::Visitor(visitor), Program::Upstream(upstream), Program::Upstream(second)] =
            procs
        else {
            return Err(String::from("proxy program composition"));
        };
        self.act_inner(now, proxy, visitor, upstream, second)?;

        Ok(())
    }
}
impl skein_world::Referee<Program> for Judge {
    fn watch(&mut self, _: Time, procs: &[Program]) -> Result<(), String> {
        if let Some(why) = &self.finding {
            return Err(why.clone());
        }

        let [Program::Proxy(proxy), Program::Visitor(visitor), Program::Upstream(upstream), Program::Upstream(second)] =
            procs
        else {
            return Err(String::from("proxy composition"));
        };
        self.act_ready = !self.stopped
            && proxy.ready()
            && upstream.ready()
            && second.ready()
            && (self.calls.iter().all(|id| visitor.answers.contains_key(id))
                || (self.name == "the_proxy_keeps_age_and_serves_nothing_stale" && matches!(self.stage, 8 | 9)));

        Ok(())
    }
    fn act(&mut self, now: Time, procs: &mut [Program]) {
        if self.finding.is_none() {
            self.finding = self.act_checked(now, procs).err();
        }
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
        self.finding.is_none() && self.stopped
    }
}
pub(super) fn run(seed: u64, name: &str) {
    let machine = temper_playground::machine::Machine::new(temper_playground::machine::fresh());
    let mut world =
        skein_world::World::new_controlled(seed, skein_sim::Config::calm(), skein_world::Memory::Unchecked, |_| {
            Judge {
                finding: None,
                seed,
                name: name.into(),
                stage: 0,
                calls: Vec::new(),
                wake: None,
                stopped: false,
                act_ready: false,
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use skein_world::Referee as _;

    #[test]
    fn an_act_finding_blocks_passed_and_is_kept_for_watch() {
        let mut judge = Judge {
            finding: None,
            seed: 17,
            name: "proxy finding".into(),
            stage: 0,
            calls: Vec::new(),
            wake: None,
            stopped: true,
            act_ready: false,
        };
        assert!(judge.passed());
        judge.act(Time::ZERO, &mut []);
        assert!(!judge.passed(), "a kept act finding prevents success");
        assert_eq!(judge.watch(Time::ZERO, &[]), Err(String::from("proxy program composition")));
        judge.act(Time::ZERO, &mut []);
        assert_eq!(judge.watch(Time::ZERO, &[]), Err(String::from("proxy program composition")));
    }
}
