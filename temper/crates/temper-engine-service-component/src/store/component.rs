//! The component's deployment store (protocol/services.md, 4.2).
use super::{KERNEL_HANDOFF_ENTRIES, PATCH_SETS, WIRES, route};
use jig_protocol_store as protocol;
use protocol::host::{Hosted, Lane, NotReady};
use route::Up;
use skein_io::kernel::{Complete, Fd, Op, Submit};
use skein_lib::{Queue, Time, Token};
use temper_engine_domain as root;

const MASK: u64 = (1_u64 << 60) - 1;
#[derive(Debug)]
pub struct Store {
    #[cfg(test)]
    pub(crate) forge_loads: skein_lib::List<Option<u64>>,
    #[cfg(test)]
    pub(crate) erases: u32,
    #[cfg(test)]
    pub(crate) hold_mark: bool,
    #[cfg(test)]
    pub(crate) held_mark: Option<root::StoreRequest>,
    #[cfg(test)]
    pub(crate) secret_requests: u32,
    pub(crate) host: Hosted,
    pub(crate) routes: route::Routes,
    pub(crate) routed: Queue<Up>,
    pub(crate) submitted: Queue<Submit>,
    pub(crate) opened: bool,
    pub(crate) pending_generation: bool,
    /// The installation's identifier: the drawn one until the store opens,
    /// then the one it keeps (jig's protocol/store/README.md 4.4).
    pub(crate) installation: [u8; 16],
}

impl Store {
    #[must_use]
    pub fn opening_limits(&self) -> protocol::Limits {
        *self.host.limits()
    }

