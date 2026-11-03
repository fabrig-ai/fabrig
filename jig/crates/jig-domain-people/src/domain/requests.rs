//! Keyed requests, shared answer reservations and terminal replies (domain/people.md, 5.1).
use super::{Domain, acts, admission, deadline, fact, invalid_ask, mark_seen, project, role, valid_ask};
use crate::{Again, Ask, Fact, Key, Limits, Outcome, Refusal, Reply, Request, RequestKey, Stored};
use alloc::boxed::Box;
use skein_lib::{Env, Id, List, Queue, ReplyTo, Time, Wall};

#[derive(PartialEq, Eq, Hash, Debug)]
pub(crate) struct Answered {
    pub(super) ask: Ask,
    pub(super) models: Box<[Box<[u8]>]>,
    pub(super) outcome: Outcome,
    pub(super) at: Wall,
    pub(super) due: Time,
}

#[derive(Debug)]
pub(crate) struct PartyPending {
    pub(super) part: Option<Ask>,
    pub(super) key: RequestKey,
    pub(super) ask: Ask,
    pub(super) models: Box<[Box<[u8]>]>,
    pub(super) replies: List<ReplyTo>,
}

#[derive(Debug)]
#[expect(clippy::large_enum_variant, reason = "one shared slab prices the largest admitted flight inline")]
pub(crate) enum Pending {
    Party(PartyPending),
    Act(acts::PendingAct),
    /// The answered move keeps its slab slot and key index for Written's answer.
    MoveOut {
        holder: crate::AccountHolder,
        account: u32,
        number: u32,
        source: u32,
    },
}
impl Pending {
    pub(super) fn party(&self) -> Option<&PartyPending> {
        match self {
            Self::Party(party) => Some(party),
            Self::Act(_) | Self::MoveOut { .. } => None,
        }
    }
    pub(super) fn party_mut(&mut self) -> Option<&mut PartyPending> {
        match self {
            Self::Party(party) => Some(party),
            Self::Act(_) | Self::MoveOut { .. } => None,
        }
    }
}

impl Domain {
    pub(super) fn party_pending(&self, id: Id<Pending>) -> Option<&PartyPending> {
        self.pending.get(id)?.party()
    }
    pub(super) fn party_pending_mut(&mut self, id: Id<Pending>) -> Option<&mut PartyPending> {
        self.pending.get_mut(id)?.party_mut()
    }

    /// Release a cancelled move's volatile Written place (protocol/llm.md, 5.6).
    /// `None` cancels all moves when the accounts component closes.
    pub fn cancel_moves(&mut self, source: Option<u32>) {
        self.input();
        for _ in 0..self.flights.capacity() {
            let mut found = None;
            for (key, id) in &self.flights {
                if let Pending::MoveOut { source: held, .. } = self.pending.get(*id).expect("indexed pending")
                    && (source.is_none() || source == Some(*held))
                {
                    found = Some((*key, *id));
                    break;
                }
            }
            let Some((key, id)) = found else { break };
            assert!(self.flights.remove(&key) == Some(id), "one cancelled move index");
            self.pending.retire(id);
        }
    }
}

pub(super) fn answer_expiry(at: Wall, limits: &Limits) -> Wall {
    Wall::from_nanos(at.as_nanos().saturating_add(limits.request_retention.as_nanos()))
}

pub(super) fn again(to: ReplyTo, why: Again, out: &mut Queue<Request>) {
    out.push(Request::Reply { to, reply: Reply::Again(why) });
}

pub(super) fn signed_out(to: ReplyTo, out: &mut Queue<Request>) {
    out.push(Request::Reply { to, reply: Reply::SignedOut });
}

