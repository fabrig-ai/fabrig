//! Bounded people state, task-derived inbox counts, request admission and restoration
//! (domain/people.md, sections 3–6). `step` handles root events; `fire` expires sign-ins.
//! The child keeps identities, roles, sign-ins and keyed outcomes, routes each
//! admitted flight once and waits for `Decided`. Its bounded volatile need references
//! count every live task projection independently of the entry cache. Root owns policy, task creation
//! and durability; this child never performs IO or retains credential secrets.

mod deletion;
mod ended;
mod inbox;
pub use ended::EndedMark;
pub(crate) mod acts;
mod applications;
mod asks;
mod parties;
mod requests;
use applications::admission;
pub use applications::{application_part, application_part_done};
pub(crate) use asks::{invalid_ask, valid_ask};
use asks::{note_pattern_bytes, project, valid_adoption};
use parties::{SignIn, apply_roles, deadline, end_signin, has_person, make_service, role, seed, signin, valid_roles};
pub(crate) use requests::{Answered, Pending};
use requests::{admit_ask, again, answer_decided, answer_expiry, answer_reservations, decided, refused, signed_out};
mod start;
use inbox::{ChatTotal, Need, Recent, conversation_seen_decision, mark_seen, prune_unheld_roles, replace_waiting};

use crate::{
    Again, Ask, Entry, Event, Fact, Holding, Identity, IdentityKey, InitialOwner, Key, Kind, Limits, Outcome,
    ProjectRole, Reply, Request, RequestKey, ResultRef, Role, Seed, Stored, Whom,
};
use alloc::boxed::Box;
use skein_lib::{Deadlines, Env, Id, List, Map, Queue, Set, Slab, Time, Token, Wall};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Start {
    NotStarted,
    Loading,
    Linked,
    Sweeping,
    Open,
    Failed,
}

/// Bounded secret-free identities, sign-ins/deadlines, project holdings, completed keys and
/// volatile flights and task-derived inbox references; knows no task content.
/// (domain/people.md, sections 2–6).
#[derive(Debug)]
pub struct Domain {
    deletions: Map<u32, crate::DeletionStep>,
    start: Start,
    start_at: Option<Wall>,
    restore_failure: Option<crate::RestoreFailure>,
    ended_witnesses: Set<(u64, u64)>,
    deployment_provider: u16,
    people: Map<u64, Identity>,
    results_seen: Map<u64, u64>,
    conversations_seen: Map<(u64, u64), u64>,
    replies: Map<u64, Box<[Entry]>>,
    chat_totals: Map<u64, ChatTotal>,
    chat_updates: Map<(u64, u32), u128>,
    result_census: Map<(u64, u32), Box<[ResultRef]>>,
    recent: Map<u64, Box<[Recent]>>,
    unread: Map<u64, Box<[ResultRef]>>,
    waiting: Map<Whom, Box<[Entry]>>,
    needs: Map<u64, Box<[Need]>>,
    identities: Map<IdentityKey, u64>,
    sign_ins: Map<u64, SignIn>,
    alarms: Deadlines<u64>,
    expiries: Set<(Wall, u64)>,
    roles: Map<u32, Box<[Holding]>>,
    /// The deployment's first owners, who decide its LLM accounts (accounts.md, 3.2) for as long
    /// as it runs; their first sign-ins seed their roles only while `seeding`.
    owners: Box<[InitialOwner]>,
    /// The deployment's first start, until the core begins a restore (protocol/configuration.md,
    /// 3: owners seed the first start only).
    seeding: bool,
    answers: Map<RequestKey, Answered>,
    acted: Map<crate::ActKey, acts::Acted>,
    act_alarms: Deadlines<crate::ActKey>,
    act_flights: Map<crate::ActKey, Id<Pending>>,
    grants: Box<[crate::ActGrant]>,
    answer_alarms: Deadlines<RequestKey>,
    pending: Slab<Pending>,
    flights: Map<RequestKey, Id<Pending>>,
    facts: Queue<Fact>,
    lost: u64,
    room_refusals: Queue<crate::CapacityRefusal>,
    room_refusals_lost: u64,
}

