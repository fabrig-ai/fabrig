//! The workload fires people through the same authenticated routes as its focused stories.
use super::{
    Act, BOUND, CHANNEL, Driver, Duration, Ends, Expect, OPENING, Party, Record, Sweep, core, ended, engine, people,
    running, task, tasks,
};
use crate::acts;
use skein_world::faults::Struck;

impl Driver {
    #[expect(
        clippy::too_many_lines,
        reason = "one exhaustive dispatcher keeps each drawn act beside its authenticated route"
    )]
    pub(super) fn fire_actions(&mut self, sweep: &Sweep) -> Result<(), String> {
        self.pending_acts.extend(self.watch.due(&self.world.store, &self.names, self.step));
        let pending = std::mem::take(&mut self.pending_acts);
        for action in pending {
            let Some(target) = self.names.task(&action.target) else { continue };
            let goal = self
                .workload
                .goal(&action.target)
                .or_else(|| self.workload.change(&action.target).map(|(goal, _)| goal))
                .expect("drawn target");
            let goal_number = self.names.task(&goal.name).expect("goal named");
            let holding = if matches!(action.act, Act::AnswerLate | Act::AnswerTwice)
                && task(&self.world, target).is_some_and(|row| !row.waiting_on.is_empty())
            {
                vec!["a dependency holds the answer"]
            } else if action.act == Act::CancelGoal && crate::acts::goal_branch_making(&self.world, target) {
                vec!["a goal branch making in flight"]
            } else if action.act == Act::AmendPassedDeadline && action.party == goal.party {
                vec!["the amending party started the goal"]
            } else {
                Vec::new()
            };
            if let Some(exclusion) = sweep.leaves_out(acts::kind(action.act), &holding) {
                self.ledger.leave_out(exclusion.scenario, acts::kind(action.act), self.world.env.now);
                self.unanswered.remove(&action.target);
                continue;
            }
            match action.act {
                Act::CancelGoal | Act::CancelChange => {
                    if ended(&self.world, target) {
                        continue;
                    }
                    self.cancel(action.party, target);
                }
                Act::CancelLanding => {
                    let Some(landing) = acts::landing(&self.world.store, target).map(|row| row.landing.task) else {
                        continue;
                    };
                    self.cancel(action.party, landing);
                }
                Act::HoldMember => {
                    if ended(&self.world, target) || acts::landing(&self.world.store, target).is_none() {
                        continue;
                    }
                    let result = self.request(action.party, people::Ask::Stop { project: 1, task: target });
                    check_request(&result, "hold member")?;
                }
                Act::ReleaseMember => {
                    let Some(row) = task(&self.world, target) else { continue };
                    if !matches!(row.phase, tasks::Phase::Held { .. }) {
                        continue;
                    }
                    let revision = row.revisions.hold;
                    let result = self.request(
                        action.party,
                        people::Ask::Release { project: 1, task: target, revision, again: None },
                    );
                    check_request(&result, "release member")?;
                }
                Act::AmendPassedDeadline => {
                    let row = task(&self.world, target).expect("deadline target");
                    if !matches!(row.phase, tasks::Phase::Held { why: tasks::Hold::Deadline, .. }) {
                        continue;
                    }
                    let revision = row.revisions.spec;
                    let release = row.revisions.hold;
                    let mut authority = person_authority(&row.authority);
                    authority.deadline = None;
                    let result = self.request(
                        action.party,
                        people::Ask::Amend {
                            project: 1,
                            task: target,
                            revision,
                            release: Some(release),
                            amendment: people::Amendment {
                                tracked: None,
                                addressed: None,
                                instructions: None,
                                spec: None,
                                wake: None,
                                dependencies: None,
                                authority: Some(authority),
                                llm: None,
                                reason: b"Continue after the passed deadline".as_slice().into(),
                            },
                        },
                    );
                    check_request(&result, "amend passed deadline")?;
                }
                Act::AnswerLate | Act::AnswerTwice | Act::AnswerEndedRun => {
                    let Some(goal_run) = running(&self.world, goal_number) else {
                        self.pending_acts.push(action);
                        continue;
                    };
                    let Some(question) =
                        task(&self.world, target).and_then(|row| row.questions.first()).map(|q| q.number)
                    else {
                        self.pending_acts.push(action);
                        continue;
                    };
                    let producer = acts::producers(&self.world.store, target)
                        .max_by_key(|row| row.number)
                        .map(|row| row.number)
                        .expect("question producer");
                    self.next_call += 1;
                    self.world.goal_answer(&goal_run, self.next_call, target, question, b"Proceed");
                    if action.act == Act::AnswerTwice {
                        self.world.goal_answer(&goal_run, self.next_call, target, question, b"Proceed");
                    }
                    if action.act == Act::AnswerEndedRun {
                        let expected = format!("Task {producer} ended; your answer to its question");
                        self.observations.push((goal_number, target, expected));
                        self.resume_after_answer.insert(target);
                    } else {
                        self.observations.push((producer, target, "answer once".into()));
                    }
                    self.unanswered.remove(&action.target);
                }
                Act::NeverAnswer => {}
                Act::WordsToEndedProducer => {
                    let Some(goal_run) = running(&self.world, goal_number) else {
                        self.pending_acts.push(action);
                        continue;
                    };
                    let answer = self.call(
                        &goal_run,
                        engine::Tool::Message {
                            target,
                            form: engine::MessageForm::Words,
                            words: b"Words after producing".as_slice().into(),
                        },
                    );
                    assert!(matches!(answer, engine::CallAnswer::Sent { .. }), "words to ended producer: {answer:?}");
                    self.observations.push((goal_number, target, "Relays questions".into()));
                }
            }
            let now = self.world.env.now;
            let injected = self.ledger.inject(
                acts::kind(action.act),
                now,
                Struck { request: Some(action.target.clone()), ..Struck::default() },
            );
            self.ledger.stop(injected, now)?;
            self.world.ends.faults_stop(now);
            self.fallen.push(action);
        }
        Ok(())
    }

    fn cancel(&mut self, party: Party, target: u64) {
        let result = self.request(
            party,
            people::Ask::Cancel {
                project: 1,
                task: target,
                reason: b"Workload person's cancellation".as_slice().into(),
                grace: Some(Duration::ZERO),
            },
        );
        assert!(!matches!(result, people::Outcome::Refused(_)), "cancel {target}: {result:?}");
    }

    pub(super) fn end_askers(&mut self) {
        for target in self.resume_after_answer.clone() {
            if let Some(row) = task(&self.world, target)
                && matches!(row.phase, tasks::Phase::Held { .. })
            {
                let revision = row.revisions.hold;
                let logical = self.names.logical(target).expect("named change");
                let party = self.workload.change(logical).expect("change").0.party;
                let result =
                    self.request(party, people::Ask::Release { project: 1, task: target, revision, again: None });
                assert!(!matches!(result, people::Outcome::Refused(_)), "resume after ended asker: {result:?}");
                self.resume_after_answer.remove(&target);
            }
        }
        let targets: Vec<_> = self
            .unanswered
            .iter()
            .filter_map(|(name, act)| (*act == Act::AnswerEndedRun).then_some(name.clone()))
            .collect();
        for name in targets {
            let Some(target) = self.names.task(&name) else { continue };
            if task(&self.world, target).is_none_or(|row| row.questions.is_empty()) {
                continue;
            }
            let Some(producer) =
                acts::producers(&self.world.store, target).find(|row| !row.questions.is_empty()).map(|row| row.number)
            else {
                continue;
            };
            if self.ending_askers.insert(producer) {
                let party = self.workload.change(&name).expect("change").0.party;
                self.cancel(party, producer);
            }
        }
    }

    pub(super) fn settle_stops(&mut self) {
        // This pass only needs identities, not the retained brief and charter.
        for index in 0..self.world.assigned.len() {
            let run = &self.world.assigned[index];
            if self.completed.contains(&(run.task, run.attempt)) {
                continue;
            }
            let stopping = task(&self.world, run.task).is_some_and(|row| match &row.phase {
                tasks::Phase::Closing(tasks::Closing { stage: tasks::Stage::Run { attempt }, .. })
                | tasks::Phase::Held { was: tasks::Was::Active(tasks::Active::Running { attempt }), .. } => {
                    *attempt == run.attempt
                }
                _ => false,
            });
            if stopping {
                let key = (run.task, run.attempt);
                let event =
                    self.world.protocol.answer(CHANNEL, key.0, key.1, 0, tasks::End::Parked, engine::Parts::none());
                self.world.send(event);
                self.completed.insert(key);
            }
        }
    }

    pub(super) fn prepare_jobs(&mut self) {
        while let Some(job) = self.world.jobs.first() {
            let number = job.run.raw();
            if ended(&self.world, number) {
                self.world.jobs.remove(0);
                continue;
            }
            let due = self.jobs_at.entry(number).or_insert_with(|| self.step + self.scheduling.between(10, 70));
            if self.step < *due {
                break;
            }
            let result = self.world.push_prepare();
            let job = self.world.jobs.remove(0);
            let event = self.world.protocol.answer_job(
                CHANNEL,
                number,
                job.attempt.raw(),
                jig_domain_host::JobEnding::Finished {
                    result,
                    work: jig_domain_host::Work { left: None, saved: None },
                },
            );
            self.world.send(event);
        }
    }

    fn waits(&self, name: &str) -> bool {
        self.fallen.iter().any(|action| {
            let target_goal = self
                .workload
                .goal(&action.target)
                .or_else(|| self.workload.change(&action.target).map(|(goal, _)| goal))
                .expect("target goal");
            target_goal.name == name
                && (action.act == Act::NeverAnswer
                    || (action.act == Act::HoldMember
                        && !self
                            .actions
                            .iter()
                            .any(|release| release.act == Act::ReleaseMember && release.target == action.target)))
        })
    }

    pub(super) fn finished(&self) -> bool {
        !self.watch.delaying()
            && self.pending_acts.is_empty()
            && self
                .workload
                .goals
                .iter()
                .all(|goal| ended(&self.world, self.names.task(&goal.name).expect("goal")) || self.waits(&goal.name))
    }

    pub(super) fn judge_goals(&mut self) -> Result<(), String> {
        for goal in &self.workload.goals {
            let number = self.names.task(&goal.name).expect("goal");
            let deadline_pending = !ended(&self.world, number)
                && self
                    .actions
                    .iter()
                    .any(|action| action.act == Act::AmendPassedDeadline && action.target == goal.name)
                && !self
                    .fallen
                    .iter()
                    .any(|action| action.act == Act::AmendPassedDeadline && action.target == goal.name);
            if deadline_pending {
                continue;
            }
            let struck = self.actions.iter().any(|action| {
                action.target == goal.name || goal.changes.iter().any(|change| change.name == action.target)
            });
            let expect = if self.waits(&goal.name) {
                Expect::Waits
            } else if struck || self.actions.is_empty() || self.world.workload_deadlines.is_some() {
                Expect::Ends
            } else {
                Expect::Untouched
            };
            let last_fault = self
                .ledger
                .entries()
                .iter()
                .chain(self.world.restarts.iter().flat_map(|injector| injector.ledger.entries()))
                .map(|entry| entry.until.unwrap_or(entry.from))
                .max();
            let progress = self.goals.get_mut(&goal.name).expect("goal");
            if progress.expect != Some(expect) {
                let mut ends = Ends::new(self.seed, OPENING);
                ends.expect(number, expect, BOUND)?;
                ends.faults_stop(self.world.env.now);
                progress.ends = Some(ends);
                progress.expect = Some(expect);
                progress.fault_at = last_fault;
            } else if last_fault != progress.fault_at {
                if let Some(at) = last_fault {
                    progress.ends.as_mut().expect("goal referee").faults_stop(at);
                }
                progress.fault_at = last_fault;
            }
            progress.ends.as_mut().expect("goal referee").step(self.world.env.now, &self.world.store).map_err(
                |finding| {
                    format!(
                        "{finding}; tasks {:?}; assignments {:?}",
                        self.world
                            .store
                            .rows
                            .values()
                            .filter_map(|record| {
                                if let Record::Core(core::Record::Tasks(tasks::Stored::Live(row))) = record {
                                    Some((row.number, row.requester, &row.phase, row.tries, &row.waiting_on))
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>(),
                        self.world.assigned.iter().map(|run| (run.task, run.attempt)).collect::<Vec<_>>()
                    )
                },
            )?;
        }
        Ok(())
    }

    pub(super) fn check_ends(&mut self) -> Result<(), String> {
        for progress in self.goals.values_mut() {
            progress.ends.as_mut().expect("every goal judged").close(self.world.env.now, &self.world.store)?;
        }
        self.world.ends.close(self.world.env.now, &self.world.store)?;
        for (target, source, expected) in &self.observations {
            if expected == "answer once" {
                let count = self.world.store.rows.values().filter(|record| matches!(record, Record::Core(core::Record::Tasks(tasks::Stored::Activity(line))) if line.task == *target && matches!(line.line, tasks::Line::Words { kind: tasks::MessageKind::Answer { .. }, .. }))).count();
                if count != 1 {
                    return Err(format!("producer {target}: answer reached its question {count} times"));
                }
            } else {
                let mut messages = std::collections::BTreeSet::new();
                if let Some(row) = task(&self.world, *target) {
                    messages.extend(
                        row.inbox
                            .iter()
                            .filter(|word| {
                                word.from == tasks::Party::Task(*source) && word.words.starts_with(expected.as_bytes())
                            })
                            .map(|word| word.number),
                    );
                }
                messages.extend(self.world.store.rows.values().filter_map(|record| match record {
                    Record::Core(core::Record::Tasks(tasks::Stored::Activity(line))) if line.task == *target => {
                        match &line.line {
                            tasks::Line::Words { message, by, words, .. }
                                if *by == tasks::Party::Task(*source) && words.starts_with(expected.as_bytes()) =>
                            {
                                Some(*message)
                            }
                            _ => None,
                        }
                    }
                    _ => None,
                }));
                let count = messages.len();
                if count != 1 {
                    return Err(format!(
                        "task {target}: expected one {expected:?} notice from {source}, found {count}"
                    ));
                }
            }
        }
        Ok(())
    }
}

fn check_request(outcome: &people::Outcome, operation: &str) -> Result<(), String> {
    if matches!(outcome, people::Outcome::Refused(_)) { Err(format!("{operation}: {outcome:?}")) } else { Ok(()) }
}

fn person_pattern(pattern: &tasks::Pattern) -> people::Pattern {
    people::Pattern {
        segments: pattern.segments.clone(),
        last: match &pattern.last {
            tasks::Last::Open(bytes) => people::Last::Open(bytes.clone()),
            tasks::Last::Exact(bytes) => people::Last::Exact(bytes.clone()),
        },
    }
}

pub(super) fn person_authority(authority: &tasks::Authority) -> people::Authority {
    people::Authority {
        tools: authority.tools.0,
        models: Some(people::Models::All),
        grants: authority
            .grants
            .iter()
            .map(|grant| people::Grant {
                connector: grant.connector,
                kind: grant.kind,
                pattern: person_pattern(&grant.pattern),
            })
            .collect(),
        delegation: people::Delegation {
            kinds: authority
                .delegation
                .kinds
                .iter()
                .map(|kind| match kind {
                    tasks::AuthorityExecutor::Charter(n) => people::Executor::Charter(*n),
                    tasks::AuthorityExecutor::Procedure(n) => people::Executor::Procedure(*n),
                    tasks::AuthorityExecutor::Role(n) => people::Executor::Role(*n),
                    tasks::AuthorityExecutor::Job(n) => people::Executor::Job(*n),
                })
                .collect(),
            tasks: authority.delegation.tasks,
            depth: authority.delegation.depth,
        },
        spend: authority.budget.spend,
        deadline: authority.budget.deadline,
        notes: authority.notes.0,
        note_resources: authority
            .note_resources
            .iter()
            .map(|scope| people::ResourceScope { connector: scope.connector, pattern: person_pattern(&scope.pattern) })
            .collect(),
    }
}
