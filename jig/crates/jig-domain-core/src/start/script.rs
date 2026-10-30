//! Script transitions and journal release identities (jig/docs/design/domain/engine.md, 6).
use super::{Action, Continuation, Failure, LiveSet, Phase, SweepInput};
use crate::{Core, Key, Limits, Queue, Request, Write, people, tasks};
use skein_lib::Env;

impl Core {
    pub fn start_done(&mut self, env: &Env<Limits>, done: Phase, out: &mut Queue<Request>) -> Action {
        self.start_input();
        self.start_collect_failure();
        if let Some(failure) = self.start_failure() {
            return Action::Refused(failure);
        }
        assert!(self.start.phase == Some(done) && self.start.waiting.is_none(), "start phase completed in order");
        let next = match done {
            Phase::LoadCore => {
                self.counters.finish_restore();
                if self.counters.dirty() {
                    crate::routing::first_start::start_seed(self, env, out);
                    self.start_collect_failure();
                    if let Some(failure) = self.start_failure() {
                        return Action::Refused(failure);
                    }
                    return self.start_commit(Phase::LoadInbox);
                }
                Phase::LoadInbox
            }
            Phase::LoadInbox => {
                assert!(self.census_position.is_none(), "census pair complete before Link");
                match self.connectors.first() {
                    Some(&connector) => Phase::RestoreConnector { connector },
                    None => Phase::Link,
                }
            }
            Phase::RestoreConnector { connector } => match self.next_connector(connector) {
                Some(connector) => Phase::RestoreConnector { connector },
                None => Phase::Link,
            },
            Phase::Link => {
                let before = out.len();
                self.start_link(env, out);
                if let Some(failure) = self.start_failure() {
                    return Action::Refused(failure);
                }
                self.people.start_sweep();
                self.restoring_proofs = skein_lib::Map::with_capacity(self.restoring_proofs.capacity());
                if out.len() != before {
                    return self.start_commit(Phase::Sweep);
                }
                Phase::Sweep
            }
            Phase::Sweep => unreachable!("sweep completes through Finished"),
            Phase::Switch => Phase::Wires,
            Phase::Wires => return self.start_commit(Phase::AdoptRuns),
            Phase::AdoptRuns => {
                assert!(self.adopted.is_empty(), "restored claims delivered before reads");
                match self.connectors.first() {
                    Some(&connector) => Phase::ReadAfresh { connector },
                    None => Phase::Open,
                }
            }
            Phase::ReadAfresh { connector } => match self.next_connector(connector) {
                Some(connector) => Phase::ReadAfresh { connector },
                None => match self.connectors.first() {
                    Some(&connector) => Phase::SettleOutbox { connector },
                    None => Phase::Open,
                },
            },
            Phase::SettleOutbox { connector } => match self.next_connector(connector) {
                Some(connector) => Phase::SettleOutbox { connector },
                None => Phase::Open,
            },
            Phase::Open => {
                self.start.phase = None;
                return Action::Idle;
            }
        };
        self.start_ask(next)
    }

    pub fn start_sweep(&mut self, env: &Env<Limits>, input: SweepInput, out: &mut Queue<Request>) -> Action {
        self.start_input();
        if let Some(failure) = self.start_failure() {
            return Action::Refused(failure);
        }
        assert!(self.start.phase == Some(Phase::Sweep) && self.start.waiting.is_none(), "sweep waits for its commit");
        match input {
            SweepInput::Record(record) => {
                let mut child = Queue::uncapped();
                self.people.start_sweep_row(
                    &Env { now: env.now, wall: self.start.at, limits: env.limits.people },
                    &record,
                    &mut child,
                );
                if child.is_empty() {
                    return Action::Idle;
                }
                for _ in 0..child.len() {
                    let request = child.pop().expect("one expiry output per row");
                    #[expect(clippy::wildcard_enum_match_arm, reason = "the borrowed expiry walk emits only erases")]
                    match request {
                        people::Request::Erase { key } => out.push(Request::Write(Write::Erase(Key::People(key)))),
                        _ => unreachable!("expiry walk only erases"),
                    }
                }
                self.start_commit(Phase::Sweep)
            }
            SweepInput::Finished => self.start_ask(Phase::Switch),
        }
    }

    pub fn start_resume(&mut self, env: &Env<Limits>, continuation: Continuation, out: &mut Queue<Request>) -> Action {
        self.start_input();
        if let Some(failure) = self.start_failure() {
            return Action::Refused(failure);
        }
        assert!(self.start.waiting.as_ref() == Some(&continuation), "stale or out-of-order start continuation");
        self.start.waiting = None;
        match continuation.phase {
            Phase::AdoptRuns => {
                assert!(
                    self.people.start_complete(&Env { now: env.now, wall: env.wall, limits: env.limits.people }),
                    "linked people complete after durable Wires"
                );
                let _adoption = self.start_ask(Phase::AdoptRuns);
                for _ in 0..self.start.held.len() {
                    let request = self.start.held.pop().expect("start output held through Wires");
                    out.push(request);
                }
                let pending = core::mem::replace(&mut self.start.waiting_tasks, Queue::uncapped());
                crate::routing::start_task_outputs(self, env, pending, out);
                let crate::Requests::Out(mut activated) =
                    crate::step(self, env, crate::Event::Tasks(tasks::Event::Restored));
                for _ in 0..activated.len() {
                    let request = activated.pop().expect("bounded restored activation outputs");
                    out.push(request);
                }
            }
            Phase::LoadInbox | Phase::Sweep => {}
            Phase::LoadCore
            | Phase::RestoreConnector { .. }
            | Phase::Link
            | Phase::Switch
            | Phase::Wires
            | Phase::ReadAfresh { .. }
            | Phase::SettleOutbox { .. }
            | Phase::Open => {
                unreachable!("phase has no durable continuation")
            }
        }
        self.start_ask(continuation.phase)
    }