pub(super) fn outcome_reply(outcome: Outcome) -> Reply {
    match outcome {
        Outcome::Again(why) => Reply::Again(why),
        Outcome::SignedOut => Reply::SignedOut,
        Outcome::Restored { .. }
        | Outcome::Compound { .. }
        | Outcome::CompoundRefused { .. }
        | Outcome::MarkedSeen { .. }
        | Outcome::Applied { .. }
        | Outcome::NoteEdited { .. }
        | Outcome::ServiceMade { .. }
        | Outcome::Adopted { .. }
        | Outcome::Created { .. }
        | Outcome::Proposed { .. }
        | Outcome::Withdrawn { .. }
        | Outcome::Stopped { .. }
        | Outcome::Cancelled { .. }
        | Outcome::Lifted { .. }
        | Outcome::Released { .. }
        | Outcome::PersonTaken { .. }
        | Outcome::PersonHandedBack { .. }
        | Outcome::PersonAnswered { .. }
        | Outcome::Moved { .. }
        | Outcome::ProposalDecided { .. }
        | Outcome::Said { .. }
        | Outcome::QuestionAnswered { .. }
        | Outcome::Amended { .. }
        | Outcome::RolesSet { .. }
        | Outcome::PolicyChanged { .. }
        | Outcome::Renamed { .. }
        | Outcome::ProjectMade { .. }
        | Outcome::ProjectRemoved { .. }
        | Outcome::VisibilitySet { .. }
        | Outcome::SettingsChanged { .. }
        | Outcome::MovedToNewest { .. }
        | Outcome::PoolSet { .. }
        | Outcome::EscalationDecided { .. }
        | Outcome::AccountSignInStarted { .. }
        | Outcome::AccountProgramDeclared { .. }
        | Outcome::AccountKeyKept { .. }
        | Outcome::AccountMovingOut { .. }
        | Outcome::AccountFinishing { .. }
        | Outcome::AccountMovedIn { .. }
        | Outcome::AccountChanged
        | Outcome::AccountRemoved { .. }
        | Outcome::AccountSignedOut { .. }
        | Outcome::Refused(_) => Reply::Outcome(outcome),
    }
}

pub(super) fn refused(to: ReplyTo, refusal: Refusal, out: &mut Queue<Request>) {
    out.push(Request::Reply { to, reply: Reply::Refused(refusal) });
}

