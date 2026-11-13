//! Visibility decisions and the last fresh proxy copy (temper/docs/design/protocol/sign-in.md, 2.5).
use super::{checks::PROXY, observation::Observer, proxy::Proxy, requests, service_checks, visitor::Visitor};
use crate::fixtures;
use jig_client::contract as c;
use jig_client_protocol_wire::public_v1 as p;
use skein_http::Method;
use skein_lib::{Reader, Time};

pub(super) struct Walk {
    stage: usize,
    calls: Vec<u64>,
    controls: Vec<u64>,
    projects: Vec<u64>,
    wake: Option<Time>,
    primed: Time,
    cached: u32,
    heads: u32,
    race_project: usize,
}
impl Walk {
    pub fn new() -> Self {
        Self {
            stage: 0,
            calls: Vec::new(),
            controls: Vec::new(),
            projects: Vec::new(),
            wake: None,
            primed: Time::ZERO,
            cached: 0,
            heads: 0,
            race_project: 0,
        }
    }
    pub fn wake(&self) -> Option<Time> {
        self.wake
    }
    fn pair(&mut self, visitor: &mut Visitor, address: std::net::SocketAddr) {
        self.calls.clear();
        for project in &self.projects {
            self.calls.push(visitor.send(
                address,
                Method::Get,
                format!("/jig/v1/public/0/1/2/{project:016x}").as_bytes(),
                &[(b"Host", b"one.example")],
            ));
        }
    }
    fn answers(&self, visitor: &Visitor, status: u16, cached: bool) {
        for id in &self.calls {
            let answer = visitor.answers[id].as_ref().expect("HTTP response");
            assert_eq!(answer.status, status, "revocation stage {}", self.stage);
            service_checks::policy(answer, cached);
            if status == 404 {
                assert!(answer.body.is_empty());
            }
        }
    }
    #[expect(clippy::too_many_lines, reason = "the two revocations share their cached copies and freshness boundary")]
    pub fn act(
        &mut self,
        now: Time,
        observer: &mut Observer,
        proxy: &Proxy,
        visitor: &mut Visitor,
        first: u64,
        quiescent: bool,
    ) -> bool {
        if (20..=22).contains(&self.stage) {
            self.race(observer, visitor, quiescent);
            return false;
        }
        if self.wake.is_some_and(|wake| now < wake)
            || self.calls.iter().chain(self.controls.iter()).any(|id| !visitor.answers.contains_key(id))
        {
            return false;
        }
        self.wake = None;
        match self.stage {
            0 => self.controls.push(requests::ask(
                visitor,
                &c::Ask::MakeProject {
                    name: c::Text::new(b"Second public".as_slice().into(), 64).expect("name"),
                    policy: 1,
                },
                111,
            )),
            1 => {
                let answer =
                    requests::answer(&visitor.answers[&self.controls[0]].as_ref().expect("make response").body);
                assert!(
                    matches!(
                        answer,
                        c::Answer::Done(c::Outcome::Document(c::DocumentOutcome::ProjectMade { project: 2 }))
                    ),
                    "make second project: {answer:?}"
                );
                self.controls.push(requests::ask(visitor, &c::Ask::SetVisibility { project: 2, public: true }, 112));
            }
            2 => {
                assert!(matches!(
                    requests::answer(&visitor.answers[&self.controls[1]].as_ref().expect("visibility response").body),
                    c::Answer::Done(_)
                ));
                self.calls.push(visitor.send(
                    fixtures::CLIENTS,
                    Method::Get,
                    b"/jig/v1/public/0/1/1",
                    &[(b"Host", b"one.example")],
                ));
            }
            3 => {
                let answer = visitor.answers[&self.calls[0]].as_ref().expect("projects");
                assert_eq!(answer.status, 200);
                let projects =
                    p::PublicProjects::decode(&p::CEILINGS, &mut Reader::new(&answer.body)).expect("projects record");
                assert_eq!(projects.projects().len(), 2);
                self.projects = projects.projects().iter().map(p::PublicProject::project).collect();
                assert!(self.projects.contains(&first));
                self.wake = Some(Time::from_nanos(now.as_nanos() + 1_000_000_000));
            }
            4 => {
                self.primed = now;
                self.pair(visitor, PROXY);
            }
            5 => {
                self.answers(visitor, 200, true);
                observer.hold = true;
                self.controls.push(requests::ask(
                    visitor,
                    &c::Ask::Rename {
                        project: 1,
                        name: c::Text::new(b"Public project revised".as_slice().into(), 64).expect("name"),
                    },
                    115,
                ));
                self.stage = 20;
                return false;
            }
            6 => {
                for id in self.controls.iter().skip(2) {
                    let answer = requests::answer(&visitor.answers[id].as_ref().expect("revocation response").body);
                    assert!(matches!(answer, c::Answer::Done(_)), "visibility/deletion committed: {answer:?}");
                }
                self.answers(visitor, 404, false);
                // One held request and one revocation fit the sign-in's two ordinary exchanges.
                if self.race_project == 0 {
                    self.race_project = 1;
                    observer.hold = true;
                    self.controls.push(requests::ask(
                        visitor,
                        &c::Ask::Rename {
                            project: 2,
                            name: c::Text::new(b"Second public revised".as_slice().into(), 64).expect("name"),
                        },
                        116,
                    ));
                    self.stage = 20;
                    return false;
                }
                self.pair(visitor, fixtures::CLIENTS);
            }
            7 => {
                self.answers(visitor, 404, false);
                self.pair(visitor, PROXY);
            }
            8 => {
                self.answers(visitor, 200, true);
                self.cached = proxy.counts.served_from_cache;
                assert_eq!(self.cached, 2, "both projects were cached while revoked");
                self.wake = Some(Time::from_nanos(self.primed.as_nanos() + 2_000_000_000));
            }
            9 | 11 => self.pair(visitor, PROXY),
            10 => {
                self.answers(visitor, 404, true);
                assert_eq!(proxy.counts.served_from_cache, self.cached, "no revoked document after max-age");
                self.wake = Some(Time::from_nanos(now.as_nanos() + 2_000_000_000));
            }
            12 => {
                self.answers(visitor, 404, true);
                return true;
            }
            other => panic!("revocation stage {other}"),
        }
        self.stage += 1;
        false
    }
    fn race(&mut self, observer: &mut Observer, visitor: &mut Visitor, quiescent: bool) {
        match self.stage {
            20 if !observer.held.is_empty() => {
                self.heads = observer.public_heads;
                self.calls.clear();
                self.calls.push(visitor.send(
                    fixtures::CLIENTS,
                    Method::Get,
                    format!("/jig/v1/public/0/1/2/{:016x}", self.projects[self.race_project]).as_bytes(),
                    &[(b"Host", b"one.example")],
                ));
                self.stage = 21;
            }
            21 if observer.public_heads == self.heads + 1 => {
                assert!(
                    self.calls.iter().all(|id| !visitor.answers.contains_key(id)),
                    "the read waits for the held commit before revocation"
                );
                self.heads = observer.keyed_heads;
                self.controls.push(if self.race_project == 0 {
                    requests::ask(visitor, &c::Ask::SetVisibility { project: 1, public: false }, 113)
                } else {
                    requests::ask(visitor, &c::Ask::RemoveProject { project: 2, grace: None }, 114)
                });
                self.stage = 22;
            }
            22 if observer.keyed_heads == self.heads + 1 && quiescent => {
                assert!(
                    self.controls.iter().rev().take(2).all(|id| !visitor.answers.contains_key(id)),
                    "revocation proceeds while the reads are still under way: {:?}",
                    self.controls
                        .iter()
                        .skip(2)
                        .filter_map(|id| visitor.answers.get(id).map(|answer| {
                            jig_client_protocol_wire::v21::KeyedAnswer::decode(
                                &jig_client_protocol_wire::v21::CEILINGS,
                                &mut Reader::new(&answer.as_ref().expect("HTTP control").body),
                            )
                            .expect("control record")
                        }))
                        .collect::<Vec<_>>()
                );
                observer.hold = false;
                self.stage = 6;
            }
            20..=22 => {}
            other => panic!("revocation race stage {other}"),
        }
    }
}
