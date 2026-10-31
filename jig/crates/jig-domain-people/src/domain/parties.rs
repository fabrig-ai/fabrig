//! Parties, sign-ins, project roles and restart census (domain/people.md, sections 3–4).
use super::{Domain, again, fact, prune_unheld_roles, refused, signed_out};
use crate::{
    Again, Ask, Fact, Holding, Identity, IdentityKey, InitialOwner, Key, Kind, LimitKind, Limits, Outcome, Refusal,
    Reply, Request, Role, Seed, Stored,
};
use alloc::boxed::Box;
use skein_lib::{Env, Id, List, Queue, ReplyTo, Time, Token, Wall};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct SignIn {
    pub(super) person: u64,
    pub(super) expires: Wall,
    pub(super) due: Time,
}

impl Domain {
    /// People census for the host rule (accounts.md, 3.4); restoration is conservative.
    #[must_use]
    pub fn persons(&self) -> u32 {
        if !self.ready() {
            return u32::MAX;
        }
        let mut count = 0_u32;
        for (_, identity) in &self.people {
            if self.kind_of(identity) == Kind::Person {
                count = count.saturating_add(1);
            }
        }
        count
    }
    #[must_use]
    pub fn parties(&self) -> List<u64> {
        let mut parties = List::with_capacity(self.people.len());
        if self.ready() || self.start_lookup() {
            for (party, _) in &self.people {
                parties.push(*party).expect("one entry per stored identity");
            }
        }
        parties
    }
}

pub(super) fn valid_identity(limits: &Limits, identity: &Identity) -> bool {
    let Some(first) = identity.key.subject.len().checked_add(identity.login.len()) else {
        return false;
    };
    match first.checked_add(identity.name.len()) {
        Some(bytes) => {
            !identity.key.subject.is_empty() && bytes <= usize::try_from(limits.identity_bytes).expect("u32 fits usize")
        }
        None => false,
    }
}