    #[must_use]
    pub fn new(
        application: &root::Limits,
        limits: &protocol::Limits,
        models: &[jig_domain_core::Model],
        installation: [u8; 16],
    ) -> Self {
        let routes = route::Routes::new(
            application.loads.loads,
            application,
            temper_protocol_store::bounds(&application.forge, application.tasks.spec_bytes)
                .expect("checked family bounds"),
        );
        let application_sizes = temper_protocol_store::application(&application.forge, application.tasks.spec_bytes)
            .expect("checked application sizes");
        let seed = u64::from_le_bytes(*installation.first_chunk::<8>().expect("installation seed"));
        let mut host = Hosted::new(limits, application_sizes, models, seed, NotReady::Above);
        assert!(host.component.wires(&WIRES), "valid engine wires before open");
        host.component.patch_sets(PATCH_SETS);
        host.component.keyspaces(temper_protocol_store::KEYSPACES);
        Self {
            #[cfg(test)]
            forge_loads: skein_lib::List::with_capacity(128),
            #[cfg(test)]
            erases: 0,
            #[cfg(test)]
            hold_mark: false,
            #[cfg(test)]
            held_mark: None,
            #[cfg(test)]
            secret_requests: 0,
            routes,
            routed: Queue::uncapped(),
            host,
            submitted: Queue::with_capacity(KERNEL_HANDOFF_ENTRIES),
            opened: false,
            pending_generation: false,
            installation,
        }
    }
    /// Replace the binary's sets before adopting its deployment directory.
    pub(crate) fn patch_sets(&mut self, sets: protocol::PatchSets) {
        self.host.component.patch_sets(sets);
    }
    /// Whether the component has reported that its deployment opened.
    #[must_use]
    pub const fn opened(&self) -> bool {
        self.opened
    }
    pub fn adopt(&mut self, fd: Fd, user: u32) -> Result<(), Fd> {
        self.host.adopt(fd, user, self.installation)
    }
    pub fn down(&mut self, out: root::StoreRequest) -> bool {
        let out = match out {
            root::StoreRequest::Catalogue { names } => {
                self.host.component.catalogue(names);
                return true;
            }
            out @ (root::StoreRequest::Switch
            | root::StoreRequest::Commit { .. }
            | root::StoreRequest::Load { .. }
            | root::StoreRequest::Read { .. }) => out,
        };
        #[cfg(test)]
        if self.hold_mark
            && let root::StoreRequest::Commit { writes, .. } = &out
            && writes.contains(&root::Write::Wires)
        {
            assert!(self.held_mark.is_none(), "one held startup mark");
            self.held_mark = Some(out);
            return true;
        }
        if let Some(request) = route::down(&mut self.routes, out) {
            self.host.ask(request);
            true
        } else {
            false
        }
    }
    /// Get answers stay in this pass; secret writes' terminals are handed to their owner.
    pub fn secrets(&mut self, owner: Token, request: protocol::Secrets) -> Option<protocol::Event> {
        #[cfg(test)]
        {
            self.secret_requests = self.secret_requests.checked_add(1).expect("test secret requests");
        }
        #[cfg(test)]
        if owner == crate::forge::records::LOAD
            && let protocol::Secrets::Load { after, .. } = &request
        {
            self.forge_loads.push(*after).expect("test page trace");
        }
        #[cfg(test)]
        if let protocol::Secrets::Erase { .. } = &request {
            self.erases = self.erases.checked_add(1).expect("test erases");
        }
        self.host.secrets(owner, request)
    }
    #[must_use]
    pub fn work_pending(&self, now: Time) -> bool {
        self.host.work_pending(now) || !self.routed.is_empty() || !self.submitted.is_empty()
    }
    #[must_use]
    pub fn closed(&self) -> bool {
        self.host.closed() && self.idle()
    }
    #[must_use]
    pub fn idle(&self) -> bool {
        (self.host.closing() || self.routes.idle())
            && self.routed.is_empty()
            && self.submitted.is_empty()
            && self.host.idle()
    }
    pub fn close(&mut self) {
        self.host.close();
        for _ in 0..self.routed.len() {
            let _up = self.routed.pop();
        }
    }
    pub fn complete(&mut self, mut complete: Complete) {
        let index = usize::try_from((complete.op.raw() >> 60_u32).checked_sub(8).expect("driver namespace"))
            .expect("three drivers");
        complete.op = Token::new(complete.op.raw() & MASK);
        if let Op::Cancel { target } = &mut complete.kind {
            *target = Token::new(target.raw() & MASK);
        }
        let lane = *Lane::ALL.get(index).expect("driver index");
        self.host.complete(lane, complete);
        self.publish();
    }
    fn publish(&mut self) {
        for _ in 0..self.host.submissions() {
            let protocol::host::Submission { lane, mut submit } = self.host.pop_submission().expect("host submissions");
            let prefix = *crate::tokens::STORE.get(lane.index()).expect("driver prefix");
            submit.op = Token::new(submit.op.raw() | prefix);
            if let Op::Cancel { target } = &mut submit.kind {
                *target = Token::new(target.raw() | prefix);
            }
            self.submitted.push(submit);
        }
    }
    pub fn iterate(&mut self) {
        self.host.iterate();
        self.publish();
    }
}
impl Store {
    /// Use an opened store and the facts its Open terminal gave, retaining checked routes and drivers.
    #[must_use]
    pub fn on_opened(mut self, component: jig_protocol_store::Store, installation: [u8; 16], pending: bool) -> Self {
        self.host.component = component;
        self.opened = true;
        self.pending_generation = pending;
        self.installation = installation;
        self
    }
    pub fn exchange_component(&mut self, component: jig_protocol_store::Store) -> jig_protocol_store::Store {
        core::mem::replace(&mut self.host.component, component)
    }
    pub fn ask(&mut self, request: jig_protocol_store::Request) {
        self.host.ask(request);
    }
    pub fn events(&mut self) -> &mut Queue<jig_protocol_store::Event> {
        &mut self.host.above
    }
    #[must_use]
    pub fn mark_durable(&self) -> bool {
        self.host.mark_durable()
    }
    pub fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submitted
    }
}
