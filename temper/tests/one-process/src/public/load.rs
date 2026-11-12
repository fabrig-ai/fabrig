//! Concurrent public admission beside signed-in work (temper/docs/design/protocol/sign-in.md, 2.5).
use super::{
    checks::PROXY, http::Answer, observation::Observer, proxy::Proxy, requests, service, service_checks,
    visitor::Visitor,
};
use crate::fixtures;
use jig_client::contract as c;
use jig_client_protocol_wire::v20 as w;
use skein_http::Method;
use skein_lib::{Reader, Time};

pub(super) struct Walk {
    phase: usize,
    stage: usize,
    calls: Vec<u64>,
    control: Option<u64>,
    signed_read: Option<u64>,
    wake: Option<Time>,
    started: Time,
    upstream: u32,
}
impl Walk {
    pub fn new() -> Self {
        Self {
            phase: 0,
            stage: 0,
            calls: Vec::new(),
            control: None,
            signed_read: None,
            wake: None,
            started: Time::ZERO,
            upstream: 0,
        }
    }
    pub fn diagnostic(&self) -> String {
        format!("load phase {}, stage {}, calls {:?}", self.phase, self.stage, self.calls)
    }
    pub fn wake(&self) -> Option<Time> {
        self.wake
    }
    pub fn ready(&self, visitor: &Visitor, observer: &Observer) -> bool {
        match self.stage {
            0 | 1 => true,
            2 => self.calls.last().is_some_and(|id| visitor.answers.contains_key(id)),
            3 => !observer.held.is_empty(),
            4 => self
                .calls
                .iter()
                .any(|id| visitor.answers.get(id).is_some_and(|a| a.as_ref().is_some_and(|a| a.status == 503))),
            5 => self
                .calls
                .iter()
                .chain(self.control.iter())
                .chain(self.signed_read.iter())
                .all(|id| visitor.answers.contains_key(id)),
            _ => false,
        }
    }
    fn signed_read(&mut self, visitor: &mut Visitor, address: std::net::SocketAddr) {
        let bearer = [b"Bearer ".as_slice(), service::bearer().as_ref()].concat();
        self.signed_read = Some(visitor.send_body(
            address,
            Method::Post,
            b"/jig/v1/read",
            &[(b"Host", b"one.example"), (b"Content-Type", b"application/octet-stream"), (b"Authorization", &bearer)],
            &requests::read_body(&w::Read::Me),
        ));
    }
    fn public(&mut self, visitor: &mut Visitor, address: std::net::SocketAddr, project: u64) {
        let path = if self.phase == 2 {
            format!("/jig/v1/public/2/1/2/{project:016x}")
        } else if self.phase == 0 {
            "/jig/v1/public/0/1/1".to_owned()
        } else {
            format!(
                "/jig/v1/public/0/1/2/{:016x}",
                project.wrapping_add(u64::try_from(self.phase * 100 + self.calls.len()).expect("few reads"))
            )
        };
        self.calls.push(visitor.send(address, Method::Get, path.as_bytes(), &[(b"Host", b"one.example")]));
    }
    fn controls(&self, visitor: &Visitor, now: Time) {
        assert!(now.saturating_since(self.started) <= fixtures::PERSON_WAIT, "signed-in work keeps its usual bound");
        let read = visitor.answers[&self.signed_read.expect("signed-in read")].as_ref().expect("read response");
        assert_eq!(read.status, 200);
        assert!(matches!(
            w::ReadAnswer::decode(&w::CEILINGS, &mut Reader::new(&read.body)).expect("me answer"),
            w::ReadAnswer::Value(_)
        ));
        let control = visitor.answers[&self.control.expect("signed-in request")].as_ref().expect("request response");
        assert!(matches!(requests::answer(&control.body), c::Answer::Done(_)), "signed-in request answered");
    }
    #[expect(
        clippy::too_many_lines,
        reason = "each admission phase holds and releases the same disk barrier beside signed-in work"
    )]
    pub fn act(
        &mut self,
        now: Time,
        observer: &mut Observer,
        proxy: &mut Proxy,
        visitor: &mut Visitor,
        project: u64,
    ) -> bool {
        if self.wake.is_some_and(|wake| now < wake) {
            return false;
        }
        self.wake = None;
        match self.stage {
            0 => {
                if self.phase == 8 {
                    return true;
                }
                self.calls.clear();
                self.control = None;
                self.signed_read = None;
                proxy.config.per_address.reads_per_second = if self.phase == 3 { 1 } else { 100 };
                proxy.config.per_address.reads_in_flight = if self.phase == 4 { 1 } else { 32 };
                proxy.config.public_in_flight = if self.phase == 5 { 1 } else { 32 };
                proxy.config.per_address.anonymous_per_second = if self.phase == 7 { 1 } else { 32 };
                proxy.config.public_pool = if self.phase == 6 { 1 } else { 32 };
                self.wake = Some(Time::from_nanos(now.as_nanos() + 1_000_000_000));
                self.stage = 1;
            }
            1 => {
                self.started = now;
                if self.phase == 0 {
                    self.public(visitor, fixtures::CLIENTS, project);
                    self.stage = 2;
                } else {
                    observer.hold = true;
                    self.control = Some(requests::ask(
                        visitor,
                        &c::Ask::Rename {
                            project: 1,
                            name: c::Text::new(format!("Public name {}", self.phase).into_bytes().into(), 64)
                                .expect("project name"),
                        },
                        u8::try_from(90 + self.phase).expect("key"),
                    ));
                    self.stage = 3;
                }
            }
            2 => {
                if !visitor.answers.contains_key(self.calls.last().expect("rate read")) {
                    return false;
                }
                let answer = visitor.answers[self.calls.last().expect("rate read")].as_ref().expect("HTTP answer");
                assert_eq!(
                    answer.status,
                    if self.calls.len() <= 8 { 200 } else { 503 },
                    "rate read {}",
                    self.calls.len()
                );
                service_checks::policy(answer, false);
                if self.calls.len() == 9 {
                    busy(answer);
                    self.stage = 5;
                } else {
                    self.public(visitor, fixtures::CLIENTS, project);
                    if self.calls.len() == 9 {
                        self.signed_read(visitor, fixtures::CLIENTS);
                        self.control = Some(requests::ask(
                            visitor,
                            &c::Ask::Rename {
                                project: 1,
                                name: c::Text::new(b"Public name rate".as_slice().into(), 64).expect("name"),
                            },
                            90,
                        ));
                    }
                }
            }
            3 => {
                if observer.held.is_empty() {
                    return false;
                }
                self.upstream =
                    if self.phase == 7 { proxy.counts.upstream_other } else { proxy.counts.upstream_public };
                let count = if self.phase == 1 {
                    5
                } else if self.phase == 2 {
                    3
                } else {
                    2
                };
                let address = if self.phase < 3 { fixtures::CLIENTS } else { PROXY };
                for _ in 0..count {
                    if self.phase == 7 {
                        self.calls.push(visitor.send(
                            PROXY,
                            Method::Get,
                            b"/jig/v1/stream",
                            &[(b"Host", b"one.example")],
                        ));
                    } else {
                        self.public(visitor, address, project);
                    }
                }
                self.signed_read(visitor, address);
                self.stage = 4;
            }
            4 => {
                if !self
                    .calls
                    .iter()
                    .any(|id| visitor.answers.get(id).is_some_and(|a| a.as_ref().is_some_and(|a| a.status == 503)))
                {
                    return false;
                }
                if (3..7).contains(&self.phase) {
                    assert_eq!(
                        proxy.counts.upstream_public - self.upstream,
                        1,
                        "proxy phase {} refuses before an upstream connection",
                        self.phase
                    );
                    assert!(proxy.counts.busy > 0);
                }
                observer.hold = false;
                self.stage = 5;
            }
            5 => {
                if !self
                    .calls
                    .iter()
                    .chain(self.control.iter())
                    .chain(self.signed_read.iter())
                    .all(|id| visitor.answers.contains_key(id))
                {
                    return false;
                }
                self.controls(visitor, now);
                if self.phase == 7 {
                    assert_eq!(
                        proxy.counts.upstream_other - self.upstream,
                        2,
                        "one anonymous call and the signed-in read reach the other pool"
                    );
                }
                let mut answered = 0;
                let mut refused = 0;
                for id in &self.calls {
                    let answer = visitor.answers[id].as_ref().expect("public response");
                    service_checks::policy(answer, self.phase >= 3);
                    if answer.status == 503 {
                        busy(answer);
                        refused += 1;
                    } else {
                        assert!(if self.phase == 7 {
                            answer.status == 401
                        } else {
                            matches!(answer.status, 200 | 404)
                        });
                        answered += 1;
                    }
                }
                let expected = match self.phase {
                    0 => (8, 1),
                    1 => (4, 1),
                    2 => (2, 1),
                    _ => (1, 1),
                };
                assert_eq!((answered, refused), expected, "load phase {}", self.phase);
                self.phase += 1;
                self.stage = 0;
            }
            other => panic!("load stage {other}"),
        }
        false
    }
}
fn busy(answer: &Answer) {
    assert_eq!(answer.status, 503);
    assert_eq!(answer.header(b"retry-after"), Some(b"1".as_slice()));
    assert_eq!(answer.header(b"cache-control"), Some(b"no-store".as_slice()));
    assert!(answer.body.is_empty());
}
