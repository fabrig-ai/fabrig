//! Tiny world scripting the accounts component on the driver's terms.
//! Fixtures and acts follow jig/docs/design/domain/accounts.md, sections 3 and 5.
use jig_domain_accounts::{
    self as accounts, Answer, Domain, Ended, Event, Fact, Holder, Key, Kind, Limits, Offers, Provider, Providers,
    Refusal, Request, Stored, ToComponent,
};
use jig_domain_link::Worker;
use skein_lib::{Duration, Env, Queue, Time, Token, Wall};
use std::collections::BTreeMap;
pub const ANALYST: u32 = 7;
pub const PROVIDER: u32 = 1;
pub const ALICE_PARTY: u64 = 9;
pub const BOB_PARTY: u64 = 4;
pub const ALICE: Holder = Holder::Party(ALICE_PARTY);
pub const BOB: Holder = Holder::Party(BOB_PARTY);
pub const OWNER: u64 = 7;
pub const OTHER_PARTY: u64 = 8;
pub const HOLDER: Holder = Holder::Party(OWNER);
/// A holder keeps fewer accounts than the deployment, so that each bound is passed alone.
pub const LIMITS: Limits = Limits {
    accounts: 3,
    holder_accounts: 2,
    program_names: 12,
    program_hosts: 65,
    sources: 3,
    sign_ins: 1,
    credentials: 2,
    backoff_base: Duration::from_secs(2),
    backoff_max: Duration::from_secs(8),
    retry: Duration::from_secs(30),
    move_deadline: Duration::from_secs(60),
    spent_attention: Duration::from_secs(30),
    facts: 16,
};
#[derive(Debug)]
pub struct World {
    pub domain: Domain,
    pub limits: Limits,
    pub now: Time,
    /// The wall clock the child sees, for what the store keeps (engine.md, 5.4).
    pub wall: Wall,
}
impl Default for World {
    fn default() -> World {
        World::with_start(LIMITS, providers(vec![provider(PROVIDER, true, false)]), &source_rows(&[ANALYST]))
    }
}
impl World {
    /// Load stored rows before activation and linking.
    #[must_use]
    pub fn with_start(limits: Limits, providers: Providers, records: &[Stored]) -> World {
        let mut world = World { domain: Domain::new(&limits, providers), limits, now: Time::ZERO, wall: Wall::EPOCH };
        world.restore(records.to_vec());
        world
    }
    #[must_use]
    pub fn wanting(accounts: &[u32]) -> World {
        let mut world =
            Self::with_start(LIMITS, providers(vec![provider(PROVIDER, true, false)]), &source_rows(accounts));
        world.step(Event::Want);
        world
    }
    /// Restores each record, as the store gives them before the start.
    pub fn restore(&mut self, records: Vec<Stored>) {
        for record in records {
            assert_eq!(self.step(Event::Restore { record }), vec![]);
        }
    }
    #[must_use]
    pub fn views(&self) -> Vec<Fact> {
        self.domain.account_views().as_slice().to_vec()
    }
    pub fn step(&mut self, event: Event) -> Vec<Request> {
        let mut out = Queue::uncapped();
        accounts::step(&mut self.domain, &Env { now: self.now, wall: self.wall, limits: self.limits }, event, &mut out);
        drain(&mut out)
    }
    /// Fires at a time, the wall clock moving with the monotonic one.
    pub fn fire_at(&mut self, seconds: u64) -> Vec<Request> {
        let now = Time::ZERO.saturating_add(Duration::from_secs(seconds));
        self.wall = Wall::from_nanos(self.wall.as_nanos().saturating_add(now.saturating_since(self.now).as_nanos()));
        self.now = now;
        let mut out = Queue::uncapped();
        accounts::fire(&mut self.domain, &Env { now: self.now, wall: self.wall, limits: self.limits }, &mut out);
        drain(&mut out)
    }
    /// Every fact queued since the last call.
    pub fn facts(&mut self) -> Vec<Fact> {
        let mut result = Vec::new();
        while let Some(fact) = self.domain.pop_fact() {
            result.push(fact);
        }
        result
    }
    /// How each party's sign-in ended since the last call.
    pub fn ended(&mut self) -> Vec<Ended> {
        let mut result = Vec::new();
        while let Some(ended) = self.domain.pop_ended() {
            result.push(ended);
        }
        result
    }
}
/// Explicit durable rows for the source-state stories.
#[must_use]
pub fn source_rows(sources: &[u32]) -> Vec<Stored> {
    let mut rows = Vec::new();
    for (place, number) in sources.iter().enumerate() {
        let mut row = account(1_000 + number, PROVIDER, Holder::Deployment, b"");
        if let Stored::Account { place: at, .. } = &mut row {
            *at = u32::try_from(place).expect("bounded source places");
        }
        rows.push(row);
        rows.push(source(1_000 + number, *number, 0, Kind::SignIn));
    }
    rows
}
/// A provider serving one model, offering a sign-in, a key, or neither: the `none` source.
#[must_use]
pub fn provider(number: u32, sign_in: bool, key: bool) -> Provider {
    Provider {
        list: None,
        name: b"Fake".as_slice().into(),
        signed_in_plan: true,
        manage_usage: Some(b"https://usage.example/".as_slice().into()),
        number,
        offers: Offers { sign_in, key, programs: Box::new([]) },
    }
}
#[must_use]
pub fn providers(list: Vec<Provider>) -> Providers {
    Providers { providers: list.into_boxed_slice() }
}
#[must_use]
pub fn account(number: u32, provider: u32, holder: Holder, subject: &[u8]) -> Stored {
    Stored::Account {
        list: None,
        place: 0,
        uses: None,
        number,
        provider,
        holder,
        subject: subject.into(),
        display: Box::new([]),
    }
}
/// An account of the world's holder with its configured provider.
#[must_use]
pub fn held(number: u32, subject: &[u8]) -> Stored {
    account(number, PROVIDER, HOLDER, subject)
}
/// A program name reserved by the world's holder.
#[must_use]
pub fn issued(number: u32, name: &[u8]) -> Stored {
    Stored::Issued { number, holder: HOLDER, name: name.into(), signed_out: false }
}
#[must_use]
pub fn source(account: u32, number: u32, place: u32, kind: Kind) -> Stored {
    Stored::Source { account, number, place, kind, name: Box::new([]), held: None }
}
#[must_use]
pub fn program(account: u32, number: u32, place: u32, name: &[u8]) -> Stored {
    Stored::Source { account, number, place, kind: Kind::Program, name: name.into(), held: None }
}
fn drain(out: &mut Queue<Request>) -> Vec<Request> {
    let mut result = Vec::new();
    while let Some(request) = out.pop() {
        result.push(request);
    }
    result
}

