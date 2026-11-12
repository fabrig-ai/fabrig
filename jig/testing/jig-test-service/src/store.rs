//! Store requests and application records routed through the host
//! (jig/docs/design/protocol/service.md, 3–5).
use alloc::boxed::Box;
use jig_protocol_store as protocol;
use jig_test_domain as root;
use jig_test_store::{StoreOut, Up, route};
use protocol::host::{Hosted, Lane, NotReady, check_read_page, max_whole};
use skein_io::kernel::{Complete, Fd, Op, Submit};
use skein_lib::{Queue, Time, Token};
pub(super) const KERNEL_HANDOFF_ENTRIES: u32 = protocol::host::KERNEL_HANDOFF_ENTRIES;
#[derive(Debug)]
pub(super) struct Store {
    pub(super) host: Hosted,
    routes: route::Routes,
    restoring: Option<jig_domain_core::start::Phase>,
    pub(super) routed: Queue<Up>,
    pub(super) submitted: Queue<Submit>,
    pub(super) opened: bool,
    pub(super) pending: bool,
    /// The installation's identifier: the drawn one until the store opens,
    /// then the one it keeps (protocol/store/README.md 4.4).
    pub(super) installation: [u8; 16],
    /// The configured models the component names a role's models by, for a test's memory store.
    #[cfg(test)]
    pub(super) models: Box<[jig_domain_core::Model]>,
}

pub(super) fn limits(root: &root::Limits, profile: &protocol::Limits) -> Option<protocol::Limits> {
    let mut kv = profile.store;
    let sizes = protocol::sizes::check_commits(
        &root.core,
        &jig_test_store::application(&root.connector),
        &protocol::sizes::Commits { in_flight: root.journal.commits, writes: root.journal.writes },
        &kv,
        profile.orphan_erases,
    )
    .ok()?;
    protocol::sizes::fill(&mut kv, &sizes)?;
    kv.page.bytes = sizes.key.checked_add(sizes.value)?;
    let limits = protocol::Limits {
        core: root.core,
        store: kv,
        secrets: kv,
        commits: protocol::sizes::Commits { in_flight: root.journal.commits, writes: root.journal.writes },
        joins: root.core.load_slots,
        queue: kv.queued,
        files: profile.files,
        reads: root.core.load_slots.checked_add(root.core.tasks.tasks)?,
        orphan_erases: profile.orphan_erases,
        held: profile.held,
    };
    protocol::sizes::whole(max_whole(&limits)).ok()?;
    check_read_page(&limits)?;
    Some(limits)
}
impl Store {
    pub(super) fn new(
        application: &root::Limits,
        limits: &protocol::Limits,
        models: &[jig_domain_core::Model],
        installation: [u8; 16],
        seed: u64,
    ) -> Self {
        // Route correlations grow with admitted work, without reserving an output envelope.
        let mut routes = route::Routes::new(u32::MAX);
        routes.configure(application);
        routes.configure_restore(limits).expect("restart page bounds fit");
        let host =
            Hosted::new(limits, jig_test_store::application(&application.connector), models, seed, NotReady::InPass);
        Self {
            host,
            routes,
            restoring: None,
            routed: Queue::uncapped(),
            submitted: Queue::with_capacity(KERNEL_HANDOFF_ENTRIES),
            opened: false,
            pending: false,
            installation,
            #[cfg(test)]
            models: models.into(),
        }
    }
    pub(super) fn adopt(&mut self, fd: Fd, user: u32) -> Result<(), Fd> {
        self.host.adopt(fd, user, self.installation)
    }
    pub(super) fn catalogue(&mut self, names: Box<[jig_domain_core::settings::ModelName]>) {
        self.host.component.catalogue(names);
    }
    pub(super) fn down(&mut self, out: StoreOut) -> bool {
        if let StoreOut::Start { phase, .. } = &out {
            if self.restoring == Some(*phase) {
                assert_eq!(*phase, jig_domain_core::start::Phase::Sweep, "only sweep resumes a paused walk");
                return true;
            }
            assert!(self.restoring.is_none(), "one start walk at a time");
            self.restoring = Some(*phase);
        }
        if let Some(request) = route::down(&mut self.routes, out) {
            self.host.ask(request);
            true
        } else {
            false
        }
    }
    pub(super) fn work_pending(&self, now: Time) -> bool {
        self.host.work_pending(now) || !self.routed.is_empty() || !self.submitted.is_empty()
    }
    pub(super) fn closed(&self) -> bool {
        self.host.closed() && self.idle()
    }
    pub(super) fn idle(&self) -> bool {
        (self.host.closing() || self.routes.idle())
            && self.routed.is_empty()
            && self.submitted.is_empty()
            && self.host.idle()
    }
    pub(super) fn close(&mut self) {
        self.host.close();
        for _ in 0..self.routed.len() {
            let _up = self.routed.pop();
        }
    }
    pub(super) fn complete(&mut self, mut complete: Complete) {
        let index = crate::tokens::store_index(complete.op).expect("driver namespace");
        complete.op = Token::new(complete.op.untagged());
        if let Op::Cancel { target } = &mut complete.kind {
            *target = Token::new(target.untagged());
        }
        let lane = *Lane::ALL.get(index).expect("driver index");
        self.host.complete(lane, complete);
        self.publish();
    }
    fn publish(&mut self) {
        for _ in 0..self.host.submissions() {
            let protocol::host::Submission { lane, mut submit } = self.host.pop_submission().expect("host submissions");
            let namespace = *crate::tokens::STORE.get(lane.index()).expect("driver namespace");
            submit.op = crate::tokens::token(namespace, submit.op.raw());
            if let Op::Cancel { target } = &mut submit.kind {
                *target = crate::tokens::token(namespace, target.raw());
            }
            self.submitted.push(submit);
        }
    }
    pub(super) fn iterate(&mut self) {
        self.host.iterate();
        self.publish();
    }
}
pub(super) fn owns(token: Token) -> bool {
    crate::tokens::store_index(token).is_some()
}

