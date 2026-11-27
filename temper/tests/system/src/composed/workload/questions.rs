//! The producer's person task is answered by that goal's party, at once.
use super::{Party, Token, World, engine, people, tasks};

impl World {
    pub(super) fn prepare_workload_questions(&mut self) {
        let mut amendments = Vec::new();
        let story = self.workload.as_ref().expect("workload");
        for goal in &story.workload.goals {
            if let Some(task) = story.names.task(&goal.name)
                && (goal.train || self.waits_for_branch_room(task))
                && let Some(row) = self.root.tasks().task(task)
                && !story.widened.contains(&task)
            {
                let mut authority = person_authority(row);
                let mut kinds = authority.delegation.kinds.into_vec();
                if !kinds.contains(&people::Executor::Role(0)) {
                    kinds.push(people::Executor::Role(0));
                }
                authority.delegation.kinds = kinds.into();
                amendments.push((goal.party, task, row.revisions.spec, authority));
            }
            for change in &goal.changes {
                let Some(task) = story
                    .names
                    .task(&change.name)
                    .and_then(|task| self.store.change_hot(task))
                    .and_then(|row| row.delegate.map(|(task, _)| task))
                else {
                    continue;
                };
                let Some(row) = self.root.tasks().task(task) else { continue };
                if (!change.asks && !self.workload_parks(task))
                    || story.widened.contains(&task)
                    || row.authority.delegation.tasks != 0
                {
                    continue;
                }
                let mut authority = person_authority(row);
                authority.delegation =
                    people::Delegation { kinds: Box::new([people::Executor::Role(0)]), tasks: 1, depth: 1 };
                amendments.push((goal.party, task, row.revisions.spec, authority));
            }
        }
        for (party, task, revision, authority) in amendments {
            self.workload.as_mut().expect("workload").widened.insert(task);
            self.workload_enqueue(
                party,
                people::Ask::Amend {
                    project: 1,
                    task,
                    revision,
                    release: None,
                    amendment: people::Amendment {
                        tracked: None,
                        addressed: None,
                        instructions: None,
                        spec: None,
                        wake: None,
                        dependencies: None,
                        authority: Some(authority),
                        llm: None,
                        reason: b"Permit the workload's one question to its person.".as_slice().into(),
                    },
                },
            );
        }
    }

    pub(super) fn answer_workload_questions(&mut self, tasks: &[tasks::ViewTask]) {
        let mut answers = Vec::new();
        let story = self.workload.as_ref().expect("workload");
        for view in tasks {
            let row = self.root.tasks().task(view.number).expect("view task");
            if !matches!(row.executor, tasks::Executor::Person(_)) || story.answered.contains(&row.number) {
                continue;
            }
            for goal in &story.workload.goals {
                for change in &goal.changes {
                    if row.spec.words.starts_with(format!("@ask:{}@", change.name).as_bytes()) {
                        answers.push((goal.party, row.number, row.revisions.spec));
                    }
                }
            }
        }
        for (party, task, revision) in answers {
            self.workload.as_mut().expect("workload").answered.insert(task);
            self.workload_enqueue(
                party,
                people::Ask::AnswerPerson {
                    project: 1,
                    task,
                    revision,
                    result: people::PersonResult::Report { words: b"Yes, deliver the named files.".as_slice().into() },
                },
            );
        }
    }

    pub(super) fn link_workload_changes(&mut self) {
        let mut amendments = Vec::new();
        let story = self.workload.as_ref().expect("workload");
        for goal in &story.workload.goals {
            for change in &goal.changes {
                let Some(task) = story.names.task(&change.name) else { continue };
                if story.linked.contains(&task) {
                    continue;
                }
                let cross: Vec<_> = change
                    .after
                    .iter()
                    .filter(|name| !goal.changes.iter().any(|sibling| sibling.name == **name))
                    .map(|name| story.names.task(name).expect("previous goal's change"))
                    .collect();
                if cross.is_empty() {
                    continue;
                }
                let row = self.root.tasks().task(task).expect("new change");
                let mut dependencies = row.dependencies.to_vec();
                dependencies.extend(cross);
                amendments.push((goal.party, task, row.revisions.spec, dependencies));
            }
        }
        for (party, task, revision, dependencies) in amendments {
            self.workload.as_mut().expect("workload").linked.insert(task);
            self.workload_enqueue(
                party,
                people::Ask::Amend {
                    project: 1,
                    task,
                    revision,
                    release: None,
                    amendment: people::Amendment {
                        tracked: None,
                        addressed: None,
                        instructions: None,
                        spec: None,
                        wake: None,
                        dependencies: Some(dependencies.into()),
                        authority: None,
                        llm: None,
                        reason: b"Keep the workload's already-landed cross-goal dependency.".as_slice().into(),
                    },
                },
            );
        }
    }

