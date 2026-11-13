//! Real excluded identifiers compared at one clock instant (temper/docs/design/protocol/sign-in.md, 2.5).
use super::{
    checks::PROXY, http::Answer, observation::Observer, proxy::Proxy, requests, service_checks, visitor::Visitor,
};
use crate::{fixtures, referee::Store};
use jig_client::contract as c;
use jig_client_protocol_wire::public_v1 as p;
use skein_http::Method;
use skein_lib::{Reader, Time};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
#[derive(Clone)]
struct Ids {
    project: u64,
    change: u64,
    delegates: Vec<u64>,
}
thread_local! {
    static UNKNOWN: RefCell<Option<Answer>> = const { RefCell::new(None) };
    static IDS: RefCell<Option<Ids>> = const { RefCell::new(None) };
    pub(super) static PROXIED: Cell<bool> = const { Cell::new(false) };
}
pub(super) fn remember(project: u64, change: u64, delegates: Vec<u64>) {
    IDS.with(|ids| *ids.borrow_mut() = Some(Ids { project, change, delegates }));
}
pub(super) struct Walk {
    store: Rc<RefCell<Store>>,
    stage: usize,
    calls: Vec<u64>,
    controls: Vec<u64>,
    projects: Vec<u64>,
    baseline: Option<Answer>,
    instant: Option<Time>,
    wake: Option<Time>,
    public_heads: u32,
}
impl Walk {
    pub fn new(store: Rc<RefCell<Store>>) -> Self {
        Self {
            store,
            stage: 0,
            calls: Vec::new(),
            controls: Vec::new(),
            projects: Vec::new(),
            baseline: None,
            instant: None,
            wake: None,
            public_heads: 0,
        }
    }
    pub fn wake(&self) -> Option<Time> {
        self.wake
    }
    fn answer<'a>(&self, visitor: &'a Visitor) -> &'a Answer {
        visitor.answers[self.calls.last().expect("read")].as_ref().expect("HTTP answer")
    }
    #[expect(
        clippy::too_many_lines,
        reason = "the excluded identifiers share one preparation, instant and byte comparison"
    )]
    pub fn act(
        &mut self,
        now: Time,
        observer: &mut Observer,
        _proxy: &Proxy,
        visitor: &mut Visitor,
        project: u64,
    ) -> bool {
        if self.stage == 9 && observer.public_heads > self.public_heads {
            observer.hold = false;
        }
        if self.wake.is_some_and(|wake| now < wake) || self.calls.iter().any(|id| !visitor.answers.contains_key(id)) {
            return false;
        }
        self.wake = None;
        if self.stage == 8 && observer.held.is_empty() {
            return false;
        }
        let ids = IDS.with(|ids| ids.borrow().as_ref().expect("public calibration").clone());
        assert_eq!(ids.project, project, "fixture deployments draw the same public permutation");
        assert!(!ids.delegates.is_empty(), "the tracked merge exposes the resolution delegate's identifier");
        match self.stage {
            0 | 2 => {
                let number = if self.stage == 0 { 2 } else { 3 };
                self.controls.push(requests::ask(
                    visitor,
                    &c::Ask::MakeProject {
                        name: c::Text::new(format!("Project {number}").into_bytes().into(), 64).expect("name"),
                        policy: 1,
                    },
                    u8::try_from(120 + number).expect("key"),
                ));
            }
            1 | 3 => {
                if !self.controls.iter().all(|id| visitor.answers.contains_key(id)) {
                    return false;
                }
                let answer = requests::answer(
                    &visitor.answers[self.controls.last().expect("project request")]
                        .as_ref()
                        .expect("make project")
                        .body,
                );
                assert!(matches!(answer, c::Answer::Done(_)), "make private project: {answer:?}");
                let number = if self.stage == 1 { 2 } else { 3 };
                self.controls.push(requests::ask(
                    visitor,
                    &c::Ask::SetVisibility { project: number, public: true },
                    u8::try_from(130 + number).expect("key"),
                ));
            }
            4 => {
                if !self.controls.iter().all(|id| visitor.answers.contains_key(id)) {
                    return false;
                }
                self.calls.push(visitor.send(
                    fixtures::CLIENTS,
                    Method::Get,
                    b"/jig/v1/public/0/1/1",
                    &[(b"Host", b"one.example")],
                ));
            }
            5 => {
                let projects = p::PublicProjects::decode(&p::CEILINGS, &mut Reader::new(&self.answer(visitor).body))
                    .expect("all projects");
                assert_eq!(projects.projects().len(), 3);
                self.projects = projects.projects().iter().map(p::PublicProject::project).collect();
                assert_eq!(self.projects[0], project);
                let store = self.store.borrow();
                let tasks: Vec<_> = store
                    .rows
                    .values()
                    .filter_map(|row| {
                        if let temper_engine_domain::Record::Core(jig_domain_core::Record::Tasks(
                            jig_domain_tasks::Stored::Live(task) | jig_domain_tasks::Stored::Ended(task),
                        )) = row
                        {
                            Some(task)
                        } else {
                            None
                        }
                    })
                    .collect();
                assert!(
                    tasks.iter().any(|task| task.number == 2
                        && task.tracked.is_none()
                        && task.project == 1
                        && task.requester == jig_domain_tasks::Party::Task(1)),
                    "change outside every goal is a conversation's delegate"
                );
                assert!(
                    tasks
                        .iter()
                        .any(|task| task.tracked.is_none() && task.requester == jig_domain_tasks::Party::Task(2)),
                    "its resolution is a conversation descendant"
                );
                self.wake = Some(Time::from_nanos(now.as_nanos() + 1_000_000_000));
            }
            _ => {
                let row = (self.stage - 6) / 2;
                if row == 6 {
                    return true;
                }
                if (self.stage - 6).is_multiple_of(2) {
                    if row == 0 {
                        self.instant = Some(now);
                    }
                    assert_eq!(Some(now), self.instant, "excluded reads share one instant");
                    let path = match row {
                        0 => format!("/jig/v1/public/0/1/3/{:016x}/{:016x}", self.projects[1], ids.change),
                        1 => format!("/jig/v1/public/0/1/2/{:016x}", self.projects[2]),
                        2 => format!("/jig/v1/public/0/1/2/{:016x}", self.projects[1]),
                        3 => "/jig/v1/public/0/1/2/0000000000000000".to_owned(),
                        4 => format!("/jig/v1/public/0/1/3/{project:016x}/{:016x}", ids.delegates[0]),
                        5 => format!("/jig/v1/public/2/2/1/{project:016x}/{:016x}", ids.change),
                        _ => unreachable!(),
                    };
                    if row == 1 {
                        self.public_heads = observer.public_heads;
                    }
                    self.calls.push(visitor.send(
                        if PROXIED.with(Cell::get) { PROXY } else { fixtures::CLIENTS },
                        Method::Get,
                        path.as_bytes(),
                        &[(b"Host", b"one.example")],
                    ));
                } else {
                    let answer = self.answer(visitor);
                    assert_eq!(answer.status, 404, "excluded row {row}");
                    assert!(answer.body.is_empty());
                    service_checks::policy(answer, false);
                    if let Some(baseline) = &self.baseline {
                        assert_eq!(answer, baseline, "all excluded identifiers have one answer, row {row}");
                    } else {
                        UNKNOWN.with(|unknown| {
                            if PROXIED.with(Cell::get) {
                                assert_eq!(
                                    Some(answer),
                                    unknown.borrow().as_ref(),
                                    "direct and proxy unknown are byte for byte equal"
                                );
                            } else {
                                *unknown.borrow_mut() = Some(answer.clone());
                            }
                        });
                        self.baseline = Some(answer.clone());
                    }
                    if row == 0 {
                        observer.hold = true;
                        self.controls.push(requests::ask(
                            visitor,
                            &c::Ask::SetVisibility { project: 2, public: false },
                            141,
                        ));
                        self.controls.push(requests::ask(
                            visitor,
                            &c::Ask::RemoveProject { project: 3, grace: None },
                            142,
                        ));
                    }
                }
            }
        }
        self.stage += 1;
        false
    }
}
