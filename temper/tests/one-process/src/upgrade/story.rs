//! History through a fresh-window start (temper/docs/design/testing/README.md, 2.5).
mod image;
mod cuts;
mod history;
mod scenario;
mod judge;
mod runner;
use history::history;
use scenario::{provider,forge_peer,continued,mutations};
pub use runner::{settled,unsettled,power_loss,drawn_power_loss};
use super::{FIXTURE_SETS, maps::Maps};
use crate::{ctl, fixtures, golden, referee, stories, world::{self, HarnessProcess, Person, ServiceObserver, Story}};
use jig_domain_core::{CoreRecord, Record};
use skein_io::kernel::{Complete, Fd, Submit};
use skein_lib::{Time, Wall};
use std::{cell::RefCell, rc::Rc};
use temper_playground::{Links, machine::{self, Machine, SignalHost}, programs::{self, Harness}, service::Observe};

struct Upgrade;
type Proc = programs::Process<Upgrade>;
impl Harness for Upgrade {
    type Service = Observer;
    type Process = HarnessProcess;
    fn patch_sets() -> Option<jig_protocol_store::PatchSets> {
        Some(STATE.with(|s| if s.borrow().upgrade { FIXTURE_SETS } else { temper_service::store::PATCH_SETS }))
    }
    fn seeds(_: u64) -> temper_service::settings::Seeds { fixtures::seeds() }
    fn agent_seed(_: u64) -> u64 { fixtures::SEED }
    fn provider(_: u64) -> skein_fake_peers::llm::Peer { provider() }
    fn files() -> temper_service::settings::Files { fixtures::files() }
    fn given(document: &serde_json::Value) -> temper_service::stock::Given {
        let temper_service::stock::Given::Own {mut jig, forge, ways} = temper_playground::documents::given(document) else { panic!("world defaults") };
        let effect = jig_domain_core::ToolFamilies::JIG.effect.0;
        jig.rules.ceiling.tools.0 |= effect;
        for number in [1,2] {
            let policy = &mut jig.policies.get_mut(&number).expect("world policy").policy;
            policy.ceiling.tools.0 |= effect;
            for role in &mut policy.roles { role.authority.tools.0 |= effect; }
        }
        temper_service::stock::Given::Own {jig, forge, ways}
    }
}
#[derive(Default)]
#[expect(clippy::struct_excessive_bools, reason = "independent upgrade, history, fault and effect observations")]
struct State {
    upgrade: bool,
    before: Option<Maps>,
    checked: bool,
    starts: u32,
    issue: bool,
    armed: bool,
    faulted: bool,
    write_at: Option<u32>,
    disk: Option<Rc<RefCell<skein_fake_machine::Machine>>>,
    task: u64,
    attempt: u64,
    tokens: Vec<(skein_lib::Secret, u64)>,
    users: Vec<(u64, Box<[u8]>)>,
    kept_answer: Vec<u8>,
    stopped_wall: Option<Wall>,
    points: cuts::Points,
    epoch: u32,
    epoch_floor: Option<u64>,
    transport: Option<temper_fake_git_hosted::Typed>,
    end_after_kill: bool,
    killed: bool,
    calibrating: bool,
    kill_told: bool,
}
thread_local! { static STATE: RefCell<State> = RefCell::new(State::default()); }
struct Observer { inner: ServiceObserver, cuts: cuts::Capture }
impl Observe for Observer {
    fn started(root: Fd, inherited: &skein_world::Inherited) -> Self {
        STATE.with(|s| {
            let mut s = s.borrow_mut();
            s.starts += 1;
            s.epoch += 1;
            if s.epoch > 1 {
                STORE.with(|store| {
                    let store = store.borrow();
                    let mut store = store.as_ref().expect("store").borrow_mut();
                    store.recovering();
                    store.agent_records.clear();
                });
                s.epoch_floor = None;
                s.checked = false;
            }
        });
        let mut inner = ServiceObserver::started(root, inherited);
        inner.wire.keep_starts();
        Self { inner, cuts: cuts::Capture::new(root) }
    }
    fn complete(&mut self, complete: Complete, service: &mut temper_service::Service) -> Option<Complete> {
        if STATE.with(|s|s.borrow().upgrade) && matches!((&complete.kind,&complete.result),(skein_io::kernel::Op::Rename {to,..},Ok(skein_io::kernel::Done::Nothing)) if to.as_ref() == b"active") {
            assert_eq!(service.wait_state().engine.expect("engine").restart,Some(jig_domain_core::start::Phase::Switch),"all restores finish before the switch");
        }
        STATE.with(|s| self.cuts.complete(&complete,self.inner.submitted,&mut s.borrow_mut().points));
        self.inner.complete(complete, service)
    }
    fn before_iterate(&mut self, service: &mut temper_service::Service, now: Time, wall: Wall) -> bool {
        let pending = self.inner.before_iterate(service, now, wall);
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            if !state.upgrade { state.stopped_wall=Some(wall); }
            if state.upgrade && !state.checked {
                let wires = STORE.with(|s| s.borrow().as_ref().expect("store").borrow().wires);
                if state.epoch_floor.is_none() {
                    state.epoch_floor = STORE.with(|s| s.borrow().as_ref().expect("store").borrow().restored.as_ref().and_then(|rows| rows.values().find_map(|r| match r {
                        temper_engine_domain::Record::Core(Record::Core(CoreRecord::Deployment(header))) => Some(header.commits),
                        _ => None,
                    })));
                }
                if wires.is_some_and(|n| n > state.epoch_floor.unwrap_or(u64::MAX)) {
                    let after = Maps::read(&mut state.disk.as_ref().expect("disk").borrow_mut());
                    let before=state.before.as_ref().expect("stop snapshot");
                    let source=Maps::generation(&mut state.disk.as_ref().expect("disk").borrow_mut(),before.active);
                    assert_eq!(&source,before,"patching the copy leaves its source generation untouched");
                    history(before, &after);
                    state.points.wires.get_or_insert(self.inner.submitted);
                    state.checked = true;
                }
            }
        });
        pending
    }
    fn submitted(&mut self, submit: &Submit) { self.inner.submitted(submit); }
    fn after_iterate(&mut self, service: &mut temper_service::Service, submitted: u32) {
        self.inner.after_iterate(service, submitted);
    }
    fn handover_read(&mut self, _: &temper_service::HandoverRead) -> Option<Result<Option<skein_lib::Secret>, temper_service::HandoverUnread>> { None }
    fn drain(&mut self, service: &mut temper_service::Service, now: Time) { self.inner.drain(service, now); }
}
thread_local! { static STORE: RefCell<Option<Rc<RefCell<referee::Store>>>> = const { RefCell::new(None) }; }