/// A configured position for account lifecycle inputs; no channel observation is invented.
#[must_use]
pub fn worker(number: u32) -> Worker {
    let count = accounts::programs::PROGRAM_HOSTS;
    let names = (0..count).map(|index| format!("worker-{index}").into_bytes().into_boxed_slice()).collect();
    let membership = jig_domain_link::Membership::new(names, count, 32).expect("fixture membership");
    membership.worker(number).expect("configured fixture worker")
}

#[must_use]
pub fn answer(request: u64, answer: Answer) -> Request {
    Request::Answer { request: Token::new(request), answer }
}
#[must_use]
pub fn refused(request: u64, refusal: Refusal) -> Vec<Request> {
    vec![answer(request, Answer::Refused { refusal })]
}
#[must_use]
pub fn save(record: Stored) -> Request {
    Request::Save { record }
}
#[must_use]
pub fn want(source: u32) -> Request {
    Request::Component(ToComponent::Want { account: source })
}
/// The store commits saves by replacement and erases by key.
pub fn commit(store: &mut BTreeMap<Key, Stored>, requests: &[Request]) {
    for request in requests {
        match request {
            Request::Save { record } => drop(store.insert(record.key(), record.clone())),
            Request::Erase { key } => drop(store.remove(key)),
            Request::Programs { .. }
            | Request::ReadList { .. }
            | Request::CheckProgram { .. }
            | Request::SignOutProgram { .. }
            | Request::Component(_)
            | Request::Signing(_)
            | Request::Answer { .. }
            | Request::Granted { .. }
            | Request::Closed => {}
        }
    }
}