#[expect(clippy::too_many_arguments, reason = "one keyed request includes its canonical original model context")]
pub(super) fn admit_ask(
    domain: &mut Domain,
    env: &Env<Limits>,
    to: ReplyTo,
    sign_in: u64,
    bytes: [u8; 16],
    ask: Ask,
    models: Box<[Box<[u8]>]>,
    out: &mut Queue<Request>,
) {
    if !domain.ready() {
        return again(to, Again::NotReady, out);
    }
    let Some(session) = domain.sign_ins.get(&sign_in) else {
        return signed_out(to, out);
    };
    if session.expires <= env.wall || session.due <= env.now {
        return signed_out(to, out);
    }
    let key = RequestKey { person: session.person, key: bytes };
    if !valid_ask(&env.limits, &ask) || !crate::valid_model_context(&ask, &models) {
        return refused(to, invalid_ask(&env.limits, &ask), out);
    }
    if let Some(answer) = domain.answers.get(&key)
        && (answer_expiry(answer.at, &env.limits) <= env.wall || answer.due <= env.now)
    {
        let old = domain.answers.remove(&key);
        assert!(old.is_some(), "expired answer was retained");
        domain.answer_alarms.cancel(key);
        out.push(Request::Erase { key: Key::Answer(key) });
    }
    if let Some(answer) = domain.answers.get(&key) {
        if answer.ask != ask || answer.models != models {
            return refused(to, Refusal::KeyConflict, out);
        }
        return out.push(Request::Reply { to, reply: Reply::Outcome(answer.outcome.clone()) });
    }

    if let Some(id) = domain.flights.get(&key).copied() {
        if let Pending::MoveOut { holder, account, .. } = domain.pending.get(id).expect("indexed move") {
            let same = ask
                == Ask::AccountChange { holder: *holder, change: crate::AccountChange::MoveOut { account: *account } };
            return if same { again(to, Again::Busy, out) } else { refused(to, Refusal::KeyConflict, out) };
        }
        let flight = domain.party_pending_mut(id).expect("key index names live pending request");
        if flight.ask != ask || flight.models != models {
            return refused(to, Refusal::KeyConflict, out);
        }
        if flight.replies.len() >= env.limits.waiters {
            room_pressure(domain, env, key, &ask, crate::CapacityCause::Waiters { limit: env.limits.waiters });
            return again(to, Again::Busy, out);
        }
        flight.replies.push(to).expect("bounded list room checked");
        return;
    }
    if let Some(project) = super::deletion::ask_project(&ask)
        && project != crate::DEPLOYMENT
        && !domain.has_project(project)
    {
        return refused(to, Refusal::Unknown, out);
    }
    let written = written_place(domain, key.person, &ask);
    // A move is admitted whole; Written transfers its already reserved place.
    // Seen still uses only pending room (protocol/llm.md, 5.6; people.md, 5.1).
    let occupied = domain.answers.len().saturating_add(domain.acted.len()).saturating_add(answer_reservations(domain));
    let credit = u32::from(written.is_some());
    if occupied.saturating_sub(credit).saturating_add(answer_places(&ask)) > env.limits.requests {
        room_pressure(domain, env, key, &ask, crate::CapacityCause::Requests { limit: env.limits.requests });
        return again(to, full_answers(domain, env, occupied), out);
    }
    let project = project(&ask);
    // An account request's check is its holder, never a role (domain/authority.md, 8.4).
    let role = if let Ask::Application { scope: crate::ApplicationScope::Own, .. }
    | Ask::Lift { .. }
    | Ask::AccountSignIn { .. }
    | Ask::AccountSignOut { .. }
    | Ask::AccountChange { .. } = &ask
    {
        None
    } else {
        role(domain, key.person, project)
    };
    let refusal = admission(domain, key.person, project, role, &ask);
    if let Some(refusal) = refusal {
        if let Some((previous, id)) = written {
            assert!(domain.flights.remove(&previous) == Some(id), "Written consumes its reserved place");
            domain.pending.retire(id);
        }
        let outcome = Outcome::Refused(refusal);
        save_answer(domain, env, key, ask, models, outcome.clone(), out);
        return out.push(Request::Reply { to, reply: outcome_reply(outcome) });
    }
    if written.is_none() && domain.pending.is_full() {
        room_pressure(domain, env, key, &ask, crate::CapacityCause::Pending { limit: env.limits.pending });
        return again(to, Again::Busy, out);
    }
    let mut replies = List::with_capacity(env.limits.waiters);
    replies.push(to).expect("bounded list room checked");
    let flight = Pending::Party(PartyPending { key, ask: ask.clone(), models: models.clone(), replies, part: None });
    let id = match written {
        Some((previous, id)) => {
            assert!(domain.flights.remove(&previous) == Some(id), "Written takes its move's index");
            *domain.pending.get_mut(id).expect("reserved Written slot") = flight;
            id
        }
        None => domain.pending.insert(flight).expect("pending request admitted"),
    };
    let indexed = domain.flights.insert(key, id);
    assert!(indexed == Ok(None), "one key per flight within pending capacity");
    out.push(Request::Route { request: id.token(), person: key.person, project, role, ask: Box::new(ask), models });
    fact(domain, Fact::Routed { person: key.person });
}

pub(super) fn full_answers(domain: &Domain, env: &Env<Limits>, held: u32) -> Again {
    let mut oldest = None;
    for (_, answer) in &domain.answers {
        oldest = Some(oldest.unwrap_or(answer.at).min(answer.at));
    }
    for (_, answer) in &domain.acted {
        oldest = Some(oldest.unwrap_or(answer.at()).min(answer.at()));
    }
    // Reservations have no expiry yet. With no retained answer, report
    // now: keep the ordinary backoff and the caller's patience, rather
    // than inventing an expiry for a decision still in flight.
    let room_at = match oldest {
        Some(at) => answer_expiry(at, &env.limits),
        None => env.wall,
    };
    Again::Full { held, room_at }
}