pub(super) fn writes(mut queue: Queue<root::Write>) -> Box<[root::Write]> {
    let mut list = skein_lib::List::with_capacity(queue.len());
    for _ in 0..queue.len() {
        list.push(queue.pop().expect("journal write count")).expect("same writes");
    }
    list.into_boxed()
}
pub(super) fn request(engine: &mut crate::Engine, out: StoreOut) {
    if !engine.store.down(out) {
        crate::stop::invalid_restore(engine);
    }
}
pub(super) fn answer(engine: &mut crate::Engine) {
    if engine.end.is_some() {
        for _ in 0..engine.store.host.above.len() {
            let _late = engine.store.host.above.pop();
        }
        for _ in 0..engine.store.routed.len() {
            let _late = engine.store.routed.pop();
        }
        return;
    }
    for _ in 0..engine.store.host.above.len() {
        let event = engine.store.host.above.pop().expect("store terminals");
        match event {
            protocol::Event::Opened { installation, pending, .. } => {
                engine.store.installation = installation;
                engine.store.pending = pending;
                engine.store.opened = true;
            }
            event @ protocol::Event::Switched { .. } => {
                engine.store.pending = false;
                route::up(&mut engine.store.routes, event, &mut engine.store.routed);
            }
            // This shell prints no per-open progress lines.
            protocol::Event::Decoded { .. }
            | protocol::Event::Undecodable { .. }
            | protocol::Event::Rehearsing { .. }
            | protocol::Event::Back { .. }
            | protocol::Event::BackRefused { .. }
            | protocol::Event::Closed { .. } => unreachable!("an engine store never rehearses or goes back"),
            protocol::Event::Patched { .. } => {}
            protocol::Event::OpenRefused { why, .. } => {
                let failure = engine
                    .store
                    .host
                    .component
                    .start_open_failure()
                    .cloned()
                    .unwrap_or(protocol::restore::OpenFailure::Refused(why));
                engine.end = Some(crate::End::Stopped(crate::Stop::StoreOpen {
                    phase: engine.domain.core_mut().start_phase(),
                    failure,
                }));
            }
            event @ (protocol::Event::SecretsNotReady { owner }
            | protocol::Event::Kept { owner }
            | protocol::Event::Stale { owner }
            | protocol::Event::Erased { owner }
            | protocol::Event::Carried { owner, .. }
            | protocol::Event::Secret { owner, .. }
            | protocol::Event::SecretsLoaded { owner, .. }) => {
                if crate::accounts::owns(owner) {
                    engine.store.host.sources.push(event);
                } else {
                    engine.store.host.secrets.push(event);
                }
            }
            protocol::Event::Corrupt { .. } => {
                unreachable!("the service opens its store in start mode")
            }
            protocol::Event::PageRefused { failure, .. } => {
                engine.end = Some(crate::End::Stopped(crate::Stop::StorePage {
                    phase: engine.domain.core_mut().start_phase(),
                    failure,
                }));
            }
            // This shell never hands patch sets; routing answers Site with NoSite.
            other @ (protocol::Event::Page { .. }
            | protocol::Event::Site { .. }
            | protocol::Event::Committed { .. }
            | protocol::Event::Failed { .. }
            | protocol::Event::FrameTooLarge { .. }
            | protocol::Event::Loaded { .. }
            | protocol::Event::Read { .. }
            | protocol::Event::Got { .. }
            | protocol::Event::Turns { .. }
            | protocol::Event::Unread { .. }) => route::up(&mut engine.store.routes, other, &mut engine.store.routed),
        }
        if engine.end.is_some() {
            return;
        }
    }
    answer_routed(engine);
}