pub(super) fn apply_roles(
    domain: &mut Domain,
    env: &Env<Limits>,
    request: Token,
    out: &mut Queue<Request>,
) -> Result<(), Outcome> {
    if !domain.ready() {
        return Err(Outcome::Again(Again::NotReady));
    }
    let flight = domain.party_pending(Id::from_token(request)).ok_or(Outcome::Refused(Refusal::Unknown))?;
    let project = match flight.part.as_ref().unwrap_or(&flight.ask) {
        Ask::SetRoles { project, .. } => *project,
        Ask::Compound { .. }
        | Ask::MarkSeen { .. }
        | Ask::Application { .. }
        | Ask::MakeService { .. }
        | Ask::EditNote { .. }
        | Ask::Adopt { .. }
        | Ask::ChangePolicy { .. }
        | Ask::RemoveProject { .. }
        | Ask::Rename { .. }
        | Ask::MakeProject { .. }
        | Ask::ChangeSettings { .. }
        | Ask::MoveToNewest { .. }
        | Ask::SetPool { .. }
        | Ask::DecideEscalation { .. }
        | Ask::DecideProposal { .. }
        | Ask::Say { .. }
        | Ask::AnswerQuestion { .. }
        | Ask::Amend { .. }
        | Ask::Move { .. }
        | Ask::TakePerson { .. }
        | Ask::HandBackPerson { .. }
        | Ask::AnswerPerson { .. }
        | Ask::Stop { .. }
        | Ask::Cancel { .. }
        | Ask::Lift { .. }
        | Ask::Release { .. }
        | Ask::Create { .. }
        | Ask::Propose { .. }
        | Ask::Withdraw { .. }
        | Ask::AccountSignIn { .. }
        | Ask::AccountSignOut { .. }
        | Ask::AccountChange { .. } => {
            return Err(Outcome::Refused(Refusal::Unknown));
        }
    };
    if !domain.roles.contains_key(&project) {
        return Err(Outcome::Refused(Refusal::Unknown));
    }
    if role(domain, flight.key.person, project) != Some(Role::Owner) {
        return Err(Outcome::Refused(Refusal::Role { held: role(domain, flight.key.person, project) }));
    }
    let holdings = match flight.part.as_ref().unwrap_or(&flight.ask) {
        Ask::SetRoles { holdings, .. } => {
            valid_roles(domain, &env.limits, project, holdings)?;
            for holding in holdings {
                if holding.person == 0 {
                    return Err(Outcome::Refused(Refusal::Limit { which: LimitKind::Roles }));
                }
                if !domain.people.contains_key(&holding.person) {
                    return Err(Outcome::Refused(Refusal::Unknown));
                }
            }
            holdings.clone()
        }
        Ask::Compound { .. }
        | Ask::MarkSeen { .. }
        | Ask::Application { .. }
        | Ask::MakeService { .. }
        | Ask::EditNote { .. }
        | Ask::Adopt { .. }
        | Ask::ChangePolicy { .. }
        | Ask::RemoveProject { .. }
        | Ask::Rename { .. }
        | Ask::MakeProject { .. }
        | Ask::ChangeSettings { .. }
        | Ask::MoveToNewest { .. }
        | Ask::SetPool { .. }
        | Ask::DecideEscalation { .. }
        | Ask::DecideProposal { .. }
        | Ask::Say { .. }
        | Ask::AnswerQuestion { .. }
        | Ask::Amend { .. }
        | Ask::Move { .. }
        | Ask::TakePerson { .. }
        | Ask::HandBackPerson { .. }
        | Ask::AnswerPerson { .. }
        | Ask::Stop { .. }
        | Ask::Cancel { .. }
        | Ask::Lift { .. }
        | Ask::Release { .. }
        | Ask::Create { .. }
        | Ask::Propose { .. }
        | Ask::Withdraw { .. }
        | Ask::AccountSignIn { .. }
        | Ask::AccountSignOut { .. }
        | Ask::AccountChange { .. } => {
            unreachable!("validated roster flight")
        }
    };
    out.push(Request::Save { record: Stored::Roles { project, holdings: holdings.clone() } });
    let saved = domain.roles.insert(project, holdings);
    assert!(saved.is_ok(), "existing role row replaces without consuming capacity");
    prune_unheld_roles(domain, project);
    Ok(())
}

pub(super) fn valid_roles(domain: &Domain, limits: &Limits, project: u32, holdings: &[Holding]) -> Result<(), Outcome> {
    if holdings.len() > usize::try_from(limits.holdings).expect("u32 fits usize") {
        return Err(Outcome::Refused(Refusal::Limit { which: LimitKind::Roles }));
    }
    if !domain.roles.contains_key(&project) && domain.roles.len() >= limits.projects {
        return Err(Outcome::Again(Again::Busy));
    }
    for (at, holding) in holdings.iter().enumerate() {
        match holding.role {
            Role::Policy { role } if role <= 3 => {
                return Err(Outcome::Refused(Refusal::Limit { which: LimitKind::Roles }));
            }
            Role::Owner | Role::Maintainer | Role::Member | Role::Observer | Role::Policy { .. } => {}
        }
        for earlier in holdings.iter().take(at) {
            if earlier.person == holding.person {
                return Err(Outcome::Refused(Refusal::Limit { which: LimitKind::Roles }));
            }
        }
    }
    Ok(())
}