impl Domain {
    /// Kept deletions, restored at start (jig/docs/design/domain/people.md, 5.2).
    #[must_use]
    pub fn deletions(&self) -> &Map<u32, crate::DeletionStep> {
        &self.deletions
    }

    pub(crate) fn input(&mut self) {
        if self.start == Start::NotStarted {
            self.start = Start::Open;
        }
    }

    /// The admitted ask retained for one root route, including an asynchronous lookup.
    #[must_use]
    pub fn pending_ask(&self, request: Token) -> Option<&Ask> {
        let flight = self.party_pending(Id::from_token(request))?;
        Some(&flight.ask)
    }

    /// Create restoring state with capacities from validated `limits` and at most
    /// `limits.initial_owners` owned bootstrap entries. Panics on unrepresentable/invalid limits,
    /// oversized owners or duplicate project/identity pairs; project roles must be initialized
    /// before sign-in.
    #[must_use]
    pub fn new(
        limits: &Limits,
        owners: Box<[InitialOwner]>,
        deployment_provider: u16,
        grants: Box<[crate::ActGrant]>,
    ) -> Domain {
        assert!(crate::valid(limits), "people limits are valid");
        assert!(
            owners.len() <= usize::try_from(limits.initial_owners).expect("u32 fits usize"),
            "configured owners fit"
        );
        for (at, owner) in owners.iter().enumerate() {
            assert!(
                !owner.identity.subject.is_empty()
                    && owner.identity.subject.len() <= usize::try_from(limits.identity_bytes).expect("u32 fits usize")
                    && owner.identity.provider != deployment_provider,
                "configured first owners are bounded people"
            );
            for earlier in owners.iter().take(at) {
                assert!(
                    owner.project != earlier.project || owner.identity != earlier.identity,
                    "one initial owner identity per project"
                );
            }
        }
        assert!(
            grants.len() <= usize::try_from(limits.act_grants).expect("u32 fits usize"),
            "configured act grants fit"
        );
        for (at, grant) in grants.iter().enumerate() {
            assert!(grant.may.len() <= crate::ActKind::ALL.len(), "bounded configured act kinds");
            for earlier in grants.iter().take(at) {
                assert!(earlier.connector != grant.connector, "unique connector grants");
            }
            for (at, kind) in grant.may.iter().enumerate() {
                for earlier in grant.may.iter().take(at) {
                    assert!(earlier != kind, "unique configured act kinds");
                }
            }
        }
        Domain {
            deletions: Map::with_capacity(limits.projects),
            start: Start::NotStarted,
            start_at: None,
            restore_failure: None,
            ended_witnesses: Set::with_capacity(
                limits.people.checked_mul(limits.inbox_tasks).expect("bounded conversation witnesses"),
            ),
            deployment_provider,
            people: Map::with_capacity(limits.people),
            results_seen: Map::with_capacity(limits.people),
            conversations_seen: Map::with_capacity(
                limits.people.checked_mul(limits.inbox_tasks).expect("bounded conversation positions"),
            ),
            replies: Map::with_capacity(limits.inbox_tasks),
            chat_totals: Map::with_capacity(limits.inbox_tasks),
            chat_updates: Map::with_capacity(limits.people.checked_mul(limits.projects).expect("project reply census")),
            result_census: Map::with_capacity(
                limits.people.checked_mul(limits.projects).expect("project result census"),
            ),
            recent: Map::with_capacity(limits.people),
            unread: Map::with_capacity(limits.people),
            needs: Map::with_capacity(limits.inbox_tasks),
            waiting: Map::with_capacity(
                limits
                    .people
                    .checked_add(limits.projects.checked_mul(limits.holdings).expect("role cache room"))
                    .expect("inbox cache room"),
            ),
            identities: Map::with_capacity(limits.people),
            sign_ins: Map::with_capacity(limits.sign_ins),
            alarms: Deadlines::with_capacity(limits.sign_ins),
            expiries: Set::with_capacity(limits.sign_ins),
            roles: Map::with_capacity(limits.projects),
            owners,
            seeding: true,
            answers: Map::with_capacity(limits.requests),
            acted: Map::with_capacity(limits.requests),
            act_alarms: Deadlines::with_capacity(limits.requests),
            act_flights: Map::with_capacity(limits.pending),
            grants,
            answer_alarms: Deadlines::with_capacity(limits.requests),
            pending: Slab::with_capacity(limits.pending),
            flights: Map::with_capacity(limits.pending),
            facts: Queue::with_capacity(limits.facts),
            lost: 0,
            room_refusals: Queue::with_capacity(limits.room_refusals),
            room_refusals_lost: 0,
        }
    }

