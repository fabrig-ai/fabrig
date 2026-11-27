//! Reactions observed on committed-output requests, with causes read only from the ledger (blame.md, 4).
use super::{Faults, World, forge::REMOTE};
use crate::composed::engine;
use skein_fake_forge_domain::{self as fake, api as raw};
use skein_world::faults::Struck;
use temper_blame::{MISSING_REPORT, Names, Observed, Reaction, WRONG_TREE};
use temper_engine_domain_forge as forge;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as landing;
use temper_fake_checkout::git::Remote;

#[derive(Clone, Copy)]
pub(super) enum MissingReport {
    Waiting,
    Armed(u64),
    Fell(u64),
}

pub(super) fn hex(commit: skein_forge::api::Commit) -> String {
    String::from_utf8(forge::commit_text(&commit).into()).expect("hex")
}

impl World {
    pub(in crate::composed) fn workload_blame_writes(&mut self, writes: &[engine::Write]) {
        let Some(mut f) = self.workload_faults.take() else { return };
        for write in writes {
            let engine::Write::Save(engine::Record::Forge { row }) = write else { continue };
            match row.as_ref() {
                forge::Stored::ChangeHot(row) => {
                    if let Some((task, change::Delegate::Repair(why))) = row.delegate
                        && f.repairs.insert(task)
                    {
                        let candidate = match why {
                            change::Repair::LandingAt { candidate, .. } => Some(candidate),
                            _ => row.change.last_head,
                        }
                        .expect("repair candidate");
                        let base = self
                            .store
                            .change(row.task)
                            .and_then(|old| self.forge.branch(b"org/repo", &old.base))
                            .expect("repair base");
                        f.reactions.push(Observed {
                            at: self.env.now,
                            reaction: Reaction::Repair {
                                change: format!("task {}", row.task),
                                candidate: hex(candidate),
                                base: hex(temper_engine_forge_world::translate::commit(base)),
                            },
                        });
                    }
                }
                forge::Stored::ChangeCold(row) => {
                    if let Some(candidate) = row.repairs
                        && f.repairs.insert(row.task)
                    {
                        let base = self.forge.branch(b"org/repo", &row.base).expect("repair base");
                        f.reactions.push(Observed {
                            at: self.env.now,
                            reaction: if row.base_repair {
                                Reaction::BaseRepair { base: hex(candidate) }
                            } else {
                                Reaction::Repair {
                                    change: format!("task {}", row.task),
                                    candidate: hex(candidate),
                                    base: hex(temper_engine_forge_world::translate::commit(base)),
                                }
                            },
                        });
                    }
                }
                forge::Stored::Landing(row) => {
                    let landing::Phase::Held { why, .. } = row.landing.phase else { continue };
                    let (member, run) = match why {
                        landing::Hold::Faults { member, .. } => (member, None),
                        landing::Hold::Timeout { member, run, .. } => (member, Some(run)),
                        _ => continue,
                    };
                    let member = &row.landing.members[usize::from(member)];
                    let run = run.or_else(|| {
                        self.gate_runs()
                            .iter()
                            .filter(|run| {
                                Some(temper_engine_forge_world::translate::commit(run.commit)) == member.candidate
                            })
                            .max_by_key(|run| run.id)
                            .map(|run| run.id)
                    });
                    let identity = format!("{}:{run:?}", row.landing.task);
                    if f.holds.insert(identity) {
                        f.reactions.push(Observed {
                            at: self.env.now,
                            reaction: Reaction::Hold {
                                object: format!("task {}", row.landing.task),
                                names: Names::Runner,
                                run: run.map(|run| run.to_string()),
                                candidate: member.candidate.map(hex),
                                base: member.base.map(hex),
                            },
                        });
                    }
                }
                _ => {}
            }
        }
        self.workload_faults = Some(f);
    }

