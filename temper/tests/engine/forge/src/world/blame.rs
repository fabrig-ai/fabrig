//! Reactions observed at the world's boundaries (testing/blame.md, 3–4).
use super::{World, raw, top, translate};
use skein_forge::api;
use skein_world::faults::{Kind, Struck};
use std::collections::{BTreeMap, BTreeSet};
use temper_blame::{Names, Observed, Reaction};
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as landing;

#[derive(Default)]
pub(super) struct Blame {
    observed: Vec<Observed>,
    landings: BTreeMap<u64, top::LandingRow>,
    cold: BTreeMap<u64, top::ChangeCold>,
    hot: BTreeMap<u64, top::ChangeHot>,
    retries: BTreeSet<(u64, u64, u32)>,
    wrong: Vec<raw::File>,
    candidates: BTreeSet<u64>,
    scripted: Option<Kind>,
    completed: BTreeSet<(u64, u64, u32)>,
    first_gate_fault: bool,
}

fn checking(state: &change::State) -> bool {
    match state {
        change::State::Checking { .. } => true,
        change::State::Held { was, .. } => checking(was),
        _ => false,
    }
}

fn gate_name(repository: api::Repository, key: &[u8]) -> String {
    format!("{}:{}:{}", repository.forge, repository.repository, String::from_utf8_lossy(key))
}

pub fn hex(commit: api::Commit) -> String {
    String::from_utf8(top::commit_text(&commit).into()).expect("commit hex")
}

impl World {
    /// Keep a second, non-draining observation stream for the blame referee.
    pub fn watch_blame(&mut self) {
        assert!(self.blame.is_none());
        self.blame = Some(Blame::default());
    }

    #[must_use]
    pub fn blame(&self) -> &[Observed] {
        &self.blame.as_ref().expect("blame enabled").observed
    }

    /// Fail the first dispatched job, then restore unfaulted runners.
    pub fn gate_fault_once(&mut self, kind: Kind) {
        assert!(kind == temper_blame::LOST || kind == temper_blame::SET_UP);
        self.blame.as_mut().expect("blame enabled").first_gate_fault = true;
        self.runner_lost(u32::from(kind == temper_blame::LOST) * 1000);
        self.runner_setup_faults(u32::from(kind == temper_blame::SET_UP) * 1000);
    }

    pub(super) fn after_gate_fault(&mut self, kind: Kind) {
        if (kind == temper_blame::LOST || kind == temper_blame::SET_UP)
            && self.blame.as_ref().is_some_and(|b| b.first_gate_fault)
        {
            self.runner_lost(0);
            self.runner_setup_faults(0);
            self.blame.as_mut().expect("enabled").first_gate_fault = false;
        }
    }

    /// Script head CI's failure and record each failed attempt as it completes.
    pub fn head_runner_fault(&mut self, fault: raw::Failing) {
        self.blame.as_mut().expect("blame enabled").scripted = match fault {
            raw::Failing::Setup => Some(temper_blame::SET_UP),
            raw::Failing::NoStep => Some(temper_blame::LOST),
            raw::Failing::Workflow => None,
        };
        super::forge::set_failing(&mut self.forge, b"org/repo", b"build", fault);
    }

    pub fn push_wrong_tree(&mut self, branch: &[u8], path: &[u8], content: &[u8]) {
        self.push_file(branch, path, content);
        if let Some(blame) = &mut self.blame {
            let witness = raw::File { path: path.into(), content: content.into() };
            if !blame.wrong.contains(&witness) {
                blame.wrong.push(witness);
            }
        }
        self.record_wrong_candidate(self.branch(branch));
    }

    pub(super) fn record_wrong_candidate(&mut self, candidate: u64) {
        let Some(mut blame) = self.blame.take() else { return };
        let wrong = blame.wrong.iter().any(|witness| {
            matches!(self.inspect(&raw::Read::File { commit: candidate, path: witness.path.clone() }),
                raw::Answer::File(file) if file == witness.content)
        });
        if wrong && blame.candidates.insert(candidate) {
            self.inject(
                temper_blame::WRONG_TREE,
                Struck { candidate: Some(hex(translate::commit(candidate))), ..Struck::default() },
            );
        }
        self.blame = Some(blame);
    }

    pub fn red_base(&mut self, branch: &[u8]) {
        self.status(branch, raw::Check::Failed);
        self.inject(
            temper_blame::BASE_RED,
            Struck { base: Some(hex(translate::commit(self.branch(branch)))), ..Struck::default() },
        );
    }

    /// Cancel in flight as a user other than the dispatcher.
    pub fn cancel_ci(&mut self, run: u64) {
        let raw::Answer::Run(before) = self.inspect(&raw::Read::Run { run }) else { panic!("run") };
        assert_eq!(before.state, raw::RunState::Running);
        self.external_repository_at(
            b"org/repo",
            raw::Op::Write(raw::Write::Cancel { run }),
            2,
            self.env.now,
            self.env.wall,
        );
        let raw::Answer::Run(after) = self.inspect(&raw::Read::Run { run }) else { panic!("run") };
        assert_eq!(after.cancelled_by, Some(2));
        self.inject(temper_blame::CANCELLED, Struck { gate_run: Some(run.to_string()), ..Struck::default() });
    }