pub(super) fn seed(
    domain: &mut Domain,
    limits: &Limits,
    project: u32,
    collaborators: &[Seed],
    out: &mut Queue<Request>,
) {
    assert!(domain.can_seed(limits, project, collaborators), "root preflights an adoption seed");
    let old = domain.roles.get(&project).expect("preflighted project");
    let mut holdings = List::with_capacity(limits.holdings);
    for holding in &**old {
        holdings.push(*holding).expect("preflighted roster room");
    }
    let mut changed = false;
    for collaborator in collaborators {
        let person = match domain.identities.get(&collaborator.identity) {
            Some(person) => *person,
            None => {
                let person = collaborator.candidate;
                let identity = Identity { key: collaborator.identity.clone(), login: Box::new([]), name: Box::new([]) };
                domain.identities.insert(collaborator.identity.clone(), person).expect("preflighted identity room");
                domain.people.insert(person, identity.clone()).expect("preflighted person room");
                out.push(Request::Save { record: Stored::Person { number: person, identity } });
                person
            }
        };
        if !has_person(holdings.as_slice(), person) {
            holdings.push(Holding { person, role: collaborator.role }).expect("preflighted roster room");
            changed = true;
        }
    }
    if changed {
        let holdings = holdings.into_boxed();
        domain.roles.insert(project, holdings.clone()).expect("preflighted project");
        out.push(Request::Save { record: Stored::Roles { project, holdings } });
    }
}

pub(super) fn has_person(holdings: &[Holding], person: u64) -> bool {
    for holding in holdings {
        if holding.person == person {
            return true;
        }
    }
    false
}

pub(super) fn make_service(
    domain: &mut Domain,
    env: &Env<Limits>,
    request: Token,
    person: u64,
    out: &mut Queue<Request>,
) {
    if !domain.ready() {
        out.push(Request::ServiceMade { request, outcome: Outcome::Again(Again::NotReady) });
        return;
    }
    let flight = domain.party_pending(Id::from_token(request)).expect("service creation names a live keyed flight");
    let (project, name, service_role) = match flight.part.as_ref().unwrap_or(&flight.ask) {
        Ask::MakeService { project, name, role } => (*project, name.clone(), *role),
        Ask::Compound { .. }
        | Ask::MarkSeen { .. }
        | Ask::Application { .. }
        | Ask::EditNote { .. }
        | Ask::Adopt { .. }
        | Ask::Create { .. }
        | Ask::Propose { .. }
        | Ask::Withdraw { .. }
        | Ask::Stop { .. }
        | Ask::Cancel { .. }
        | Ask::Lift { .. }
        | Ask::Release { .. }
        | Ask::TakePerson { .. }
        | Ask::HandBackPerson { .. }
        | Ask::AnswerPerson { .. }
        | Ask::Move { .. }
        | Ask::DecideProposal { .. }
        | Ask::Say { .. }
        | Ask::AnswerQuestion { .. }
        | Ask::Amend { .. }
        | Ask::SetRoles { .. }
        | Ask::ChangePolicy { .. }
        | Ask::RemoveProject { .. }
        | Ask::Rename { .. }
        | Ask::MakeProject { .. }
        | Ask::ChangeSettings { .. }
        | Ask::MoveToNewest { .. }
        | Ask::SetPool { .. }
        | Ask::DecideEscalation { .. }
        | Ask::AccountSignIn { .. }
        | Ask::AccountSignOut { .. }
        | Ask::AccountChange { .. } => unreachable!("service creation is routed only from its ask"),
    };
    let identity = Identity {
        key: IdentityKey { provider: domain.deployment_provider, subject: person.to_be_bytes().into() },
        login: Box::new([]),
        name,
    };
    let holdings = domain.roles.get(&project);
    let outcome = if role(domain, flight.key.person, project) != Some(Role::Owner) {
        Outcome::Refused(Refusal::Role { held: role(domain, flight.key.person, project) })
    } else if person == 0 || domain.people.contains_key(&person) || domain.identities.contains_key(&identity.key) {
        Outcome::Refused(Refusal::Unknown)
    } else if !valid_identity(&env.limits, &identity) {
        Outcome::Refused(Refusal::Limit { which: LimitKind::Identity })
    } else if domain.people.len() >= env.limits.people
        || match holdings {
            Some(rows) => rows.len() >= usize::try_from(env.limits.holdings).expect("u32 fits usize"),
            None => true,
        }
    {
        Outcome::Again(Again::Busy)
    } else {
        let old = holdings.expect("project room checked");
        let mut next = List::with_capacity(env.limits.holdings);
        for holding in &**old {
            next.push(*holding).expect("preflighted role room");
        }
        next.push(Holding { person, role: service_role }).expect("preflighted role room");
        let next = next.into_boxed();
        let indexed = domain.identities.insert(identity.key.clone(), person);
        assert!(indexed == Ok(None), "service identity preflighted");
        let saved = domain.people.insert(person, identity.clone());
        assert!(saved == Ok(None), "service person preflighted");
        let saved = domain.roles.insert(project, next.clone());
        assert!(saved.is_ok(), "service project preflighted");
        out.push(Request::Save { record: Stored::Person { number: person, identity } });
        out.push(Request::Save { record: Stored::Roles { project, holdings: next } });
        Outcome::ServiceMade { person }
    };
    out.push(Request::ServiceMade { request, outcome });
}

