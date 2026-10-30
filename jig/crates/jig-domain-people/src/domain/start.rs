//! Explicit load/fit/link and a borrowed second expiry walk (jig/docs/design/domain/engine.md, 6).
mod fit;
use super::{Answered, Domain, Phase as State, SignIn, answer_expiry, deadline};
use crate::{
    Key, LimitKind, Limits, Outcome, Refusal, Request, RestoreBound, RestoreFailure, RestoreProjection, Stored,
};
use skein_lib::{Env, Queue, Time, Wall};

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Phase {
    Default,
    Loading,
    Linked,
    Sweeping,
    Complete,
}

impl Domain {
    /// The first selected-start fit failure, retained independently of output consumption.
    #[must_use]
    pub fn restore_failure(&self) -> Option<RestoreFailure> {
        self.restore_failure.clone()
    }

    pub(super) fn assert_projection_event(&self) {
        assert!(
            self.start == Phase::Default || self.ready(),
            "selected people projections require start_project before open"
        );
    }

    /// Rebuild a bounded volatile census projection while loading, without clocks or effects.
    #[must_use]
    pub fn start_project(&mut self, limits: &Limits, input: RestoreProjection) -> bool {
        assert!(self.start == Phase::Loading, "people projections require selected loading");
        if self.phase == State::Failed {
            return false;
        }
        match input {
            RestoreProjection::Waiting { task, entries } => super::inbox::replace_waiting(self, limits, task, &entries),
            RestoreProjection::Result { party, entry } => self.project_result(limits, party, entry),
            RestoreProjection::ChatState { party, project, task, last_reply, live, seen, replies } => {
                super::inbox::project_chat(self, limits, party, project, task, last_reply, live, seen, replies);
            }
            RestoreProjection::EndedConversation { task } => {
                for (key, _) in &self.conversations_seen {
                    if key.1 == task {
                        self.ended_witnesses.insert(*key).expect("witnesses are a subset of loaded read positions");
                    }
                }
            }
        }
        true
    }

    /// Select explicit restoration once before any input, capturing the start's expiry instant.
    pub fn start_restore(&mut self, at: Wall) {
        assert!(
            self.start == Phase::Default && !self.touched && self.phase == State::Restoring,
            "people start selected once before input"
        );
        self.start = Phase::Loading;
        self.start_at = Some(at);
    }

    pub(super) fn start_lookup(&self) -> bool {
        self.start != Phase::Default && self.phase != State::Failed
    }

    /// Seal loading and assert required retained references. Does not open admission or erase.
    pub fn start_link(&mut self, _env: &Env<Limits>) -> bool {
        assert!(self.start == Phase::Loading, "people link follows loading once");
        self.start = Phase::Linked;
        if self.phase == State::Failed {
            return false;
        }
        for (number, row) in &self.sign_ins {
            link(self, Key::SignIn(*number), row.person);
        }
        for (project, holdings) in &self.roles {
            for holding in &**holdings {
                link(self, Key::Roles(*project), holding.person);
            }
        }
        for (key, answer) in &self.answers {
            let source = Key::Answer(*key);
            link(self, source, key.person);
            match &answer.outcome {
                Outcome::ServiceMade { person } => link(self, source, *person),
                Outcome::RolesSet { project } => {
                    assert!(self.roles.contains_key(project), "{source:?} links missing roster {project}");
                    // Only successful roster asks retain their members as required references.
                    link_roster(self, source, &answer.ask);
                }
                Outcome::Again(_)
                | Outcome::SignedOut
                | Outcome::Compound { .. }
                | Outcome::CompoundRefused { .. }
                | Outcome::MarkedSeen { .. }
                | Outcome::Applied { .. }
                | Outcome::NoteEdited { .. }
                | Outcome::Adopted { .. }
                | Outcome::PolicyChanged { .. }
                | Outcome::Renamed { .. }
                | Outcome::ProjectMade { .. }
                | Outcome::SettingsChanged { .. }
                | Outcome::MovedToNewest { .. }
                | Outcome::PoolSet { .. }
                | Outcome::Said { .. }
                | Outcome::QuestionAnswered { .. }
                | Outcome::Amended { .. }
                | Outcome::Moved { .. }
                | Outcome::PersonTaken { .. }
                | Outcome::PersonHandedBack { .. }
                | Outcome::PersonAnswered { .. }
                | Outcome::Stopped { .. }
                | Outcome::Cancelled { .. }
                | Outcome::Lifted { .. }
                | Outcome::Released { .. }
                | Outcome::Created { .. }
                | Outcome::Proposed { .. }
                | Outcome::Withdrawn { .. }
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
                | Outcome::Refused(_) => {}
            }
        }
        for (party, _) in &self.results_seen {
            link(self, Key::ReadResults { party: *party }, *party);
        }
        for ((party, task), _) in &self.conversations_seen {
            link(self, Key::ReadConversation { party: *party, task: *task }, *party);
        }
        for key in &self.ended_witnesses {
            let _old = self.conversations_seen.remove(key);
        }
        self.ended_witnesses = skein_lib::Set::with_capacity(self.conversations_seen.capacity());
        true
    }