    #[must_use]
    pub fn ci_runs(&self) -> Vec<raw::Run> {
        let mut result = Vec::new();
        for page in 1.. {
            let raw::Answer::Runs(runs) =
                self.inspect(&raw::Read::Runs { workflow: None, commit: None, number: None, page, limit: 8 })
            else {
                panic!("runs")
            };
            let done = runs.len() < 8;
            result.extend(runs);
            if done {
                break;
            }
        }
        result
    }

    pub(super) fn record_ci_completions(&mut self) {
        let Some(mut blame) = self.blame.take() else { return };
        for run in self.ci_runs() {
            if run.conclusion != Some(raw::Conclusion::Failure) {
                continue;
            }
            let raw::Answer::Jobs(jobs) = self.inspect(&raw::Read::Jobs { run: run.id }) else { panic!("jobs") };
            for job in jobs {
                if job.conclusion != Some(raw::Conclusion::Failure)
                    || !blame.completed.insert((run.id, job.id, job.attempt))
                {
                    continue;
                }
                let kind = if run.workflow.is_none() { blame.scripted } else { self.gate_report_fault(&run) };
                if let Some(kind) = kind {
                    self.inject(kind, Struck { gate_run: Some(run.id.to_string()), ..Struck::default() });
                }
            }
        }
        self.blame = Some(blame);
    }

    fn gate_report_fault(&self, run: &raw::Run) -> Option<Kind> {
        let report = self
            .landing_config
            .as_ref()?
            .2
            .iter()
            .flat_map(|repo| &repo.gates)
            .find(|gate| Some(&gate.workflow) == run.workflow.as_ref())?
            .report
            .as_ref()?;
        let raw::Answer::Artifacts(artifacts) = self.inspect(&raw::Read::Artifacts { run: run.id }) else {
            panic!("a known run lists its artifacts")
        };
        let Some(artifact) = artifacts.iter().find(|artifact| artifact.name == report.artifact) else {
            return Some(temper_blame::MISSING_REPORT);
        };
        if report.format == landing::Format::Lines
            && matches!(self.inspect(&raw::Read::ArtifactById { id: artifact.id }), raw::Answer::Artifact { artifact, .. } if artifact.bytes.is_empty())
        {
            return Some(temper_blame::STARVED);
        }
        None
    }

    fn observed(&mut self, reaction: Reaction) {
        let at = self.env.now;
        self.blame.as_mut().expect("blame enabled").observed.push(Observed { at, reaction });
    }

    pub(super) fn observe_blame_event(&mut self, event: &top::Event) {
        if let Some(blame) = &mut self.blame {
            match event {
                top::Event::Change { row } => {
                    let (cold, hot) = row.clone().split();
                    blame.cold.insert(row.task, cold);
                    blame.hot.insert(row.task, hot);
                }
                top::Event::Landing { row } => {
                    blame.landings.insert(row.landing.task, row.clone());
                }
                _ => {}
            }
        }
    }

    pub(super) fn observe_blame_request(&mut self, request: &top::Request) {
        if self.blame.is_none() {
            return;
        }
        match request {
            top::Request::Save { record: top::Stored::ChangeCold(row), .. } => {
                self.blame.as_mut().expect("enabled").cold.insert(row.task, *row.clone());
            }
            top::Request::Save { record: top::Stored::ChangeHot(row), .. } => {
                self.blame.as_mut().expect("enabled").hot.insert(row.task, *row.clone());
            }
            top::Request::Save { record: top::Stored::Landing(row), .. } => {
                self.blame.as_mut().expect("enabled").landings.insert(row.landing.task, row.clone());
            }
            top::Request::Save { record: top::Stored::HistoryEvent(event), .. }
                if matches!(event.what, top::history::What::Paused { why: top::history::Pause::BranchRed }) =>
            {
                self.observed(Reaction::Hold {
                    object: format!("task {}", event.change),
                    names: Names::Base,
                    run: None,
                    candidate: None,
                    base: event.base.map(hex),
                });
            }
            top::Request::ChangeDecision { task, decision, evidence, .. } => {
                let blame = self.blame.as_ref().expect("enabled");
                let Some(row) = blame.hot.get(task) else { return };
                let cold = &blame.cold[task];
                let candidate = evidence.as_ref().and_then(|e| e.head).or(row.change.last_head);
                let base = evidence.as_ref().map_or_else(|| translate::commit(self.branch(&cold.base)), |e| e.base_tip);
                let reaction = match decision {
                    change::Decision::Delegate(change::Delegate::Repair(why)) => {
                        let candidate = match why {
                            change::Repair::LandingAt { candidate, .. } => Some(*candidate),
                            _ => candidate,
                        }
                        .expect("repair's candidate");
                        Some(Reaction::Repair {
                            change: format!("task {task}"),
                            candidate: hex(candidate),
                            base: hex(base),
                        })
                    }
                    change::Decision::QueueRepair => Some(Reaction::BaseRepair { base: hex(base) }),
                    change::Decision::Hold(hold) => {
                        let names = match hold {
                            _ if cold.base_repair => Some(Names::Base),
                            change::Hold::Runner => Some(Names::Runner),
                            change::Hold::Stalled if checking(&row.change.state) => Some(Names::Runner),
                            change::Hold::Repairs
                            | change::Hold::Failed
                            | change::Hold::NoTest
                            | change::Hold::RedStands => Some(Names::Change),
                            _ => None,
                        };
                        names.map(|names| Reaction::Hold {
                            object: format!("task {task}"),
                            names,
                            run: candidate
                                .and_then(|head| {
                                    self.ci_runs()
                                        .into_iter()
                                        .find(|run| run.workflow.is_none() && run.commit == translate::number(head))
                                })
                                .map(|run| run.id.to_string()),
                            candidate: candidate.map(hex),
                            base: Some(hex(base)),
                        })
                    }
                    _ => None,
                };
                if let Some(reaction) = reaction {
                    self.observed(reaction);
                }
            }
            top::Request::LandingDecision { task, decision: landing::Decision::Hold(hold), .. } => {
                self.observe_landing_hold(*task, hold);
            }
            _ => {}
        }
    }