    /// Pure scheduler query of the earliest armed monotonic sign-in expiry, or `None`; no
    /// expiration or output occurs here.
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        let ordinary = match (self.alarms.next(), self.answer_alarms.next()) {
            (Some(left), Some(right)) => Some(left.min(right)),
            (Some(left), None) => Some(left),
            (None, Some(right)) => Some(right),
            (None, None) => None,
        };
        match (ordinary, self.act_alarms.next()) {
            (Some(left), Some(right)) => Some(left.min(right)),
            (Some(at), None) | (None, Some(at)) => Some(at),
            (None, None) => None,
        }
    }

    /// Earliest saved wall expiry; translate beside the original monotonic
    /// deadline at the caller's current injected clocks (people.md, section 3).
    #[must_use]
    pub fn next_wall_expiry(&self) -> Option<Wall> {
        let (expires, _) = self.expiries.first()?;
        Some(*expires)
    }

    /// Timeless table membership only; makes no claim of clock validity.
    /// Authentication must use `person` with the current clocks.
    #[must_use]
    pub fn sign_in_person(&self, sign_in: u64) -> Option<u64> {
        Some(self.sign_ins.get(&sign_in)?.person)
    }

    /// Pure scheduler query returning whether any armed expiry is at or before supplied `now`; no
    /// state change or output.
    #[must_use]
    pub fn is_due(&self, now: Time) -> bool {
        match self.next_deadline() {
            Some(at) => at <= now,
            None => false,
        }
    }

    /// Reclaim retired flight slots at the parent's iteration reclaim point after their outputs
    /// have been routed; no request or client reply is emitted.
    pub fn reclaim(&mut self) {
        self.pending.reclaim();
    }

    /// Remove one optional content-free observation from the bounded fact queue; keeping/dropping
    /// facts changes no decision or persistence output.
    pub fn pop_fact(&mut self) -> Option<Fact> {
        self.facts.pop()
    }

    /// Return the saturating count of observations dropped when `Limits::facts` was full;
    /// diagnostic only, not a lifecycle acknowledgement.
    #[must_use]
    pub const fn facts_lost(&self) -> u64 {
        self.lost
    }

    /// The next retained Answer outcome in key order for Link (jig/docs/design/domain/engine.md, 6).
    #[must_use]
    pub fn next_answer(&self, after: Option<RequestKey>) -> Option<(RequestKey, &Outcome)> {
        let next = match after {
            Some(key) => self.answers.range((core::ops::Bound::Excluded(key), core::ops::Bound::Unbounded)).next(),
            None => self.answers.first(),
        };
        match next {
            Some((key, answer)) => Some((*key, &answer.outcome)),
            None => None,
        }
    }

    /// Recent permanent room refusals from retained keyed answers, together with
    /// bounded unsaved entrance pressure, newest wall time first. Reads copy no ask
    /// payloads and change no diagnostic, replay or expiry state.
    #[must_use]
    pub fn capacity_refusals(&self) -> List<crate::CapacityRefusal> {
        let capacity =
            self.answers.capacity().checked_add(self.room_refusals.capacity()).expect("accepted room limits");
        let mut ordered = Map::with_capacity(capacity);
        let mut ordinal = 0_u32;
        for (key, answer) in &self.answers {
            if let Some(cause) = crate::capacity::permanent(&answer.outcome) {
                let action = crate::capacity::action(&answer.ask, key.person);
                let value = crate::CapacityRefusal {
                    key: *key,
                    project: crate::capacity::project(&answer.ask),
                    action,
                    at: answer.at,
                    cause,
                };
                ordered.insert((value.at, ordinal), value).expect("bounded permanent refusal snapshot");
                ordinal = ordinal.checked_add(1).expect("bounded refusal ordinal");
            }
        }
        for value in &self.room_refusals {
            ordered.insert((value.at, ordinal), *value).expect("bounded entrance refusal snapshot");
            ordinal = ordinal.checked_add(1).expect("bounded refusal ordinal");
        }
        let mut result = List::with_capacity(capacity);
        for (_, value) in ordered.iter().rev() {
            result.push(*value).expect("bounded refusal view");
        }
        result
    }

    /// Unsaved entrance-pressure records evicted from the bounded live history.
    /// Permanent answers instead follow their configured request retention.
    #[must_use]
    pub const fn capacity_refusals_lost(&self) -> u64 {
        self.room_refusals_lost
    }

    /// Pure bounded membership lookup used by root escalation routing and authenticated named
    /// reads; caller separately authenticates identities. Available during restore,
    /// allocating/emitting/mutating nothing.
    #[must_use]
    pub fn role(&self, person: u64, project: u32) -> Option<Role> {
        role(self, person, project)
    }

    /// Borrow one retained authenticated party's display identity; no secret is known.
    #[must_use]
    pub fn identity(&self, party: u64) -> Option<&Identity> {
        if !self.ready() && !self.start_lookup() {
            return None;
        }
        self.people.get(&party)
    }

    /// Classify a retained party by its authenticated provider.
    #[must_use]
    pub fn kind(&self, party: u64) -> Option<Kind> {
        let identity = self.identity(party)?;
        Some(self.kind_of(identity))
    }

    fn kind_of(&self, identity: &Identity) -> Kind {
        if identity.key.provider == self.deployment_provider { Kind::Service } else { Kind::Person }
    }

    /// Ordinary project roles of a retained party, bounded by the role table; no admission changes.
    /// The deployment’s roster is read separately through `role` (people.md, 3).
    #[must_use]
    pub fn project_roles(&self, party: u64) -> Option<List<ProjectRole>> {
        self.identity(party)?;
        let mut result = List::with_capacity(self.roles.capacity());
        for (project, holdings) in &self.roles {
            for holding in holdings {
                if *project != crate::DEPLOYMENT && holding.person == party {
                    result.push(ProjectRole { project: *project, role: holding.role }).expect("one role per project");
                }
            }
        }
        Some(result)
    }

    /// Whether another project roster fits, including the deployment roster.
    #[must_use]
    pub fn project_room(&self) -> bool {
        self.roles.len() < self.roles.capacity()
    }

    /// Borrow the retained role holders of one project without changing admission.
    #[must_use]
    pub fn project_members(&self, project: u32) -> Option<&[Holding]> {
        if !self.ready() && !self.start_lookup() {
            return None;
        }
        Some(self.roles.get(&project)?)
    }

    /// Resolve a provider-authenticated collaborator to this project's role.
    #[must_use]
    pub fn role_for_identity(&self, key: IdentityKey, project: u32) -> Option<(u64, Role)> {
        let person = *self.identities.get(&key)?;
        Some((person, role(self, person, project)?))
    }

    /// Check that one whole adoption seed can commit without displacing current roles.
    #[must_use]
    pub fn can_seed(&self, limits: &Limits, project: u32, collaborators: &[Seed]) -> bool {
        if !self.ready() || collaborators.len() > usize::try_from(limits.holdings).expect("u32 fits usize") {
            return false;
        }
        let Some(holdings) = self.roles.get(&project) else { return false };
        let mut new_people = 0_usize;
        let mut new_roles = 0_usize;
        for (at, seed) in collaborators.iter().enumerate() {
            if seed.identity.subject.is_empty()
                || seed.identity.subject.len() > usize::try_from(limits.identity_bytes).expect("u32 fits usize")
                || seed.identity.provider == self.deployment_provider
                || seed.candidate == 0
                || self.people.contains_key(&seed.candidate)
            {
                return false;
            }
            for earlier in collaborators.iter().take(at) {
                if earlier.identity == seed.identity || earlier.candidate == seed.candidate {
                    return false;
                }
            }
            match self.identities.get(&seed.identity) {
                Some(person) => {
                    if !has_person(holdings, *person) {
                        new_roles = new_roles.saturating_add(1);
                    }
                }
                None => {
                    new_people = new_people.saturating_add(1);
                    new_roles = new_roles.saturating_add(1);
                }
            }
        }
        match (
            self.people.len().checked_add(u32::try_from(new_people).ok().unwrap_or(u32::MAX)),
            holdings.len().checked_add(new_roles),
        ) {
            (Some(people), Some(roles)) => {
                people <= limits.people && roles <= usize::try_from(limits.holdings).expect("u32 fits usize")
            }
            (None, _) | (_, None) => false,
        }
    }

    /// End the first owners' seeding when the core begins restore (protocol/configuration.md,
    /// 3: owners seed the first start only); they stay the deployment's first owners.
    pub fn end_seeding(&mut self) {
        self.input();
        self.seeding = false;
    }

    /// Pure lookup of whether the bounded role table contains `project`, including an empty
    /// holdings set. Available during restoration so the root preserves restored roles and seeds
    /// only missing bootstrap projects; it implies neither readiness nor person membership. The
    /// table is bounded by `Limits::projects`; this query allocates nothing, emits no output and
    /// grants no role.
    #[must_use]
    pub fn has_project(&self, project: u32) -> bool {
        self.roles.contains_key(&project)
    }

    /// Pure authenticated lookup for the root-issued `sign_in` at supplied `now` and `wall`.
    /// Returns `None` before readiness, for an absent sign-in, or when either its projected
    /// monotonic deadline is at or before `now` or its saved wall expiry is at or before `wall`. A
    /// backward wall correction cannot revive a monotonic-expired session, even before fire.
    /// Otherwise returns the stable requester person number. The sign-in table is bounded by
    /// `Limits::sign_ins`; this read allocates nothing, emits no output and changes no sign-in or
    /// role state. Root uses it for named task result reads.
    #[must_use]
    pub fn person(&self, sign_in: u64, now: Time, wall: Wall) -> Option<u64> {
        if !self.ready() {
            return None;
        }
        let session = self.sign_ins.get(&sign_in)?;
        if session.expires <= wall || session.due <= now {
            return None;
        }
        Some(session.person)
    }

    /// Whether ordinary input is admitted: before a start or after it opens, never during
    /// loading, linking or sweeping, or after a failed start.
    #[must_use]
    pub fn ready(&self) -> bool {
        match self.start {
            Start::NotStarted | Start::Open => true,
            Start::Loading | Start::Linked | Start::Sweeping | Start::Failed => false,
        }
    }
}