    /// Enter the second walk after local linking; the caller owns the global link barrier.
    pub fn start_sweep(&mut self) {
        assert!(self.start == Phase::Linked && self.phase != State::Failed, "people sweep requires successful link");
        self.start = Phase::Sweeping;
    }

    /// Erase only borrowed rows expired at the captured start time, retaining no row or key.
    pub fn start_sweep_row(&mut self, env: &Env<Limits>, record: &Stored, out: &mut Queue<Request>) {
        assert!(self.start == Phase::Sweeping && self.phase != State::Failed, "people sweep row requires sweep");
        if expired(self, &env.limits, record) {
            out.push(Request::Erase { key: record.key() });
        }
    }

    /// Open after sweep EOF and durable release, projecting live deadlines without erasure.
    pub fn start_complete(&mut self, env: &Env<Limits>) -> bool {
        assert!(self.start == Phase::Sweeping, "people completion follows sweep");
        self.start = Phase::Complete;
        if self.phase == State::Failed {
            return false;
        }
        // Snapshot only bounded live keys; no expired row survives its first-walk call.
        let mut sign_ins = skein_lib::List::with_capacity(self.sign_ins.len());
        for (key, _) in &self.sign_ins {
            sign_ins.push(*key).expect("bounded live sign-ins");
        }
        for key in sign_ins.into_boxed() {
            let signin = self.sign_ins.get_mut(&key).expect("loaded sign-in");
            signin.due = deadline(env, signin.expires);
            self.alarms.arm(key, signin.due).expect("one alarm per fitted sign-in");
        }
        let mut answers = skein_lib::List::with_capacity(self.answers.len());
        for (key, _) in &self.answers {
            answers.push(*key).expect("bounded live answers");
        }
        for key in answers.into_boxed() {
            let answer = self.answers.get_mut(&key).expect("loaded answer");
            answer.due = deadline(env, answer_expiry(answer.at, &env.limits));
            self.answer_alarms.arm(key, answer.due).expect("one alarm per fitted answer");
        }
        let mut acts = skein_lib::List::with_capacity(self.acted.len());
        for (key, _) in &self.acted {
            acts.push(*key).expect("bounded live acts");
        }
        for key in acts.into_boxed() {
            let due = self.acted.get_mut(&key).expect("loaded act").project_deadline(env);
            self.act_alarms.arm(key, due).expect("one alarm per fitted act");
        }
        self.phase = State::Ready;
        true
    }
}

fn link(domain: &Domain, source: Key, person: u64) {
    assert!(domain.people.contains_key(&person), "{source:?} links missing person {person}");
}

fn expired(domain: &Domain, limits: &Limits, record: &Stored) -> bool {
    let at = domain.start_at.expect("explicit start captured its wall");
    match record {
        Stored::SignIn { expires, .. } => *expires <= at,
        Stored::Answer { at: saved, .. } | Stored::Acted { at: saved, .. } => answer_expiry(*saved, limits) <= at,
        Stored::Person { .. } | Stored::Roles { .. } | Stored::ReadResults { .. } | Stored::ReadConversation { .. } => {
            false
        }
        Stored::Policy { .. } | Stored::EndedRead { .. } => unreachable!("wrong people family: {:?}", record.key()),
    }
}

pub(super) fn load(domain: &mut Domain, env: &Env<Limits>, record: Stored, out: &mut Queue<Request>) {
    assert!(domain.start == Phase::Loading, "people restore after seal: {:?}", record.key());
    if domain.phase == State::Failed {
        return;
    }
    // Expired decoded rows leave with their page, before any live fit or judgment.
    if expired(domain, &env.limits, &record) {
        return;
    }
    let key = record.key();
    let fits = fits(domain, env, &record);
    if let Err(cause) = fits {
        if domain.restore_failure.is_none() {
            domain.restore_failure = Some(RestoreFailure { key, cause });
        }
        return super::restore_failed(domain, key, Refusal::Limit { which: LimitKind::Restore }, out);
    }
    match record {
        Stored::Person { number, identity } => {
            domain.identities.insert(identity.key.clone(), number).expect("identity fit checked");
            domain.people.insert(number, identity).expect("person fit checked");
        }
        Stored::SignIn { number, person, expires } => {
            domain.sign_ins.insert(number, SignIn { person, expires, due: Time::ZERO }).expect("sign-in fit checked");
            domain.expiries.insert((expires, number)).expect("expiry fit checked");
        }
        Stored::Roles { project, holdings } => {
            domain.roles.insert(project, holdings).expect("roster fit checked");
        }
        Stored::ReadResults { party, through } => {
            domain.results_seen.insert(party, through).expect("read fit checked");
        }
        Stored::ReadConversation { party, task, through } => {
            domain.conversations_seen.insert((party, task), through).expect("read fit checked");
        }
        Stored::Answer { key, ask, models, outcome, at } => {
            domain
                .answers
                .insert(key, Answered { ask: *ask, models, outcome, at, due: Time::ZERO })
                .expect("answer fit checked");
        }
        Stored::Acted { key, act, outcome, at } => super::acts::load(domain, key, act, outcome, at),
        Stored::Policy { .. } | Stored::EndedRead { .. } => unreachable!("wrong people family: {key:?}"),
    }
}