    fn observe_landing_hold(&mut self, task: u64, hold: &landing::Hold) {
        let (member, gate, run) = match hold {
            landing::Hold::Faults { member, gate } => (*member, *gate, None),
            landing::Hold::Timeout { member, gate, run } => (*member, *gate, Some(*run)),
            _ => return,
        };
        let row = &self.blame.as_ref().expect("enabled").landings[&task];
        let member = &row.landing.members[usize::from(member)];
        let candidate = member.candidate;
        let workflow = self.landing_config.as_ref().and_then(|(_, _, repositories)| {
            repositories
                .iter()
                .find(|repository| {
                    repository.repository.forge == member.repository.forge
                        && repository.repository.repository == member.repository.repository
                })
                .and_then(|repository| repository.gates.get(usize::from(gate)))
                .map(|gate| &gate.workflow)
        });
        let run = run.or_else(|| {
            self.ci_runs()
                .into_iter()
                .filter(|r| r.workflow.as_ref() == workflow && Some(translate::commit(r.commit)) == candidate)
                .max_by_key(|r| r.id)
                .map(|r| r.id)
        });
        self.observed(Reaction::Hold {
            object: format!("task {task}"),
            names: Names::Runner,
            run: run.map(|id| id.to_string()),
            candidate: candidate.map(hex),
            base: member.base.map(hex),
        });
    }

    pub(super) fn observe_blame_call(&mut self, repository: api::Repository, op: &api::Op) {
        if self.blame.is_none() {
            return;
        }
        match op {
            api::Op::Write(api::Write::Rerun { attempt }) => {
                let raw::Answer::Run(run) = self.inspect(&raw::Read::Run { run: attempt.run }) else { panic!("rerun") };
                if self.blame.as_mut().expect("enabled").retries.insert((attempt.run, attempt.job, attempt.attempt)) {
                    let gate = run.inputs.iter().find(|input| input.0.as_ref() == b"temper_key").map_or_else(
                        || format!("{}:{}:job {}", repository.forge, repository.repository, attempt.job),
                        |input| gate_name(repository, &input.1),
                    );
                    let bound = if run.workflow.is_some() {
                        u64::from(self.landing_policy.gate_retries)
                    } else {
                        u64::from(self.env.limits.change_policy.reruns)
                    };
                    self.observed(Reaction::GateRetry { gate, run: attempt.run.to_string(), bound });
                }
            }
            api::Op::Write(api::Write::Dispatch { inputs, .. }) => {
                let previous = self
                    .ci_runs()
                    .into_iter()
                    .filter(|run| {
                        run.workflow.is_some()
                            && run.inputs.iter().any(|old| {
                                inputs.iter().any(|input| {
                                    old.0 == input.name && old.1 == input.value && input.name.as_ref() == b"temper_key"
                                })
                            })
                    })
                    .max_by_key(|run| run.id);
                if let Some(run) = previous.filter(|run| run.conclusion != Some(raw::Conclusion::Success)) {
                    let fault = self
                        .ledger
                        .fallen(self.env.now, &Struck { gate_run: Some(run.id.to_string()), ..Struck::default() })
                        .iter()
                        .any(|entry| {
                            temper_blame::class(self.ledger.get(*entry).kind) == Some(temper_blame::Class::Runner)
                        });
                    if fault {
                        let key =
                            inputs.iter().find(|input| input.name.as_ref() == b"temper_key").expect("matched gate key");
                        self.observed(Reaction::GateRetry {
                            gate: gate_name(repository, &key.value),
                            run: run.id.to_string(),
                            bound: u64::from(self.landing_policy.gate_retries),
                        });
                    }
                }
            }
            _ => {}
        }
    }
}