/// The first owners whose first sign-ins seed their roles: none once the first start is over.
pub(super) fn seeds(owners: &[InitialOwner], seeding: bool) -> &[InitialOwner] {
    if seeding { owners } else { &[] }
}

#[expect(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    reason = "one sign-in decision checks replay and held ownership before capacity and writes"
)]
pub(super) fn signin(
    domain: &mut Domain,
    env: &Env<Limits>,
    to: ReplyTo,
    candidate: u64,
    number: u64,
    identity: Identity,
    kind: Kind,
    held: Option<u64>,
    out: &mut Queue<Request>,
) {
    if !domain.ready() {
        return again(to, Again::NotReady, out);
    }
    if !valid_identity(&env.limits, &identity) {
        return refused(to, Refusal::Limit { which: LimitKind::Identity }, out);
    }
    match kind {
        Kind::Person if identity.key.provider == domain.deployment_provider => {
            return signed_out(to, out);
        }
        Kind::Service if identity.key.provider != domain.deployment_provider => {
            return signed_out(to, out);
        }
        Kind::Person | Kind::Service => {}
    }
    let known = domain.identities.get(&identity.key).copied();
    if kind == Kind::Service && known.is_none() {
        return signed_out(to, out);
    }
    let person = known.unwrap_or(candidate);
    if let Some(old) = domain.sign_ins.get(&number) {
        if old.person != person {
            return signed_out(to, out);
        }
        if old.expires <= env.wall || old.due <= env.now {
            return signed_out(to, out);
        }
        return out
            .push(Request::Reply { to, reply: Reply::SignedIn { person, sign_in: number, expires: old.expires } });
    }
    if let Some(held) = held
        && let Some(row) = domain.sign_ins.get(&held)
    {
        // A held sign-in is kept only for the same person as the identity.
        #[expect(clippy::collapsible_if, reason = "held ownership is an explicit comparison (people.md, 3)")]
        if row.person == person {
            if row.expires > env.wall && row.due > env.now {
                return out.push(Request::Reply {
                    to,
                    reply: Reply::SignedIn { person, sign_in: held, expires: row.expires },
                });
            }
        }
    }
    if domain.sign_ins.len() >= env.limits.sign_ins {
        if let Some(number) = domain.alarms.expire(env.now) {
            end_signin(domain, number, out);
        } else if let Some(&(expires, number)) = domain.expiries.first()
            && expires <= env.wall
        {
            end_signin(domain, number, out);
        } else {
            let &(earliest, _) = domain.expiries.first().expect("positive full sign-in pool has an expiry");
            assert!(earliest > env.wall, "full sign-ins are live by both clocks");
            return out.push(Request::Reply { to, reply: Reply::Full { sign_ins: env.limits.sign_ins, earliest } });
        }
    }
    if known.is_none() {
        if domain.people.len() >= env.limits.people {
            return again(to, Again::Busy, out);
        }
        assert!(!domain.people.contains_key(&person), "root gives a fresh candidate person number");
        // Reserve all bootstrap role slots before making either record.
        for owner in seeds(&domain.owners, domain.seeding) {
            if owner.identity == identity.key {
                let Some(holdings) = domain.roles.get(&owner.project) else {
                    return refused(to, Refusal::Unknown, out);
                };
                if holdings.len() >= usize::try_from(env.limits.holdings).expect("u32 fits usize")
                    && !has_person(holdings, person)
                {
                    return again(to, Again::Busy, out);
                }
            }
        }
    }
    let Some(expires_nanos) = env.wall.as_nanos().checked_add(env.limits.sign_in_lifetime.as_nanos()) else {
        return refused(to, Refusal::Limit { which: LimitKind::Arithmetic }, out);
    };
    let expires = Wall::from_nanos(expires_nanos);
    if known.is_none() {
        let indexed = domain.identities.insert(identity.key.clone(), person);
        assert!(indexed == Ok(None), "new identity has room");
        // Each project occurs at most once for this identity in configuration.
        for owner in seeds(&domain.owners, domain.seeding) {
            if owner.identity != identity.key {
                continue;
            }
            let old = domain.roles.get(&owner.project).expect("bootstrap project admitted");
            let mut amended = List::with_capacity(env.limits.holdings);
            for holding in &**old {
                if holding.person != person {
                    amended.push(*holding).expect("bounded list room checked");
                }
            }
            amended.push(Holding { person, role: Role::Owner }).expect("bounded list room checked");
            let holdings = amended.into_boxed();
            out.push(Request::Save { record: Stored::Roles { project: owner.project, holdings: holdings.clone() } });
            let saved = domain.roles.insert(owner.project, holdings);
            assert!(saved.is_ok(), "existing bootstrap project");
        }
    }
    let changed = kind == Kind::Person
        && match domain.people.get(&person) {
            Some(old) => old != &identity,
            None => true,
        };
    if changed {
        out.push(Request::Save { record: Stored::Person { number: person, identity: identity.clone() } });
        let saved = domain.people.insert(person, identity);
        assert!(saved.is_ok(), "person admitted before mutation");
    }
    let saved = domain.sign_ins.insert(number, SignIn { person, expires, due: deadline(env, expires) });
    assert!(saved == Ok(None), "sign-in admitted before mutation");
    arm(domain, env, number, expires);
    out.push(Request::Save { record: Stored::SignIn { number, person, expires } });
    out.push(Request::Reply { to, reply: Reply::SignedIn { person, sign_in: number, expires } });
    fact(domain, Fact::SignedIn { person, sign_in: number });
}

pub(super) fn deadline(env: &Env<Limits>, expires: Wall) -> Time {
    let left = skein_lib::Duration::from_nanos(expires.as_nanos().saturating_sub(env.wall.as_nanos()));
    env.now.saturating_add(left)
}

pub(super) fn arm(domain: &mut Domain, env: &Env<Limits>, number: u64, expires: Wall) {
    assert!(domain.expiries.insert((expires, number)).is_ok(), "one saved expiry per sign-in");
    let armed = domain.alarms.arm(number, deadline(env, expires));
    assert!(armed.is_ok(), "one alarm per admitted sign-in");
}

pub(super) fn end_signin(domain: &mut Domain, number: u64, out: &mut Queue<Request>) {
    if let Some(signin) = domain.sign_ins.remove(&number) {
        domain.expiries.remove(&(signin.expires, number));
        domain.alarms.cancel(number);
        out.push(Request::Erase { key: Key::SignIn(number) });
        fact(domain, Fact::SignedOut { sign_in: number });
    }
}

pub(super) fn role(domain: &Domain, person: u64, project: u32) -> Option<Role> {
    for holding in &**domain.roles.get(&project)? {
        if holding.person == person {
            return Some(holding.role);
        }
    }
    None
}
