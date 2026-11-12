//! The core's durable run and call identities (domain/engine/README.md, 5.4 and domain/engine/executors.md).

pub use jig_domain_core_vocabulary::stored::*;

use crate::{Core, Counters, Family, Limits, fresh};

/// Result of restoring one core-owned row during the live-range startup.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Restored {
    /// The deployment header also supplies the journal's durable number.
    Deployment { commits: u64 },
    /// One live core row was retained.
    Live,
    /// An ended row is outside the startup's live ranges.
    Archive,
    /// The row fails the current live proof or bounded identity checks.
    Rejected,
}

impl Core {
    /// Decide whether this named answer belongs to the current durable
    /// claim, retaining it only while that claim still owns the call.
    #[must_use]
    pub fn decide_named_call(&mut self, key: CallKey, part: CallPart) -> bool {
        self.start_input();
        let _pending = self.pending_calls.remove(&key);
        let current = match self.proofs.get(&key.task) {
            Some(proof) => proof.attempt == key.attempt,
            None => false,
        };
        if !current {
            return false;
        }
        let _number = fresh(&mut self.counters, Family::Call).expect("admitted call counter");
        self.record_call(key, part);
        true
    }

    /// Find the retained part of a named call without consulting a
    /// connector's separately owned payload.
    #[must_use]
    pub fn replay_call(&self, key: CallKey) -> Option<CallReplay> {
        match self.call_parts.get(&key) {
            Some(CallPart::Connector { connector }) => Some(CallReplay::Connector { connector: *connector }),
            Some(part) => Some(CallReplay::Core(part.clone())),
            None => None,
        }
    }

    /// Validate a live task-family row against the deployment and retain the
    /// correlations that a later proof and view restoration will need.
    #[must_use]
    pub(crate) fn restore_task_row(&mut self, row: &jig_domain_tasks::Stored) -> bool {
        self.start_input();
        {
            assert!(self.start_loading(), "task rows load before Link");
            assert!(row.key().live(), "ended task row in live load: {:?}", row.key());
        }
        self.remember_task_proof(row)
    }

    #[expect(clippy::wildcard_enum_match_arm, reason = "only live tasks carry run-proof correlation")]
    fn remember_task_proof(&mut self, row: &jig_domain_tasks::Stored) -> bool {
        use jig_domain_tasks as tasks;
        match row {
            jig_domain_tasks::Stored::Live(task) => {
                let hosted_attempt = match task.executor {
                    tasks::Executor::Agent { .. } | tasks::Executor::Job { .. } => task.attempt != 0,
                    tasks::Executor::Procedure { .. } | tasks::Executor::Person(_) => false,
                };
                if hosted_attempt
                    && self
                        .restoring_proofs
                        .insert(
                            task.number,
                            crate::RestoringProof {
                                attempt: task.attempt,
                                turn: task.turn,
                                run_spent: task.run_spent,
                                last_answer: task.last_answer,
                            },
                        )
                        .is_err()
                {
                    let _refused = self.start_room(
                        crate::start::LiveSet::RunProofs,
                        Key::Tasks(tasks::Key::Live(task.number)),
                        self.restoring_proofs.len(),
                        self.restoring_proofs.capacity(),
                    );
                    return false;
                }
                true
            }
            _ => true,
        }
    }

    /// Retain one of the core's own live rows. The root wraps store bytes and
    /// routes connector rows separately; it cannot alter this live decision.
    pub(crate) fn restore_core(&mut self, record: CoreRecord, limits: &Limits) -> Restored {
        self.start_input();
        {
            assert!(self.start_loading(), "core rows load before Link");
            if let CoreRecord::Deployment(_) = &record {
                assert!(self.counters.loading(), "duplicate {:?}", Key::Core(CoreKey::Deployment));
            }
        }
        match record {
            CoreRecord::Settings(record) => {
                if self.restore_setting(record, limits) {
                    Restored::Live
                } else {
                    Restored::Rejected
                }
            }
            CoreRecord::Projection(row) => {
                if crate::effects::projections::restore(self, limits, row) {
                    Restored::Live
                } else {
                    Restored::Rejected
                }
            }
            CoreRecord::Deployment(deployment) => {
                self.counters = Counters::new(deployment);
                Restored::Deployment { commits: deployment.commits }
            }
            CoreRecord::Call(record) => {
                {
                    let excess = match record.part.excess(limits, self.authority.limits()) {
                        Some(excess) => Some(excess),
                        None => match &record.settled {
                            Some(call) => call.excess(limits),
                            None => None,
                        },
                    };
                    if let Some((held, limit)) = excess {
                        let _refused = self.start_refuse(crate::start::Failure::LiveSet {
                            set: crate::start::LiveSet::Calls,
                            key: Key::Core(CoreKey::Call(record.key)),
                            held,
                            limit,
                        });
                        return Restored::Rejected;
                    }
                }
                if self.restore_call(record.key, record.part) {
                    if let Some(call) = record.settled {
                        assert!(
                            self.call_settled.insert(record.key, call).is_ok(),
                            "restored settled names share the call bound"
                        );
                    }
                    Restored::Live
                } else {
                    Restored::Rejected
                }
            }
            CoreRecord::RunProof(proof) => {
                if self.restore_proof(proof, limits.tasks.result_bytes) {
                    Restored::Live
                } else {
                    Restored::Rejected
                }
            }
            CoreRecord::Turn(_)
            | CoreRecord::Terminal(_)
            | CoreRecord::ProposalDecision(_)
            | CoreRecord::EscalationDecision(_) => Restored::Archive,
        }
    }
}