pub(super) fn room_pressure(
    domain: &mut Domain,
    env: &Env<Limits>,
    key: RequestKey,
    ask: &Ask,
    cause: crate::CapacityCause,
) {
    let action = crate::capacity::action(ask, key.person);
    if domain.room_refusals.capacity() == 0 {
        domain.room_refusals_lost = domain.room_refusals_lost.saturating_add(1);
        return;
    }
    if domain.room_refusals.room() == 0 {
        domain.room_refusals.pop();
        domain.room_refusals_lost = domain.room_refusals_lost.saturating_add(1);
    }
    domain.room_refusals.push(crate::CapacityRefusal {
        key,
        project: crate::capacity::project(ask),
        action,
        at: env.wall,
        cause,
    });
}

pub(super) fn save_answer(
    domain: &mut Domain,
    env: &Env<Limits>,
    key: RequestKey,
    ask: Ask,
    models: Box<[Box<[u8]>]>,
    outcome: Outcome,
    out: &mut Queue<Request>,
) {
    if !reserves_answer(&ask) {
        return;
    }
    out.push(Request::Save {
        record: Stored::Answer {
            key,
            ask: Box::new(ask.clone()),
            models: models.clone(),
            outcome: outcome.clone(),
            at: env.wall,
        },
    });
    let due = deadline(env, answer_expiry(env.wall, &env.limits));
    let saved = domain.answers.insert(key, Answered { ask, models, outcome, at: env.wall, due });
    assert!(saved.is_ok(), "answer room reserved at request entrance");
    let armed = domain.answer_alarms.arm(key, due);
    assert!(armed.is_ok(), "one expiry per retained completed key");
    fact(domain, Fact::Answered { person: key.person });
}

pub(super) fn answer_reservations(domain: &Domain) -> u32 {
    let mut reserved = domain.act_flights.len();
    for (_, id) in &domain.flights {
        match domain.pending.get(*id).expect("key index names live pending request") {
            Pending::Party(flight) => reserved = reserved.saturating_add(answer_places(&flight.ask)),
            Pending::MoveOut { .. } => reserved = reserved.saturating_add(1),
            Pending::Act(_) => unreachable!("acts have their own index"),
        }
    }
    reserved
}

fn answer_places(ask: &Ask) -> u32 {
    if let Ask::AccountChange { change: crate::AccountChange::MoveOut { .. }, .. } = ask {
        2
    } else {
        u32::from(reserves_answer(ask))
    }
}

fn written_place(domain: &Domain, person: u64, ask: &Ask) -> Option<(RequestKey, Id<Pending>)> {
    let Ask::AccountChange { holder, change: crate::AccountChange::Written { number } } = ask else { return None };
    for (key, id) in &domain.flights {
        if let Pending::MoveOut { holder: held, number: moving, .. } = domain.pending.get(*id).expect("indexed move")
            && key.person == person
            && holder == held
            && number == moving
        {
            return Some((*key, *id));
        }
    }
    None
}

pub(super) fn reserves_answer(ask: &Ask) -> bool {
    match ask {
        Ask::MarkSeen { .. } => false,
        Ask::Restore { .. }
        | Ask::Application { .. }
        | Ask::AccountSignIn { .. }
        | Ask::AccountSignOut { .. }
        | Ask::AccountChange { .. }
        | Ask::Compound { .. }
        | Ask::EditNote { .. }
        | Ask::MakeService { .. }
        | Ask::Adopt { .. }
        | Ask::SetRoles { .. }
        | Ask::ChangePolicy { .. }
        | Ask::RemoveProject { .. }
        | Ask::SetVisibility { .. }
        | Ask::Rename { .. }
        | Ask::MakeProject { .. }
        | Ask::ChangeSettings { .. }
        | Ask::MoveToNewest { .. }
        | Ask::SetPool { .. }
        | Ask::Create { .. }
        | Ask::Propose { .. }
        | Ask::Withdraw { .. }
        | Ask::Say { .. }
        | Ask::AnswerQuestion { .. }
        | Ask::Amend { .. }
        | Ask::DecideEscalation { .. }
        | Ask::DecideProposal { .. }
        | Ask::Move { .. }
        | Ask::TakePerson { .. }
        | Ask::HandBackPerson { .. }
        | Ask::AnswerPerson { .. }
        | Ask::Stop { .. }
        | Ask::Cancel { .. }
        | Ask::Lift { .. }
        | Ask::Release { .. } => true,
    }
}

