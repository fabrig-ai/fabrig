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
    fn answer<'a>(&self, visitor: &'a Visitor) -> Result<&'a Answer, String> {
        Ok({
            visitor.answers[self.calls.last().ok_or_else(|| String::from("read"))?]
                .as_ref()
                .ok_or_else(|| String::from("HTTP answer"))?
        })
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
    ) -> Result<bool, String> {
        if self.stage == 9 && observer.public_heads > self.public_heads {
            observer.hold = false;
        }
        if self.wake.is_some_and(|wake| now < wake) || self.calls.iter().any(|id| !visitor.answers.contains_key(id)) {
            return Ok(false);
        }
        self.wake = None;
        if self.stage == 8 && observer.held.is_empty() {
            return Ok(false);
        }
        let ids = IDS.with(|ids| ids.borrow().clone()).ok_or_else(|| String::from("public calibration"))?;
        match (&(ids.project), &(project)) {
            (left, right) if left != right => {
                let finding = String::from("fixture deployments draw the same public permutation");
                return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
            }
            _ => {}
        }
        if ids.delegates.is_empty() {
            return Err(String::from("the tracked merge exposes the resolution delegate's identifier"));
        }
        match self.stage {
            0 | 2 => {
                let number = if self.stage == 0 { 2 } else { 3 };
                self.controls.push(requests::ask(
                    visitor,
                    &c::Ask::MakeProject {
                        name: c::Text::new(format!("Project {number}").into_bytes().into(), 64)
                            .map_err(|why| format!("name: {why:?}"))?,
                        policy: 1,
                    },
                    u8::try_from(120 + number).map_err(|why| format!("key: {why:?}"))?,
                ));
            }
            1 | 3 => {
                if !self.controls.iter().all(|id| visitor.answers.contains_key(id)) {
                    return Ok(false);
                }
                let answer = requests::answer(
                    &visitor.answers[self.controls.last().ok_or_else(|| String::from("project request"))?]
                        .as_ref()
                        .ok_or_else(|| String::from("make project"))?
                        .body,
                )?;
                if !(matches!(answer, c::Answer::Done(_))) {
                    return Err(format!("make private project: {answer:?}"));
                }
                let number = if self.stage == 1 { 2 } else { 3 };
                self.controls.push(requests::ask(
                    visitor,
                    &c::Ask::SetVisibility { project: number, public: true },
                    u8::try_from(130 + number).map_err(|why| format!("key: {why:?}"))?,
                ));
            }
            4 => {
                if !self.controls.iter().all(|id| visitor.answers.contains_key(id)) {
                    return Ok(false);
                }
                self.calls.push(visitor.send(
                    fixtures::CLIENTS,
                    Method::Get,
                    b"/jig/v1/public/0/1/1",
                    &[(b"Host", b"one.example")],
                ));
            }
            5 => {
                let projects = p::PublicProjects::decode(&p::CEILINGS, &mut Reader::new(&self.answer(visitor)?.body))
                    .map_err(|why| format!("all projects: {why:?}"))?;
                match (&(projects.projects().len()), &(3)) {
                    (left, right) if left != right => {
                        let finding = String::from("assertion `left == right` failed");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                self.projects = projects.projects().iter().map(p::PublicProject::project).collect();
                match (&(self.projects[0]), &(project)) {
                    (left, right) if left != right => {
                        let finding = String::from("assertion `left == right` failed");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
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
                if !(tasks.iter().any(|task| {
                    task.number == 2
                        && task.tracked.is_none()
                        && task.project == 1
                        && task.requester == jig_domain_tasks::Party::Task(1)
                })) {
                    return Err(String::from("change outside every goal is a conversation's delegate"));
                }
                if !(tasks
                    .iter()
                    .any(|task| task.tracked.is_none() && task.requester == jig_domain_tasks::Party::Task(2)))
                {
                    return Err(String::from("its resolution is a conversation descendant"));
                }
                self.wake = Some(Time::from_nanos(now.as_nanos() + 1_000_000_000));
            }
            _ => {
                let row = (self.stage - 6) / 2;
                if row == 6 {
                    return Ok(true);
                }
                if (self.stage - 6).is_multiple_of(2) {
                    if row == 0 {
                        self.instant = Some(now);
                    }
                    match (&(Some(now)), &(self.instant)) {
                        (left, right) if left != right => {
                            let finding = String::from("excluded reads share one instant");
                            return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                        }
                        _ => {}
                    }
                    let path = match row {
                        0 => format!("/jig/v1/public/0/1/3/{:016x}/{:016x}", self.projects[1], ids.change),
                        1 => format!("/jig/v1/public/0/1/2/{:016x}", self.projects[2]),
                        2 => format!("/jig/v1/public/0/1/2/{:016x}", self.projects[1]),
                        3 => "/jig/v1/public/0/1/2/0000000000000000".to_owned(),
                        4 => format!("/jig/v1/public/0/1/3/{project:016x}/{:016x}", ids.delegates[0]),
                        5 => format!("/jig/v1/public/2/2/1/{project:016x}/{:016x}", ids.change),
                        _ => return Err(String::from("unreachable")),
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
                    let answer = self.answer(visitor)?;
                    match (&(answer.status), &(404)) {
                        (left, right) if left != right => {
                            let finding = format!("excluded row {row}");
                            return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                        }
                        _ => {}
                    }
                    if !(answer.body.is_empty()) {
                        return Err(format!("assertion failed: {}", stringify!(answer.body.is_empty())));
                    }
                    service_checks::policy(answer, false)?;
                    if let Some(baseline) = &self.baseline {
                        match (&(answer), &(baseline)) {
                            (left, right) if left != right => {
                                let finding = format!("all excluded identifiers have one answer, row {row}");
                                return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                            }
                            _ => {}
                        }
                    } else {
                        UNKNOWN.with(|unknown| {
                            if PROXIED.with(Cell::get) {
                                match (&(Some(answer)), &(unknown.borrow().as_ref())) {
                                    (left, right) if left != right => {
                                        let finding = String::from("direct and proxy unknown are byte for byte equal");
                                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                                    }
                                    _ => {}
                                }
                            } else {
                                *unknown.borrow_mut() = Some(answer.clone());
                            }

                            Ok(())
                        })?;
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
        Ok(false)
    }
}