fn link_roster(domain: &Domain, source: Key, ask: &crate::Ask) {
    match ask {
        crate::Ask::SetRoles { holdings, .. } => {
            for holding in holdings {
                link(domain, source, holding.person);
            }
        }
        crate::Ask::Compound { .. }
        | crate::Ask::MarkSeen { .. }
        | crate::Ask::Application { .. }
        | crate::Ask::EditNote { .. }
        | crate::Ask::MakeService { .. }
        | crate::Ask::Adopt { .. }
        | crate::Ask::ChangePolicy { .. }
        | crate::Ask::Rename { .. }
        | crate::Ask::MakeProject { .. }
        | crate::Ask::ChangeSettings { .. }
        | crate::Ask::MoveToNewest { .. }
        | crate::Ask::SetPool { .. }
        | crate::Ask::DecideEscalation { .. }
        | crate::Ask::DecideProposal { .. }
        | crate::Ask::Say { .. }
        | crate::Ask::AnswerQuestion { .. }
        | crate::Ask::Amend { .. }
        | crate::Ask::Move { .. }
        | crate::Ask::TakePerson { .. }
        | crate::Ask::HandBackPerson { .. }
        | crate::Ask::AnswerPerson { .. }
        | crate::Ask::Stop { .. }
        | crate::Ask::Cancel { .. }
        | crate::Ask::Lift { .. }
        | crate::Ask::Release { .. }
        | crate::Ask::Create { .. }
        | crate::Ask::Propose { .. }
        | crate::Ask::Withdraw { .. }
        | crate::Ask::AccountSignIn { .. }
        | crate::Ask::AccountSignOut { .. }
        | crate::Ask::AccountChange { .. } => {}
    }
}

fn fits(domain: &Domain, env: &Env<Limits>, record: &Stored) -> Result<(), crate::RestoreCause> {
    let key = record.key();
    match record {
        Stored::Person { number, identity } => {
            assert!(!domain.people.contains_key(number), "duplicate {key:?}");
            assert!(!domain.identities.contains_key(&identity.key), "identity index collision at {key:?}");
            fit::identity(&env.limits, identity)?;
            fit::occupancy(RestoreBound::People, domain.people.len(), 0, env.limits.people)
        }
        Stored::SignIn { number, .. } => {
            assert!(!domain.sign_ins.contains_key(number), "duplicate {key:?}");
            fit::occupancy(RestoreBound::SignIns, domain.sign_ins.len(), 0, env.limits.sign_ins)
        }
        Stored::Roles { project, holdings } => {
            assert!(!domain.roles.contains_key(project), "duplicate {key:?}");
            fit::within(RestoreBound::Holdings, holdings.len(), env.limits.holdings)?;
            fit::occupancy(RestoreBound::Roles, domain.roles.len(), 0, env.limits.projects)
        }
        Stored::ReadResults { party, .. } => {
            assert!(!domain.results_seen.contains_key(party), "duplicate {key:?}");
            fit::occupancy(RestoreBound::ReadResults, domain.results_seen.len(), 0, domain.results_seen.capacity())
        }
        Stored::ReadConversation { party, task, .. } => {
            assert!(!domain.conversations_seen.contains_key(&(*party, *task)), "duplicate {key:?}");
            fit::occupancy(
                RestoreBound::ReadConversations,
                domain.conversations_seen.len(),
                0,
                domain.conversations_seen.capacity(),
            )
        }
        Stored::Answer { key: request, ask, models, outcome, .. } => {
            assert!(!domain.answers.contains_key(request), "duplicate {key:?}");
            fit::ask(&env.limits, ask)?;
            fit::models(models)?;
            fit::outcome(&env.limits, outcome)?;
            fit::occupancy(RestoreBound::Requests, domain.answers.len(), domain.acted.len(), env.limits.requests)
        }
        Stored::Acted { key: request, act, outcome, .. } => {
            assert!(!domain.acted.contains_key(request), "duplicate {key:?}");
            fit::act(&env.limits, act)?;
            fit::outcome(&env.limits, outcome)?;
            fit::occupancy(RestoreBound::Requests, domain.answers.len(), domain.acted.len(), env.limits.requests)
        }
        Stored::Policy { .. } | Stored::EndedRead { .. } => unreachable!("wrong people family: {key:?}"),
    }
}