fn answer_routed(engine: &mut crate::Engine) {
    for _ in 0..engine.store.routed.len() {
        let up = engine.store.routed.pop().expect("store routing");
        // A sweep erase and its held continuation must finish before the next
        // row or page. Commit terminals still pass the paused walk.
        if engine.domain.core_mut().start_phase() == Some(jig_domain_core::start::Phase::Sweep)
            && !engine.domain.quiescent()
            && match &up {
                Up::Root(root::Event::Committed { .. } | root::Event::Failed { .. }) => false,
                Up::Ask(_) | Up::Root(_) => true,
            }
        {
            engine.store.routed.push(up);
            continue;
        }
        match up {
            Up::Root(root::Event::Failed { number }) => crate::stop::failed(engine, number),
            Up::Root(root::Event::StartDone(phase)) => {
                assert_eq!(engine.store.restoring.take(), Some(phase), "the selected walk finishes once");
                engine.events.push(root::Event::StartDone(phase));
            }

            Up::Root(root::Event::StartRecord(row)) => {
                if row.key() == root::Key::Core(jig_domain_core::Key::Core(jig_domain_core::CoreKey::Deployment)) {
                    engine.first_periods.truncate(0);
                }
                engine.root_steps = engine.root_steps.wrapping_add(1);
                root::start_record(&mut engine.domain, &engine.env, row);
            }
            Up::Root(event) => {
                engine.blocked = false;
                engine.step(event);
            }
            Up::Ask(request) => engine.store.host.ask(request),
        }
        if engine.domain.start_failure().is_some() || engine.domain.start_page_failure().is_some() {
            crate::stop::invalid_restore(engine);
        }
        if engine.end.is_some() {
            break;
        }
    }
}

#[cfg(test)]
mod sizing_tests {
    use jig_protocol_config::Profile;
    use jig_protocol_store::host::max_read;

    #[test]
    fn drivers_read_a_whole_page_in_every_admitted_store() {
        for profile in [Profile::Small, Profile::Standard] {
            let configured = crate::limits::profile_limits(profile);
            let limits = super::limits(&configured.root, &configured.store).expect("admitted profile");
            assert!(max_read(&limits) >= limits.store.page.bytes);
        }
    }
}
