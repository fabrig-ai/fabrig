//! The start: load, then link, and a borrowed second sweep (jig/docs/design/domain/engine/restart.md).
mod fit;
use super::{Answered, Domain, SignIn, Start, answer_expiry, deadline};
use crate::{
    Key, LimitKind, Limits, OutcomeFamily, Refusal, Request, RestoreBound, RestoreFailure, RestoreProjection, Stored,
};
use skein_lib::{Env, Queue, Time, Wall};

impl Domain {
    /// The start's first fit failure, retained independently of output consumption.
    #[must_use]
    pub fn restore_failure(&self) -> Option<RestoreFailure> {
        self.restore_failure.clone()
    }

    /// An event's projection lands on a whole census: from the link on
    /// (jig/docs/design/domain/engine/settings.md, 7), never while loading.
    pub(super) fn assert_projection_event(&self) {
        assert!(
            self.ready() || self.start == Start::Linked,
            "people projections require start_project before the link"
        );
    }

    /// Rebuild a bounded volatile census projection while loading, without clocks or effects.
    #[must_use]
    pub fn start_project(&mut self, limits: &Limits, input: RestoreProjection) -> bool {
        assert!(self.start == Start::Loading || self.start == Start::Failed, "people projections require loading");
        if self.start == Start::Failed {
            return false;
        }
        match input {
            RestoreProjection::Waiting { task, entries } => {
                let mut out = Queue::uncapped();
                super::inbox::replace_waiting(self, limits, task, &entries, &mut out);
                assert!(out.is_empty(), "the census rebuild emits no arrivals");
            }
            RestoreProjection::Result { party, entry } => self.project_result(limits, party, entry),
            RestoreProjection::ChatState { party, project, task, last_reply, live, seen, replies } => {
                super::inbox::project_chat(
                    self,
                    limits,
                    super::inbox::State { party, project, task, last_reply, live, seen },
                    replies,
                );
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

    /// Begin restoration once before any input, capturing the start's expiry instant.
    pub fn start_restore(&mut self, at: Wall) {
        assert!(self.start == Start::NotStarted, "people start begins once before input");
        self.start = Start::Loading;
        self.start_at = Some(at);
    }

    pub(super) fn start_lookup(&self) -> bool {
        match self.start {
            Start::Loading | Start::Linked | Start::Sweeping => true,
            Start::NotStarted | Start::Open | Start::Failed => false,
        }
    }

    /// Seal loading and link required retained references. Does not open admission or erase.
    pub fn start_link(&mut self) -> bool {
        assert!(self.start == Start::Loading || self.start == Start::Failed, "people link follows loading once");
        if self.start == Start::Failed {
            return false;
        }
        let mut missing = None;
        self.start = Start::Linked;
        for (number, row) in &self.sign_ins {
            link(self, &mut missing, Key::SignIn(*number), row.person);
        }
        for (project, holdings) in &self.roles {
            for holding in &**holdings {
                link(self, &mut missing, Key::Roles(*project), holding.person);
            }
        }
        for (key, answer) in &self.answers {
            let source = Key::Answer(*key);
            link(self, &mut missing, source, key.person);
            match answer.outcome.family() {
                OutcomeFamily::ServiceMade(person) => link(self, &mut missing, source, person),
                OutcomeFamily::RolesSet(project) => {
                    // Until its own rows go, an answer links to the kept deletion
                    // (jig/docs/design/domain/people/acts.md, 2).
                    if !self.roles.contains_key(&project) && !self.deletions.contains_key(&project) && missing.is_none()
                    {
                        missing = Some(RestoreFailure {
                            key: source,
                            cause: crate::RestoreCause::Link { target: Key::Roles(project) },
                        });
                    }
                    // Only successful roster asks retain their members as required references.
                    link_roster(self, &mut missing, source, &answer.ask);
                }
                OutcomeFamily::Again(_)
                | OutcomeFamily::SignedOut
                | OutcomeFamily::Compound
                | OutcomeFamily::Refused
                | OutcomeFamily::CompoundRefused
                | OutcomeFamily::ProjectMade(_)
                | OutcomeFamily::Settled => {}
            }
        }
        for (party, _) in &self.results_seen {
            link(self, &mut missing, Key::ReadResults { party: *party }, *party);
        }
        for ((party, task), _) in &self.conversations_seen {
            link(self, &mut missing, Key::ReadConversation { party: *party, task: *task }, *party);
        }
        if let Some(failure) = missing {
            self.restore_failure = Some(failure);
            self.start = Start::Failed;
            return false;
        }
        for key in &self.ended_witnesses {
            let _old = self.conversations_seen.remove(key);
        }
        self.ended_witnesses = skein_lib::Set::with_capacity(self.conversations_seen.capacity());
        true
    }

    /// Enter the second walk after local linking; the caller owns the global link barrier.
    pub fn start_sweep(&mut self) {
        assert!(self.start == Start::Linked, "people sweep requires successful link");
        self.start = Start::Sweeping;
    }

    /// Erase borrowed rows not retained at start, retaining no row or key.
    pub fn start_sweep_row(&mut self, env: &Env<Limits>, record: &Stored, out: &mut Queue<Request>) {
        assert!(self.start == Start::Sweeping, "people sweep row requires sweep");
        if discarded(self, &env.limits, record) {
            out.push(Request::Erase { key: record.key() });
        }
    }

    /// Open after sweep EOF and durable release, projecting live deadlines without erasure.
    pub fn start_complete(&mut self, env: &Env<Limits>) -> bool {
        assert!(self.start == Start::Sweeping, "people completion follows sweep");
        // Snapshot only bounded live keys; no discarded row survives its first-walk call.
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
        self.start = Start::Open;
        true
    }
}

fn link(domain: &Domain, missing: &mut Option<RestoreFailure>, source: Key, person: u64) {
    if !domain.people.contains_key(&person) && missing.is_none() {
        *missing =
            Some(RestoreFailure { key: source, cause: crate::RestoreCause::Link { target: Key::Person(person) } });
    }
}

fn discarded(domain: &Domain, limits: &Limits, record: &Stored) -> bool {
    let at = domain.start_at.expect("start captured its wall");
    match record {
        Stored::SignIn { expires, .. } => *expires <= at,
        Stored::Answer { ask, at: saved, .. } => {
            !super::requests::reserves_answer(ask) || answer_expiry(*saved, limits) <= at
        }
        Stored::Acted { at: saved, .. } => answer_expiry(*saved, limits) <= at,
        Stored::Deletion { .. }
        | Stored::Public { .. }
        | Stored::Deleted { .. }
        | Stored::Person { .. }
        | Stored::Roles { .. }
        | Stored::ReadResults { .. }
        | Stored::ReadConversation { .. } => false,
        Stored::Policy { .. } | Stored::EndedRead { .. } => unreachable!("wrong people family: {:?}", record.key()),
    }
}

pub(super) fn load(domain: &mut Domain, env: &Env<Limits>, record: Stored, out: &mut Queue<Request>) {
    assert!(
        domain.start == Start::Loading || domain.start == Start::Failed,
        "people restore after seal: {:?}",
        record.key()
    );
    if domain.start == Start::Failed {
        return;
    }
    // Discarded decoded rows leave with their page, before any live fit or judgment.
    if discarded(domain, &env.limits, &record) {
        return;
    }
    let key = record.key();
    let fits = fits(domain, env, &record);
    if let Err(cause) = fits {
        if domain.restore_failure.is_none() {
            domain.restore_failure = Some(RestoreFailure { key, cause });
        }
        return restore_failed(domain, key, Refusal::Limit { which: LimitKind::Restore }, out);
    }
    match record {
        Stored::Public { .. } => {}
        Stored::Deleted { project } => {
            domain.deleted.insert(project).expect("mark fit checked");
        }
        Stored::Deletion { project, step, grace } => {
            domain.deletions.insert(project, crate::KeptDeletion { step, grace }).expect("deletion fit checked");
        }
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

fn link_roster(domain: &Domain, missing: &mut Option<RestoreFailure>, source: Key, ask: &crate::Ask) {
    if let Some((_, holdings)) = ask.roster() {
        for holding in holdings {
            link(domain, missing, source, holding.person);
        }
    }
}

fn fits(domain: &Domain, env: &Env<Limits>, record: &Stored) -> Result<(), crate::RestoreCause> {
    let key = record.key();
    match record {
        Stored::Public { .. } => Ok(()),
        Stored::Deleted { project } => {
            assert!(!domain.deleted.contains(project), "duplicate {key:?}");
            fit::occupancy(&key, RestoreBound::Roles, domain.deleted.len(), 0, env.limits.projects)
        }
        Stored::Deletion { project, .. } => {
            assert!(!domain.deletions.contains_key(project), "duplicate {key:?}");
            fit::occupancy(&key, RestoreBound::Roles, domain.deletions.len(), 0, env.limits.projects)
        }
        Stored::Person { number, identity } => {
            assert!(!domain.people.contains_key(number), "duplicate {key:?}");
            if let Some(person) = domain.identities.get(&identity.key) {
                return Err(crate::RestoreCause::Link { target: Key::Person(*person) });
            }
            fit::identity(&key, &env.limits, identity)?;
            fit::occupancy(&key, RestoreBound::People, domain.people.len(), 0, env.limits.people)
        }
        Stored::SignIn { number, .. } => {
            assert!(!domain.sign_ins.contains_key(number), "duplicate {key:?}");
            fit::occupancy(&key, RestoreBound::SignIns, domain.sign_ins.len(), 0, env.limits.sign_ins)
        }
        Stored::Roles { project, holdings } => {
            assert!(!domain.roles.contains_key(project), "duplicate {key:?}");
            fit::within(&key, RestoreBound::Holdings, holdings.len(), env.limits.holdings)?;
            fit::occupancy(&key, RestoreBound::Roles, domain.roles.len(), 0, env.limits.projects)
        }
        Stored::ReadResults { party, .. } => {
            assert!(!domain.results_seen.contains_key(party), "duplicate {key:?}");
            fit::occupancy(
                &key,
                RestoreBound::ReadResults,
                domain.results_seen.len(),
                0,
                domain.results_seen.capacity(),
            )
        }
        Stored::ReadConversation { party, task, .. } => {
            assert!(!domain.conversations_seen.contains_key(&(*party, *task)), "duplicate {key:?}");
            fit::occupancy(
                &key,
                RestoreBound::ReadConversations,
                domain.conversations_seen.len(),
                0,
                domain.conversations_seen.capacity(),
            )
        }
        Stored::Answer { key: request, ask, models, outcome, .. } => {
            assert!(!domain.answers.contains_key(request), "duplicate {key:?}");
            fit::ask(&key, &env.limits, ask)?;
            fit::models(&key, models)?;
            fit::outcome(&key, &env.limits, outcome)?;
            fit::occupancy(&key, RestoreBound::Requests, domain.answers.len(), domain.acted.len(), env.limits.requests)
        }
        Stored::Acted { key: request, act, outcome, .. } => {
            assert!(!domain.acted.contains_key(request), "duplicate {key:?}");
            fit::act(&key, &env.limits, act)?;
            fit::outcome(&key, &env.limits, outcome)?;
            fit::occupancy(&key, RestoreBound::Requests, domain.answers.len(), domain.acted.len(), env.limits.requests)
        }
        Stored::Policy { .. } | Stored::EndedRead { .. } => unreachable!("wrong people family: {key:?}"),
    }
}

fn restore_failed(domain: &mut Domain, key: Key, refusal: Refusal, out: &mut Queue<Request>) {
    domain.start = Start::Failed;
    out.push(Request::RestoreRefused { key, refusal });
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::boxed::Box;

    #[test]
    #[ignore = "fails under Cranelift, which cannot unwind yet: run it on LLVM (2026-11-12)"]
    fn readiness_and_lookup_follow_the_start_in_every_state() {
        for (start, ready, lookup, projects) in [
            (Start::NotStarted, true, false, true),
            (Start::Loading, false, true, false),
            (Start::Linked, false, true, true),
            (Start::Sweeping, false, true, false),
            (Start::Open, true, false, true),
            (Start::Failed, false, false, false),
        ] {
            let mut domain = Domain::new(&crate::tests::LIMITS, Box::new([]), 1, Box::new([]));
            domain.start = start;
            assert_eq!(domain.ready(), ready, "{start:?}");
            assert_eq!(domain.start_lookup(), lookup, "{start:?}");
            let projection = std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| {
                domain.assert_projection_event();
            }));
            assert_eq!(projection.is_ok(), projects, "{start:?}");
        }
    }
}
