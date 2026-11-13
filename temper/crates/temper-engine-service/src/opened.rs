//! Opened substrates and engine observations (temper/docs/design/protocol/services.md, 4).
use crate::{Engine, store::Store};
use alloc::boxed::Box;
use skein_lib::{Queue, Time, Wall};
use temper_engine_domain as root;
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
    pub fn submissions(&mut self) -> &mut Queue<skein_io::kernel::Submit> {
        &mut self.submitted
    }
}
impl Engine {
    /// Replace the store substrate before driving the component.
    pub fn replace_store(&mut self, store: Store) {
        self.store = store;
    }
    pub fn exchange_store(&mut self, store: Store) -> Store {
        core::mem::replace(&mut self.store, store)
    }
    pub fn store(&mut self) -> &mut Store {
        &mut self.store
    }
    #[must_use]
    pub fn limits(&self) -> &root::Limits {
        &self.env.limits
    }
    #[must_use]
    pub fn opening_limits(&self) -> jig_protocol_store::Limits {
        *self.store.host.limits()
    }
    pub fn input(&mut self, event: root::Event) {
        self.events.push(event);
    }
    #[must_use]
    pub fn observations(&self) -> &Queue<root::Event> {
        &self.events
    }
    #[must_use]
    pub fn workers_opened(&self) -> bool {
        self.workers.host.opened()
    }
    #[must_use]
    pub fn client_wire(&self) -> &jig_client_protocol_wire::v21::Limits {
        &self.clients.env.limits.wire
    }
    pub fn client_event(&mut self, event: skein_io::Event) -> bool {
        self.clients.up(event)
    }
    pub fn client_requests(&mut self) -> &mut Queue<skein_io::Request> {
        &mut self.clients.io
    }
    pub fn advance_clients(&mut self, now: Time, wall: Wall) {
        self.clients.iterate(now, wall, &mut self.events, &self.workers.forges, self.demo.as_ref(), &mut self.accounts);
        self.clients.secrets_step(&mut self.store);
    }
    /// Restore a kept forge before driving its runtime membership.
    pub fn restore_forge(&mut self, record: temper_engine_domain_forge::ForgeRecord) {
        root::step(&mut self.domain, &self.env, root::Event::Restart { pending: self.store.pending_generation });
        root::step(
            &mut self.domain,
            &self.env,
            root::Event::Forge(Box::new(temper_engine_domain_forge::Event::Restore {
                record: temper_engine_domain_forge::Stored::Forge(record.clone()),
            })),
        );
        crate::route::request(self, root::Request::ForgeKept(record));
    }
    #[must_use]
    pub fn forge_endpoint(&self, number: u16) -> Option<skein_io::Addr> {
        match self.forge.endpoints.get(&number) {
            Some(endpoint) => Some(endpoint.address),
            None => None,
        }
    }
    pub fn resolve_name(&mut self, name: Box<[u8]>, address: core::net::IpAddr) {
        self.forge.names.insert(name, address).expect("kept host capacity");
    }
    #[must_use]
    pub fn forge_address(&self, number: u16) -> Option<jig_protocol_config::Address> {
        if let Some(source) = self.workers.forges.get(number) {
            return Some(source.api.clone());
        }
        for record in &self.forge_records.pending {
            if record.number == number {
                return Some(jig_protocol_config::Address::Host {
                    name: record.place.api.host.clone(),
                    port: record.place.api.port,
                });
            }
        }
        None
    }
    pub fn remember_record(&mut self, number: u64, value: skein_lib::Secret, bytes: u32) {
        self.forge_records.rows.push(number, value, bytes);
    }
    #[must_use]
    pub fn deployment(&self) -> &crate::family::Deployment {
        &self.clients.deployment
    }
    /// Notes held by the joined drain consume this component's original sink credit.
    pub fn held_notes(&mut self, held: u32) {
        let capacity = self.note_bound.checked_sub(held).expect("joined notes share the engine sink bound");
        if self.notes.capacity() == capacity {
            return;
        }
        let mut notes = Queue::with_capacity(capacity);
        for _ in 0..self.notes.len() {
            notes.try_push(self.notes.pop().expect("engine note count")).expect("joined note credit");
        }
        self.notes = notes;
    }
    pub fn held_listening(&mut self, held: u32) {
        let capacity = 3_u32.checked_sub(held).expect("three listening notes");
        if self.listening.capacity() == capacity {
            return;
        }
        let mut notes = Queue::with_capacity(capacity);
        for _ in 0..self.listening.len() {
            notes.try_push(self.listening.pop().expect("engine listening count")).expect("joined listening credit");
        }
        self.listening = notes;
    }
}