    pub(super) fn latch_workload(&mut self, tasks: &[tasks::ViewTask]) {
        let story = self.workload.as_ref().expect("workload");
        if story.latch.is_some_and(|latch| story.answered.contains(&latch)) {
            return;
        }
        let goal = self.workload().goals.iter().find(|goal| goal.train).expect("train");
        let marker = format!("@latch:{}@", goal.name);
        let latch = tasks.iter().find_map(|view| {
            let row = self.root.tasks().task(view.number)?;
            row.spec.words.starts_with(marker.as_bytes()).then_some((row.number, row.revisions.spec))
        });
        let Some((latch, revision)) = latch else { return };
        let goal = goal.clone();
        self.workload.as_mut().expect("workload").latch = Some(latch);
        for change in &goal.changes {
            let Some(task) = self.workload_names().task(&change.name) else { continue };
            if self.workload.as_ref().expect("workload").latched.contains(&task) {
                continue;
            }
            let row = self.root.tasks().task(task).expect("live train member");
            let revision = row.revisions.spec;
            let mut dependencies = row.dependencies.to_vec();
            dependencies.push(latch);
            self.workload.as_mut().expect("workload").latched.insert(task);
            self.workload_enqueue(
                goal.party,
                people::Ask::Amend {
                    project: 1,
                    task,
                    revision,
                    release: None,
                    amendment: people::Amendment {
                        tracked: None,
                        addressed: None,
                        instructions: None,
                        spec: None,
                        wake: None,
                        dependencies: Some(dependencies.into()),
                        authority: None,
                        llm: None,
                        reason: b"Make the train's readiness wait on its person latch.".as_slice().into(),
                    },
                },
            );
        }
        let ready = goal.changes.iter().all(|change| {
            self.workload_names()
                .task(&change.name)
                .and_then(|task| self.store.change_hot(task).map(|row| (task, row)))
                .is_some_and(|(task, row)| {
                    matches!(row.change.state, temper_engine_domain_forge_change::State::Gating { .. })
                        && self.root.tasks().procedure_due(task).is_none()
                })
        });
        if ready && self.workload.as_mut().expect("workload").answered.insert(latch) {
            self.workload_enqueue(
                goal.party,
                people::Ask::AnswerPerson {
                    project: 1,
                    task: latch,
                    revision,
                    result: people::PersonResult::Report {
                        words: b"The train's members are ready together.".as_slice().into(),
                    },
                },
            );
        }
    }

    pub(super) fn workload_enqueue(&mut self, party: Party, ask: people::Ask) {
        let story = self.workload.as_mut().expect("workload");
        let number = story.next_reply;
        story.next_reply += 1;
        story.pending_asks.insert(number);
        let sign_in = match party {
            Party::First => self.party.sign_in,
            Party::Second => story.other.sign_in,
        }
        .expect("party");
        let mut key = [0; 16];
        key[..8].copy_from_slice(&number.to_le_bytes());
        self.root_events.push_back(engine::Event::Party(engine::PartyInput::Ask {
            models: Box::default(),
            reply_to: skein_lib::ReplyTo::new(Token::new(number)),
            sign_in,
            key,
            ask,
        }));
    }
}

fn person_authority(row: &tasks::TaskRecord) -> people::Authority {
    people::Authority {
        tools: row.authority.tools.0,
        models: None,
        grants: row
            .authority
            .grants
            .iter()
            .map(|grant| people::Grant {
                connector: grant.connector,
                kind: grant.kind,
                pattern: people::Pattern {
                    segments: grant.pattern.segments.clone(),
                    last: match &grant.pattern.last {
                        tasks::Last::Exact(v) => people::Last::Exact(v.clone()),
                        tasks::Last::Open(v) => people::Last::Open(v.clone()),
                    },
                },
            })
            .collect(),
        delegation: people::Delegation {
            kinds: row
                .authority
                .delegation
                .kinds
                .iter()
                .map(|kind| match kind {
                    tasks::AuthorityExecutor::Job(number) => people::Executor::Job(*number),
                    tasks::AuthorityExecutor::Charter(number) => people::Executor::Charter(*number),
                    tasks::AuthorityExecutor::Procedure(number) => people::Executor::Procedure(*number),
                    tasks::AuthorityExecutor::Role(number) => people::Executor::Role(*number),
                })
                .collect(),
            tasks: row.authority.delegation.tasks,
            depth: row.authority.delegation.depth,
        },
        spend: row.numbers.budget,
        deadline: None,
        notes: row.authority.notes.0,
        note_resources: Box::default(),
    }
}
