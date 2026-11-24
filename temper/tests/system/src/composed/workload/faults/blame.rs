//! Reactions observed on committed-output requests, with causes read only from the ledger (blame.md, 4).
use super::{Faults, World};
use crate::composed::engine;
use skein_fake_forge_domain::api as raw;
use skein_world::faults::Struck;
use temper_blame::{Names, Observed, Reaction, WRONG_TREE};
use temper_engine_domain_forge as forge;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as landing;
use temper_fake_checkout::git::Remote;

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

    pub(super) fn record_workload_reds(&self, f: &mut Faults) {
        for run in self.gate_runs() {
            if run.conclusion != Some(raw::Conclusion::Failure) || f.wrong.contains(&run.commit) {
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
}