    #[expect(clippy::manual_map, reason = "the step-code subset uses exhaustive matches")]
    pub(crate) fn start_collect_failure(&mut self) {
        if self.start.phase.is_none() || self.start.failure.is_some() {
            return;
        }
        let failure = if let Some(failure) = self.people.restore_failure() {
            let people::RestoreCause::Bound { which, held, limit } = failure.cause;
            Some(Failure::LiveSet { set: LiveSet::PeopleBound(which), key: Key::People(failure.key), held, limit })
        } else if let Some(failure) = self.tasks.restore_failure() {
            let key = Key::Tasks(failure.key);
            Some(match failure.cause {
                tasks::RestoreCause::Bound { which, held, limit } => {
                    Failure::LiveSet { set: LiveSet::TasksBound(which), key, held, limit }
                }
                tasks::RestoreCause::Accounting => Failure::Accounting { key },
                tasks::RestoreCause::LlmChoice { .. } => Failure::LlmChoice { key },
            })
        } else if let Some(request) = self.restore_failure {
            match request {
                crate::RestartRequest::Settings { failure, changed } => Some(Failure::Settings { failure, changed }),
                crate::RestartRequest::ProjectPolicy { project } => Some(Failure::ProjectPolicy { project }),
                crate::RestartRequest::HeldPastLimit { which, held, limit } => {
                    Some(Failure::HeldPastLimit { which, held, limit })
                }
                crate::RestartRequest::UnheldRevision { task, revision } => {
                    assert!(
                        self.held(revision),
                        "{:?} links missing settings revision {revision}",
                        Key::Tasks(tasks::Key::Live(task))
                    );
                    None
                }
                crate::RestartRequest::Unresolved { .. }
                | crate::RestartRequest::Step(_)
                | crate::RestartRequest::Idle
                | crate::RestartRequest::Refused { .. } => None,
            }
        } else {
            match self.accounts.restore_limit() {
                Some(failure) => Some(Failure::AccountsPastLimit { failure }),
                None => None,
            }
        };
        if let Some(failure) = failure {
            let _refused = self.start_refuse(failure);
        }
    }

    fn start_link(&mut self, env: &Env<Limits>, out: &mut Queue<Request>) {
        if let Some((record, target)) = &self.start.missing {
            assert!(self.start.missing.is_none(), "{record:?} links missing {target:?}");
        }
        if let Err((project, _reason)) = self.authority.start_link() {
            let _refused = self.start_refuse(Failure::ProjectPolicy { project });
            return;
        }
        if !self.people.start_link(&Env { now: env.now, wall: self.start.at, limits: env.limits.people }) {
            self.start_collect_failure();
            return;
        }
        let mut child = Queue::uncapped();
        tasks::step(
            &mut self.tasks,
            &Env { now: env.now, wall: env.wall, limits: env.limits.tasks },
            tasks::Event::Check,
            &mut child,
        );
        self.start_collect_failure();
        if self.start.failure.is_some() {
            return;
        }
        self.start_cross_links();
        let crate::Requests::Out(mut accounts) = crate::want_accounts(self, env);
        self.start_collect_failure();
        if self.start.failure.is_some() {
            return;
        }
        for _ in 0..accounts.len() {
            let request = accounts.pop().expect("accounts restore writes");
            out.push(request);
        }
        for _ in 0..child.len() {
            let request = child.pop().expect("bounded derived wait writes");
            #[expect(
                clippy::wildcard_enum_match_arm,
                reason = "Check links and publishes derived wait state without activation"
            )]
            match request {
                tasks::Request::Save { record } => out.push(Request::Write(Write::Save(crate::Record::Tasks(record)))),
                tasks::Request::Released { revision } => self.release_revision(env, revision, out),
                request => self.start.waiting_tasks.push(request),
            }
        }
    }

    fn start_ask(&mut self, phase: Phase) -> Action {
        self.start.phase = Some(phase);
        Action::Step(phase)
    }

    fn start_commit(&mut self, phase: Phase) -> Action {
        self.start.sequence = self.start.sequence.checked_add(1).expect("start continuation number");
        let continuation = Continuation { sequence: self.start.sequence, phase };
        self.start.waiting = Some(continuation.clone());
        Action::Commit(continuation)
    }
}
