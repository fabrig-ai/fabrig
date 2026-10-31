//! The LLM-key component over the real secrets store and grant table.
use crate::fixtures;
use jig_domain_accounts as accounts;
use jig_protocol_accounts::{
    Component,
    component::{Effect, Event},
    table::Table,
};
use jig_protocol_store as store;
use skein_lib::{Env, Map, Queue, Secret, Time, Token, Wall};

pub struct World {
    component: Component,
    env: Env<skein_oauth_accounts::Limits>,
    effects: Queue<Effect>,
    pub table: Table,
    pub heard: Vec<accounts::FromComponent>,
    pub swept: Vec<u64>,
    pub loads: u32,
    store: store::Store,
    limits: store::Limits,
    above: Queue<store::Event>,
    below: store::Below,
}
impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
impl World {
    #[must_use]
    pub fn new() -> Self {
        let mut driver_limits = fixtures::limits();
        driver_limits.client.document.token_bytes = 32;
        Self::configured(driver_limits, Map::with_capacity(0))
    }
    #[must_use]
    pub fn configured(
        driver_limits: skein_oauth_accounts::Limits,
        providers: Map<u32, skein_oauth_accounts::Account>,
    ) -> Self {
        let mut limits = fixtures::store_limits();
        limits.secrets.page.rows = 2;
        let mut world = Self {
            component: Component::new(
                fixtures::MOVE_APPLICATION.into(),
                providers,
                driver_limits,
                skein_lib::CryptoRng::new(&skein_world::crypto_seed(2), 0).expect("seed"),
                limits.secrets.page.rows,
            ),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: driver_limits },
            effects: Queue::uncapped(),
            table: Table::new(driver_limits.accounts, driver_limits.client.document.token_bytes),
            heard: Vec::new(),
            swept: Vec::new(),
            loads: 0,
            store: store::Store::over(
                &limits,
                fixtures::APPLICATION,
                &[],
                store::Stores {
                    store: skein_kv::Store::memory(&limits.store),
                    secrets: skein_kv::Store::memory(&limits.secrets),
                },
            ),
            limits,
            above: Queue::uncapped(),
            below: store::Below {
                store: Queue::uncapped(),
                secrets: Queue::uncapped(),
                transcripts: Queue::uncapped(),
            },
        };
        let root = store::Root { token: Token::new(1), fd: skein_io::kernel::Fd::new(3) };
        world.down(store::Request::Open { owner: Token::new(1), root, installation: [5; 16] });
        let Some(store::Event::Opened { pending, owner, .. }) = world.above.pop() else { panic!("store opens") };
        if pending {
            // This store-only driver has no engine root to ask for the switch.
            world.down(store::Request::Switch { owner });
            assert!(matches!(world.above.pop(), Some(store::Event::Switched { .. })));
        }
        world.down(store::Request::Commit {
            number: 1,
            writes: Box::new([
                store::Write::Save(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Deployment(
                    jig_domain_core::Deployment {
                        id: [5; 16],
                        tasks: 0,
                        people: 0,
                        sign_ins: 0,
                        messages: 0,
                        runs: 0,
                        calls: 0,
                        connector_rows: 0,
                        projects: 0,
                        applications: 0,
                        commits: 1,
                    },
                ))),
                store::Write::Wires,
            ]),
        });
        assert!(matches!(world.above.pop(), Some(store::Event::Committed { number: 1, .. })));
        world.component.step(&world.env, Event::Opened { installation: [5; 16] }, &mut world.effects);
        world.pump();
        world
    }
    fn down(&mut self, request: store::Request) {
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: self.limits };
        store::down(&mut self.store, &env, request, &mut self.above, &mut self.below);
        assert!(self.below.store.is_empty() && self.below.secrets.is_empty(), "memory store");
    }
    pub fn secret(&mut self, request: store::Secrets) -> store::Event {
        self.down(store::Request::Secrets { owner: Token::new(1), request });
        let event = self.above.pop().expect("one secrets answer");
        assert!(self.above.is_empty());
        event
    }
    pub fn carry(&mut self, value: &[u8]) -> u64 {
        let store::Event::Carried { number, .. } = self.secret(store::Secrets::Carry {
            kind: jig_protocol_store::CarriedKind::Key,
            value: Secret::new(value.into()),
        }) else {
            panic!("carried LLM key")
        };
        number
    }
    pub fn get(&mut self, owner: store::SecretOwner, number: u64) -> Option<Secret> {
        let store::Event::Secret { value, .. } = self.secret(store::Secrets::Get { owner, number }) else {
            panic!("stored value")
        };
        value.map(|value| match value {
            store::SecretValue::Key(value)
            | store::SecretValue::Record(value)
            | store::SecretValue::Carried { kind: store::CarriedKind::Key, value } => value,
            other @ (store::SecretValue::Digest { .. }
            | store::SecretValue::Carried {
                kind: store::CarriedKind::Value | store::CarriedKind::Moved,
                ..
            }
            | store::SecretValue::Application(_)
            | store::SecretValue::Held(_)) => panic!("LLM key kind: {other:?}"),
        })
    }
    pub fn keep(&mut self, source: u32, carried: u64, replaces: bool) {
        self.component.step(
            &self.env,
            Event::Request(accounts::Request::Signing(accounts::Signing::Keep { source, carried, replaces })),
            &mut self.effects,
        );
        self.pump();
    }
    fn pump(&mut self) {
        for _ in 0..32 {
            self.component.fire(&self.env, &mut self.effects);
            while let Some(effect) = self.effects.pop() {
                match effect {
                    Effect::Store { exchange, request } => {
                        match &request {
                            store::Secrets::Erase { owner: store::SecretOwner::Source, number } => {
                                self.swept.push(*number)
                            }
                            store::Secrets::Load { .. } => self.loads += 1,
                            store::Secrets::Get { .. }
                            | store::Secrets::Adopt { .. }
                            | store::Secrets::AdoptValue { .. }
                            | store::Secrets::Keep { .. }
                            | store::Secrets::Replace { .. }
                            | store::Secrets::Hold { .. }
                            | store::Secrets::Carry { .. }
                            | store::Secrets::Erase { .. } => (),
                        }
                        self.down(store::Request::Secrets { owner: exchange, request });
                        let answer = self.above.pop().expect("store answer");
                        self.component.step(&self.env, Event::Store(answer), &mut self.effects);
                    }
                    Effect::Granted { source, generation, value, expires } => {
                        self.table.granted(source, generation, value, expires).expect("bounded key");
                    }
                    Effect::Child(accounts::Event::Component(event)) => self.heard.push(event),
                    Effect::Removed { source } => self.table.remove(source),
                    Effect::SourceRemoved { .. } => (),
                    Effect::Child(event) => panic!("unexpected child event: {event:?}"),
                    Effect::Io(_) | Effect::InvalidSecret => panic!("key uses only secrets"),
                }
            }
            if !self.component.has_work(self.env.now) {
                return;
            }
        }
        panic!("key exchange settles");
    }
    pub fn restore(&mut self, source: u32, provider: u32, kind: accounts::Kind) {
        self.component.step(
            &self.env,
            Event::Request(accounts::Request::Signing(accounts::Signing::Restore { source, provider, kind })),
            &mut self.effects,
        );
        self.pump();
    }
    pub fn want(&mut self, account: u32) {
        self.component.step(
            &self.env,
            Event::Request(accounts::Request::Component(accounts::ToComponent::Want { account })),
            &mut self.effects,
        );
        self.pump();
    }
    pub fn restored(&mut self) {
        self.component.step(
            &self.env,
            Event::Request(accounts::Request::Signing(accounts::Signing::Restored)),
            &mut self.effects,
        );
        self.pump();
    }
    pub fn restart(&mut self) {
        self.component = Component::new(
            fixtures::MOVE_APPLICATION.into(),
            Map::with_capacity(0),
            self.env.limits,
            skein_lib::CryptoRng::new(&skein_world::crypto_seed(3), 0).expect("seed"),
            self.limits.secrets.page.rows,
        );
        self.table = Table::new(self.env.limits.accounts, self.env.limits.client.document.token_bytes);
        self.heard.clear();
        self.swept.clear();
        self.loads = 0;
        self.component.step(&self.env, Event::Opened { installation: [5; 16] }, &mut self.effects);
        self.pump();
    }
    pub fn forget(&mut self, source: u32) {
        self.component.step(
            &self.env,
            Event::Request(accounts::Request::Signing(accounts::Signing::Forget { source })),
            &mut self.effects,
        );
        self.pump();
    }
}