    pub(super) fn record_workload_reds(&mut self, f: &mut Faults) {
        let runs = self.gate_runs();
        let missing = self.workload_missing_report(f, &runs);
        for run in runs {
            if run.conclusion != Some(raw::Conclusion::Failure)
                || f.wrong.contains(&run.commit)
                || Some(run.id) == missing
            {
                continue;
            }
            let tree = self.forge.tree(run.commit);
            let wrong = self.workload().goals.iter().flat_map(|goal| &goal.changes).any(|change| {
                tree.get(super::super::setup::marker(change).as_bytes()).is_some_and(|bytes| !bytes.is_empty())
            });
            if wrong {
                f.wrong.insert(run.commit);
                let entry = f.ledger.inject(
                    WRONG_TREE,
                    self.env.now,
                    Struck {
                        candidate: Some(hex(temper_engine_forge_world::translate::commit(run.commit))),
                        ..Struck::default()
                    },
                );
                f.ledger.stop(entry, self.env.now).expect("wrong tree observed");
                f.stopped(self.env.now);
            }
        }
    }

    // A missing upload is a runner fault, not evidence for a repair. A
    // later attempt of the same run can still report the change's failure.
    fn workload_missing_report(&mut self, f: &mut Faults, runs: &[raw::Run]) -> Option<u64> {
        match f.missing_report? {
            MissingReport::Waiting => {}
            MissingReport::Armed(id) => {
                if let Some(run) = runs.iter().find(|run| run.id == id) {
                    if run.state != raw::RunState::Completed {
                        return None;
                    }
                    if self.failed_without_report(run) && self.workflow_step_failed(id) {
                        let goal = f.fire(MISSING_REPORT);
                        let entry = f.ledger.inject(
                            MISSING_REPORT,
                            self.env.now,
                            Struck { request: Some(goal), gate_run: Some(id.to_string()), ..Struck::default() },
                        );
                        f.ledger.stop(entry, self.env.now).expect("missing upload observed");
                        f.stopped(self.env.now);
                        f.missing_report = Some(MissingReport::Fell(id));
                        return Some(id);
                    }
                }
                f.missing_report = Some(MissingReport::Waiting);
            }
            MissingReport::Fell(id) => {
                if runs.iter().any(|run| run.id == id && self.failed_without_report(run)) {
                    return Some(id);
                }
                // The job was retried or the run is no longer retained.
                f.missing_report = None;
                return None;
            }
        }
        if f.due(MISSING_REPORT)
            && let Some(id) = self.missing_report_target(f, runs)
        {
            fake::omit_report_once(self.forge.domain_mut(), REMOTE, id).expect("in-flight red gate");
            f.missing_report = Some(MissingReport::Armed(id));
        }
        None
    }

    fn failed_without_report(&self, run: &raw::Run) -> bool {
        if run.conclusion != Some(raw::Conclusion::Failure) {
            return false;
        }
        let Ok(raw::Answer::Artifacts(artifacts)) = self.forge.inspect(REMOTE, &raw::Read::Artifacts { run: run.id })
        else {
            panic!("a known run lists its artifacts");
        };
        artifacts.is_empty()
    }

    fn workflow_step_failed(&self, run: u64) -> bool {
        let Ok(raw::Answer::Jobs(jobs)) = self.forge.inspect(REMOTE, &raw::Read::Jobs { run }) else {
            panic!("a known run lists its jobs");
        };
        // LOST or SET_UP can suppress the upload on their own.
        jobs.iter().any(|job| {
            job.steps.iter().any(|step| step.number == raw::WORKFLOW_STEP && step.status == raw::StepStatus::Failure)
        })
    }

    fn missing_report_target(&self, f: &Faults, runs: &[raw::Run]) -> Option<u64> {
        let goal = self.workload().goal(f.target(MISSING_REPORT)?)?;
        runs.iter().find_map(|run| {
            if run.state == raw::RunState::Completed || self.candidate_goal(run.commit) != Some(&goal.name) {
                return None;
            }
            let name = run.workflow.as_deref()?.strip_suffix(b".yml")?;
            let change = goal.changes.iter().find(|change| change.red_once && change.name.as_bytes() == name)?;
            // A job retry is blamed by run id; keep distinct runner faults on distinct runs.
            let run_id = run.id.to_string();
            if f.ledger.entries().iter().any(|fault| {
                fault.struck.gate_run.as_ref() == Some(&run_id)
                    && temper_blame::class(fault.kind) == Some(temper_blame::Class::Runner)
            }) {
                return None;
            }
            let tree = self.forge.tree(run.commit);
            tree.get(super::super::setup::marker(change).as_bytes())
                .is_some_and(|bytes| !bytes.is_empty())
                .then_some(run.id)
        })
    }
}
