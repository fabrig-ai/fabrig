//! History through a registry and job upgrade (temper/docs/design/testing/tiers.md, 1.5).
mod cuts;
mod history;
mod image;
mod judge;
mod runner;
mod scenario;
use super::{CURRENT_SETS, first_sets, maps::Maps};
use crate::{
    ctl, fixtures, golden, referee, stories,
    world::{self, HarnessProcess, Person, ServiceObserver, Story},
};
use history::history;
use jig_domain_core::{CoreRecord, Record};
pub use runner::{drawn_power_loss, power_loss, settled, settled_image, unsettled};
use scenario::{continued, forge_peer, mutations, provider};
use skein_io::kernel::{Complete, Fd, Submit};
use skein_lib::{Time, Wall};
use std::{cell::RefCell, rc::Rc};
use temper_playground::{
    Links,
    machine::{self, Machine, SignalHost},
    programs::{self, Harness},
    service::Observe,
};

struct Upgrade;
type Proc = programs::Process<Upgrade>;
impl Harness for Upgrade {
    type Service = Observer;
    type Process = HarnessProcess;
    fn owner_sets() -> Option<jig_protocol_store::OwnerSets> {
        Some(STATE.with(|s| if s.borrow().upgrade { CURRENT_SETS } else { first_sets() }))
    }
    fn prepared_schemas() -> Option<&'static [skein_codec::schema::Schema]> {
        Some(crate::schemas::binary_schemas())
    }
    fn seeds(_: u64) -> temper_service::settings::Seeds {
        fixtures::seeds()
    }
    fn agent_seed(_: u64) -> u64 {
        fixtures::SEED
    }
    fn provider(_: u64) -> skein_fake_peers::llm::Peer {
        provider()
    }
    fn files() -> temper_service::settings::Files {
        fixtures::files()
    }
    fn configure_engine(engine: &mut jig_protocol_config::Engine, _: &temper_playground::documents::Configuration) {
        // The history world holds the playground reset while its work is inspected.
        engine.acts = Box::new([]);
        engine.limits.core.people.act_grants = 0;
    }
    fn given(document: &temper_playground::documents::Configuration) -> temper_service::stock::Given {
        let temper_service::stock::Given::Own { mut jig, forge, ways } = temper_playground::documents::given(document)
        else {
            panic!("world defaults")
        };
        let effect = jig_domain_core::ToolFamilies::JIG.effect.0;
        jig.rules.ceiling.tools.0 |= effect;
        for number in [1, 2] {
            let policy = &mut jig.policies.get_mut(&number).expect("world policy").policy;
            policy.ceiling.tools.0 |= effect;
            for role in &mut policy.roles {
                role.authority.tools.0 |= effect;
            }
        }
        temper_service::stock::Given::Own { jig, forge, ways }
    }
}
#[derive(Default)]
#[expect(clippy::struct_excessive_bools, reason = "independent upgrade, history, fault and effect observations")]
struct State {
    upgrade: bool,
    before: Option<Maps>,
    before_files: std::collections::BTreeMap<Vec<u8>, Option<Vec<u8>>>,
    seal_cut: bool,
    ran: Vec<(u32, Box<str>)>,
    checked: bool,
    starts: u32,
    issue: bool,
    armed: bool,
    faulted: bool,
    disk: Option<Rc<RefCell<skein_fake_machine::Machine>>>,
    task: u64,
    attempt: u64,
    tokens: Vec<(skein_lib::Secret, u64)>,
    users: Vec<(u64, Box<[u8]>)>,
    kept_answer: Vec<u8>,
    stopped_wall: Option<Wall>,
    upgrade_started: Option<Wall>,
    points: cuts::Points,
    epoch: u32,
    transport: Option<temper_fake_git_hosted::Typed>,
    end_after_kill: bool,
    killed: bool,
    capture_image: bool,
    forge_image: Option<Box<[u8]>>,
    kill_told: bool,
}
thread_local! { static STATE: RefCell<State> = RefCell::new(State::default()); }
struct Observer {
    inner: ServiceObserver,
    cuts: cuts::Capture,
    clients_ready: bool,
}
impl Observe for Observer {
    fn started(root: Fd, inherited: &skein_world::Inherited) -> Self {
        STATE.with(|s| {
            let mut s = s.borrow_mut();
            s.starts += 1;
            s.epoch += 1;
            if s.epoch > 1 {
                if s.seal_cut {
                    assert_eq!(
                        s.ran,
                        crate::upgrade::JOBS.map(|name| (1, Box::from(name))),
                        "the cut follows every job's last page"
                    );
                    assert!(s.points.seal.is_some(), "seal completed before the cut");
                    let mut disk = s.disk.as_ref().expect("disk").borrow_mut();
                    let before = s.before.as_ref().expect("source generation");
                    assert_eq!(Maps::read(&mut disk), *before, "seal cut leaves the source active and unchanged");
                    image::unchanged(&s.before_files, &mut disk, before.active);
                }
                STORE.with(|store| {
                    let store = store.borrow();
                    let mut store = store.as_ref().expect("store").borrow_mut();
                    store.recovering();
                    store.agent_records.clear();
                });
                s.checked = false;
            }
        });
        let mut inner = ServiceObserver::started(root, inherited);
        inner.wire.keep_starts();
        Self { inner, cuts: cuts::Capture::new(root), clients_ready: false }
    }
    fn complete(&mut self, complete: Complete, service: &mut temper_service::Service) -> Option<Complete> {
        if STATE.with(|s| s.borrow().upgrade)
            && matches!((&complete.kind,&complete.result),(skein_io::kernel::Op::Rename {to,..},Ok(skein_io::kernel::Done::Nothing)) if to.as_ref() == b"active")
        {
            assert_eq!(
                service.wait_state().engine.expect("engine").restart,
                Some(jig_domain_core::start::Phase::Switch),
                "all restores finish before the switch"
            );
        }
        STATE.with(|s| self.cuts.complete(&complete, self.inner.submitted, &mut s.borrow_mut().points));
        self.inner.complete(complete, service)
    }
    fn before_iterate(&mut self, service: &mut temper_service::Service, now: Time, wall: Wall) -> bool {
        while let Some(note) = service.listening().pop() {
            if matches!(
                note,
                temper_service::Note::Engine(temper_service::EngineNote::Listening {
                    listener: temper_service::Listening::Clients,
                    ..
                })
            ) {
                self.clients_ready = true;
            }
        }
        for note in service.notes().iter() {
            if let temper_service::Note::Engine(temper_engine_service::Note::Ran { of, job, .. }) = note {
                STATE.with(|s| {
                    let mut s = s.borrow_mut();
                    let epoch = s.epoch;
                    let at = s.ran.iter().rev().take_while(|(seen, _)| *seen == epoch).count();
                    let expected = crate::upgrade::JOBS.get(at).copied().expect("pending job");
                    assert_eq!(
                        (*of, job.as_ref()),
                        (crate::upgrade::job_owner(expected), expected),
                        "pending jobs run in owner order in each start"
                    );
                    s.ran.push((epoch, job.clone()));
                });
            }
        }
        let pending = self.inner.before_iterate(service, now, wall);
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            if !state.upgrade {
                state.stopped_wall = Some(wall);
            }
            if state.upgrade && !state.checked {
                // Keep the first start's wall across a power-loss restart.
                let started = *state.upgrade_started.get_or_insert(wall);
                let synced_wires = STORE.with(|s| s.borrow().as_ref().expect("store").borrow().synced_wires);
                if synced_wires {
                    let after = Maps::read(&mut state.disk.as_ref().expect("disk").borrow_mut());
                    let before = state.before.as_ref().expect("stop snapshot");
                    let source = Maps::generation(&mut state.disk.as_ref().expect("disk").borrow_mut(), before.active);
                    assert_eq!(&source, before, "patching the copy leaves its source generation untouched");
                    image::unchanged(
                        &state.before_files,
                        &mut state.disk.as_ref().expect("disk").borrow_mut(),
                        before.active,
                    );
                    assert!(
                        started >= state.stopped_wall.expect("wall at stop"),
                        "the upgrade does not turn back time"
                    );
                    history(before, &after, started);
                    state.points.wires.get_or_insert(self.inner.submitted);
                    state.checked = true;
                }
            }
        });
        pending
    }
    fn submitted(&mut self, submit: &Submit) {
        self.inner.submitted(submit);
    }
    fn after_iterate(&mut self, service: &mut temper_service::Service, submitted: u32) {
        self.inner.after_iterate(service, submitted);
    }
    fn handover_read(
        &mut self,
        _: &temper_service::HandoverRead,
    ) -> Option<Result<Option<skein_lib::Secret>, temper_service::HandoverUnread>> {
        None
    }
    fn drain(&mut self, service: &mut temper_service::Service, now: Time) {
        self.inner.drain(service, now);
    }
}
thread_local! { static STORE: RefCell<Option<Rc<RefCell<referee::Store>>>> = const { RefCell::new(None) }; }