pub(super) fn decided(
    domain: &mut Domain,
    env: &Env<Limits>,
    id: Id<Pending>,
    outcome: Outcome,
    out: &mut Queue<Request>,
) {
    if let Pending::Act(_) = domain.pending.get(id).expect("pending decision") {
        return acts::decided(domain, env, id, outcome, out);
    }
    let party = domain.party_pending(id).expect("pending decision").key.person;
    if let Outcome::MarkedSeen { seen } = outcome {
        mark_seen(domain, &env.limits, party, seen, out);
    }
    answer_decided(domain, env, id, outcome, out);
}

pub(super) fn answer_decided(
    domain: &mut Domain,
    env: &Env<Limits>,
    id: Id<Pending>,
    outcome: Outcome,
    out: &mut Queue<Request>,
) {
    let flight = domain.party_pending_mut(id).expect("root returns each routed request exactly once");
    let key = flight.key;
    let ask = flight.ask.clone();
    let models = flight.models.clone();
    let replies = core::mem::replace(&mut flight.replies, List::with_capacity(0));
    if let (
        Ask::AccountChange { holder, change: crate::AccountChange::MoveOut { .. } },
        Outcome::AccountMovingOut { account, number, source },
    ) = (&ask, &outcome)
    {
        *domain.pending.get_mut(id).expect("admitted move") =
            Pending::MoveOut { holder: *holder, account: *account, number: *number, source: *source };
    } else {
        domain.pending.retire(id);
        let removed = domain.flights.remove(&key);
        assert!(removed == Some(id), "flight has one key index");
    }

    match outcome {
        // A refused admission is retryable with the same key, including
        // pressure reported by tasks or another child through the root.
        Outcome::Again(_) | Outcome::SignedOut => {}
        Outcome::Restored { .. }
        | Outcome::Compound { .. }
        | Outcome::CompoundRefused { .. }
        | Outcome::MarkedSeen { .. }
        | Outcome::Applied { .. }
        | Outcome::NoteEdited { .. }
        | Outcome::ServiceMade { .. }
        | Outcome::Adopted { .. }
        | Outcome::RolesSet { .. }
        | Outcome::PolicyChanged { .. }
        | Outcome::Renamed { .. }
        | Outcome::ProjectMade { .. }
        | Outcome::ProjectRemoved { .. }
        | Outcome::VisibilitySet { .. }
        | Outcome::SettingsChanged { .. }
        | Outcome::MovedToNewest { .. }
        | Outcome::PoolSet { .. }
        | Outcome::Said { .. }
        | Outcome::QuestionAnswered { .. }
        | Outcome::Amended { .. }
        | Outcome::Moved { .. }
        | Outcome::Stopped { .. }
        | Outcome::Cancelled { .. }
        | Outcome::Lifted { .. }
        | Outcome::Released { .. }
        | Outcome::Created { .. }
        | Outcome::Proposed { .. }
        | Outcome::Withdrawn { .. }
        | Outcome::PersonTaken { .. }
        | Outcome::PersonHandedBack { .. }
        | Outcome::PersonAnswered { .. }
        | Outcome::EscalationDecided { .. }
        | Outcome::ProposalDecided { .. }
        | Outcome::AccountSignInStarted { .. }
        | Outcome::AccountProgramDeclared { .. }
        | Outcome::AccountKeyKept { .. }
        | Outcome::AccountMovingOut { .. }
        | Outcome::AccountFinishing { .. }
        | Outcome::AccountMovedIn { .. }
        | Outcome::AccountChanged
        | Outcome::AccountRemoved { .. }
        | Outcome::AccountSignedOut { .. }
        | Outcome::Refused(_) => save_answer(domain, env, key, ask, models, outcome.clone(), out),
    }
    for to in replies.into_boxed() {
        out.push(Request::Reply { to, reply: outcome_reply(outcome.clone()) });
    }
}