/// The most outputs one step emits.
/// Bootstrap writes one roles record per configured owner match, alongside person/sign-in records
/// and reply, plus one expired sign-in's erase; completion answers every bounded waiter.
/// Internal output queues are uncapped; callers count output payload bytes separately.
#[must_use]
pub fn outputs_bound(limits: &Limits) -> u32 {
    limits
        .initial_owners
        .saturating_add(4)
        .max(limits.waiters.saturating_add(2))
        .max(limits.sign_ins.saturating_add(1))
        .max(limits.holdings.saturating_add(1))
        .max(limits.people.saturating_add(1))
        .max(limits.requests)
}

/// Apply one root-issued `event` with iteration clocks and immutable configured bounds in `env`
/// to an uncapped `out` queue. Emits
/// routing/persistence/replies, never IO; root completes each `Route` once and withholds replies
/// until required atomic writes are durable. Restore loads decoded rows within representation bounds;
/// `start_link` checks its project/person references without rechecking current membership or
/// requiring the historical roster to equal the current one. Refused role asks retain their invalid
/// targets for keyed replay.
pub fn step(domain: &mut Domain, env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
    domain.input();
    match event {
        Event::Deleting { project, step } => deletion::keep(domain, project, step, out),
        Event::CloseProject { project } => deletion::close(domain, project, out),
        Event::ForgetProject { project } => deletion::forget(domain, project, out),
        Event::Act { reply_to, connector, key, act } => {
            acts::admit(domain, env, reply_to, crate::ActKey { connector, key }, *act, out);
        }
        Event::MoveChat { task, project, from, to } => {
            inbox::move_chat(domain, &env.limits, task, project, from, to, out);
        }
        Event::ChatState { party, project, task, last_reply, live, seen, replies } => {
            domain.assert_projection_event();
            inbox::chat_state_event(domain, env, party, project, task, last_reply, live, seen, replies, out);
        }
        Event::ConversationSeen { request, project, last_reply } => {
            conversation_seen_decision(domain, env, request, project, last_reply, out);
        }
        Event::Forget { task, project, party, last_reply } => {
            inbox::forget_event(domain, env, task, project, party, last_reply, out);
        }
        Event::MakeService { request, person } => make_service(domain, env, request, person, out),
        Event::Seed { project, collaborators } => seed(domain, &env.limits, project, &collaborators, out),
        Event::Waiting { task, entries } => {
            domain.assert_projection_event();
            replace_waiting(domain, &env.limits, task, &entries);
        }
        Event::ApplyRoles { reply_to, request } => {
            let result = apply_roles(domain, env, request, out);
            out.push(Request::RolesApplied { reply_to, request, result });
        }
        Event::Restore { record } => start::load(domain, env, record, out),
        Event::SignedIn { reply_to, person, sign_in, identity, kind, held } => {
            signin(domain, env, reply_to, person, sign_in, identity, kind, held, out);
        }
        Event::Whose { reply_to, sign_in } => {
            if !domain.ready() {
                return again(reply_to, Again::NotReady, out);
            }
            let signer = match domain.person(sign_in, env.now, env.wall) {
                Some(person) => {
                    let identity = domain.people.get(&person).expect("live sign-in has a party");
                    Some(crate::Signer { identity: identity.clone(), kind: domain.kind_of(identity) })
                }
                None => None,
            };
            out.push(Request::Reply { to: reply_to, reply: Reply::Signer(signer) });
        }
        Event::SignOut { reply_to, sign_in } => {
            if !domain.ready() {
                return again(reply_to, Again::NotReady, out);
            }
            end_signin(domain, sign_in, out);
            out.push(Request::Reply { to: reply_to, reply: Reply::SignedOut });
        }
        Event::Roles { project, holdings } => {
            if !domain.ready() {
                return;
            }
            match valid_roles(domain, &env.limits, project, &holdings) {
                Ok(()) => {
                    out.push(Request::Save { record: Stored::Roles { project, holdings: holdings.clone() } });
                    let saved = domain.roles.insert(project, holdings);
                    assert!(saved.is_ok(), "roles admitted before mutation");
                    prune_unheld_roles(domain, project);
                }
                Err(outcome) => out.push(Request::RolesRefused { project, outcome }),
            }
        }
        Event::Ask { reply_to, sign_in, key, ask, models } => {
            admit_ask(domain, env, reply_to, sign_in, key, ask, models, out);
        }
        Event::Decided { request, outcome } => decided(domain, env, Id::<Pending>::from_token(request), outcome, out),
    }
}

/// One expiration per fire; other expirations stay due for later iterations. Expire at most one
/// sign-in due by either injected clock, emitting its `Erase`; no work before ready. Caller supplies
/// an uncapped output queue and calls again in later iterations while due; parent owns durability
/// and secret cleanup.
pub fn fire(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    domain.input();
    if !domain.ready() {
        return;
    }
    if let Some(key) = domain.act_alarms.expire(env.now) {
        assert!(domain.acted.remove(&key).is_some(), "acted alarm has its row");
        out.push(Request::Erase { key: Key::Acted(key) });
    } else if let Some(key) = domain.answer_alarms.expire(env.now) {
        let removed = domain.answers.remove(&key);
        assert!(removed.is_some(), "retained answer has its expiry alarm");
        out.push(Request::Erase { key: Key::Answer(key) });
    } else if let Some(number) = domain.alarms.expire(env.now) {
        end_signin(domain, number, out);
    } else if let Some(&(expires, number)) = domain.expiries.first()
        && expires <= env.wall
    {
        end_signin(domain, number, out);
    }
}

fn fact(domain: &mut Domain, observation: Fact) {
    if domain.facts.try_push(observation).is_err() {
        domain.lost = domain.lost.saturating_add(1);
    }
}
