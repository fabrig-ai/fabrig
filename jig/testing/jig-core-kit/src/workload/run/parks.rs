//! Requester questions and park faults (`jig/docs/design/domain/testing-workloads.md`, 3 and 4).
use super::{
    Bound, Driver, Kind, Struck, Sweep, Token, Workload, World, answer, core, ended, requests, root, row, stuck, tasks,
};
use crate::parks as faults;
use skein_lib::{ReplyTo, Rng};
use skein_world::streams::{Stream, Streams};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    Drawn,
    Parking,
    Parked,
    Ending,
    Answering,
    Done,
    Omitted,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Park {
    pub(super) target: usize,
    kind: Kind,
    after: u64,
    answer_after: u64,
    seen: Option<u64>,
    stage: Stage,
    question: u64,
    attempt: u64,
    reply: Option<core::CallKey>,
}

pub(super) fn draw(
    workload: &Workload,
    streams: Streams,
    configured: &[Kind],
    limits: &tasks::Limits,
) -> (Vec<Park>, Option<stuck::Draw>) {
    let mut rng = streams.rng(Stream::Faults, "parks");
    let parks = answer_parks(workload, &mut rng, configured);
    let stuck = stuck::draw(workload, &mut rng, configured, limits, &parks);
    (parks, stuck)
}

fn answer_parks(workload: &Workload, rng: &mut Rng, configured: &[Kind]) -> Vec<Park> {
    let mut parks = Vec::new();
    if !configured.contains(&faults::PARKS_WITH_A_QUESTION_OPEN) {
        return parks;
    }
    for (target, task) in workload.tasks.iter().enumerate() {
        if !task.asks_requester {
            continue;
        }
        let parent = task.parent.expect("delegate question");
        let kinds: Vec<_> = [
            faults::ANSWERED_WHILE_PARKED,
            faults::ANSWERED_AFTER_ITS_END,
            faults::ANSWERED_TO_ITS_SUCCESSOR,
            faults::COORDINATOR_ENDS_WHILE_A_CHILD_WAITS,
        ]
        .into_iter()
        .filter(|kind| {
            configured.contains(kind)
                && (*kind != faults::COORDINATOR_ENDS_WHILE_A_CHILD_WAITS || workload.tasks[parent].parent.is_none())
        })
        .collect();
        let chosen = rng.below(2) == 0;
        let choice = rng.next_u64();
        let after = rng.below(8);
        let answer_after = rng.below(8);
        // A coordinator's end must not cancel another selected park in its tree.
        let goal = task.name.split('.').next().expect("task place");
        if chosen
            && !kinds.is_empty()
            && !parks.iter().any(|park: &Park| workload.tasks[park.target].name.split('.').next() == Some(goal))
        {
            let choice = usize::try_from(choice % u64::try_from(kinds.len()).expect("four kinds")).expect("kind index");
            parks.push(Park {
                target,
                kind: kinds[choice],
                after,
                answer_after,
                seen: None,
                stage: Stage::Drawn,
                question: 0,
                attempt: 0,
                reply: None,
            });
        }
    }
    parks
}

impl Driver {
    pub(super) fn record_park(&mut self, world: &mut World, target: usize, kind: Kind) -> Result<(), String> {
        let now = world.clocks().0;
        let ledger = world.fault_ledger.as_mut().expect("workload ledger");
        let entry = ledger.inject(
            kind,
            now,
            Struck { request: Some(self.workload.tasks[target].name.clone()), ..Struck::default() },
        );
        ledger.stop(entry, now)?;
        let mut affected = Some(target);
        while let Some(index) = affected {
            self.progress[index].last = (now, world.iterations());
            affected = self.workload.tasks[index].parent;
        }
        Ok(())
    }

    pub(super) fn message(
        &mut self,
        world: &mut World,
        source: u64,
        target: u64,
        kind: tasks::MessageKind,
        words: &[u8],
    ) -> core::CallKey {
        self.park_action(world, source, core::NamedAction::Message { target, kind, words: words.into() })
    }

    pub(super) fn park_action(&mut self, world: &mut World, source: u64, action: core::NamedAction) -> core::CallKey {
        self.completion += 1;
        let key = core::CallKey {
            task: source,
            attempt: row(world, source).expect("action's sender").attempt,
            completion: self.completion,
            position: 0,
        };
        world.enqueue(root::Event::Core(core::Event::NamedAction {
            to: ReplyTo::new(Token::new(20_000 + u64::from(self.completion))),
            key,
            action,
        }));
        key
    }

    pub(super) fn answer_children(&mut self, world: &mut World, index: usize) -> bool {
        for child in 0..self.workload.tasks.len() {
            if self.workload.tasks[child].parent != Some(index)
                || self
                    .parks
                    .iter()
                    .any(|park| park.target == child && !matches!(park.stage, Stage::Done | Stage::Omitted))
            {
                continue;
            }
            let Some(number) = self.progress[child].number else { continue };
            let Some(current) = row(world, number) else { continue };
            if let Some(question) = current.questions.first() {
                let parent = self.progress[index].number.expect("question's requester");
                self.message(
                    world,
                    parent,
                    number,
                    tasks::MessageKind::Answer { question: question.number },
                    b"proceed",
                );
                return true;
            }
        }
        false
    }

    pub(super) fn parks_block(&self, index: usize, world: &World) -> bool {
        self.parks.iter().any(|park| {
            (park.target == index || self.workload.tasks[park.target].parent == Some(index))
                && !matches!(park.stage, Stage::Done | Stage::Omitted)
                && self.progress[park.target]
                    .number
                    .and_then(|number| row(world, number))
                    .is_some_and(|row| !row.questions.is_empty())
        })
    }

    pub(super) fn parks_settled(&self, world: &World, bound: Bound) -> Result<bool, String> {
        let mut settled = true;
        for park in &self.parks {
            if matches!(park.stage, Stage::Done | Stage::Omitted)
                || park.stage == Stage::Drawn
                    && (self.progress[park.target].pruned
                        || self.progress[park.target].number.is_some_and(|number| ended(world, number)))
            {
                continue;
            }
            let last = self.progress[park.target].last;
            if bound.spent((world.clocks().0, world.iterations()), last) {
                return Err(self.park_finding(park, &format!("park past its bound in {:?}", park.stage)));
            }
            settled = false;
        }
        Ok(settled)
    }

    fn begin_park(&mut self, world: &mut World, sweep: &Sweep, index: usize) -> bool {
        let park = self.parks[index].clone();
        let number = self.progress[park.target].number.expect("park target exists");
        let parent_index = self.workload.tasks[park.target].parent.expect("delegate");
        let parent = self.progress[parent_index].number.expect("requester");
        let current = row(world, number).expect("park target");
        let Some(question) = current.questions.first() else { return false };
        if !running(current) || self.script_target.is_some_and(|t| self.related(t, park.target)) {
            return false;
        }
        let since = *self.parks[index].seen.get_or_insert(world.iterations());
        if world.iterations().saturating_sub(since) < park.after {
            return false;
        }
        let watching = row(world, parent)
            .expect("requester")
            .subscriptions
            .iter()
            .any(|s| matches!(s.kind, tasks::SubscriptionKind::Task { target, .. } if target == number));
        let holding = if watching { vec!["its requester watches it"] } else { Vec::new() };
        if let Some(exclusion) = sweep.leaves_out(faults::PARKS_WITH_A_QUESTION_OPEN, &holding) {
            let now = world.clocks().0;
            world.fault_ledger.as_mut().expect("workload ledger").leave_out(
                exclusion.scenario,
                faults::PARKS_WITH_A_QUESTION_OPEN,
                now,
            );
            self.parks[index].stage = Stage::Omitted;
            return false;
        }
        self.parks[index].question = question.number;
        self.parks[index].attempt = current.attempt;
        self.parks[index].stage = Stage::Parking;
        world.enqueue(answer(number, current.attempt, tasks::End::Parked));
        true
    }

    pub(super) fn drive_parks(&mut self, world: &mut World, sweep: &Sweep) -> Result<(), String> {
        for index in 0..self.parks.len() {
            let park = self.parks[index].clone();
            let Some(number) = self.progress[park.target].number else { continue };
            let parent_index = self.workload.tasks[park.target].parent.expect("parked delegate");
            let parent = self.progress[parent_index].number.expect("delegate's requester");
            let Some(current) = row(world, number) else { continue };
            match park.stage {
                Stage::Drawn => {
                    if self.begin_park(world, sweep, index) {
                        return Ok(());
                    }
                }
                Stage::Parking => {
                    if current.last_answer != Some(park.attempt) {
                        continue;
                    }
                    if current.phase != tasks::Phase::Active(tasks::Active::Idle) || current.questions.is_empty() {
                        return Err(self.park_finding(&park, "park did not retain its open question"));
                    }
                    self.record_park(world, park.target, faults::PARKS_WITH_A_QUESTION_OPEN)?;
                    self.parks[index].stage = Stage::Parked;
                    self.parks[index].seen = Some(world.iterations());
                }
                Stage::Parked => {
                    if world.iterations().saturating_sub(park.seen.expect("park's moment")) < park.answer_after {
                        continue;
                    }
                    if !row(world, parent).is_some_and(running) {
                        continue;
                    }
                    if park.kind == faults::ANSWERED_AFTER_ITS_END {
                        self.ask(
                            world,
                            self.workload.tasks[park.target].party,
                            requests::Ask::Cancel {
                                project: 1,
                                task: number,
                                reason: b"end before answer".as_slice().into(),
                                grace: None,
                            },
                        )?;
                        self.parks[index].stage = Stage::Ending;
                    } else if park.kind == faults::COORDINATOR_ENDS_WHILE_A_CHILD_WAITS {
                        self.record_park(world, parent_index, park.kind)?;
                        let attempt = row(world, parent).expect("requester").attempt;
                        world.enqueue(answer(
                            parent,
                            attempt,
                            tasks::End::Finished {
                                result: tasks::TaskResult::Report { words: b"coordinator ends".as_slice().into() },
                                cancel_delegates: true,
                            },
                        ));
                        self.parks[index].stage = Stage::Ending;
                    } else {
                        if current.phase != tasks::Phase::Active(tasks::Active::Idle) {
                            return Err(self.park_finding(&park, "answer did not arrive while parked"));
                        }
                        self.send_park_answer(world, index, parent, number)?;
                    }
                    return Ok(());
                }
                Stage::Ending => {
                    if !ended(world, number) {
                        continue;
                    }
                    if park.kind == faults::COORDINATOR_ENDS_WHILE_A_CHILD_WAITS {
                        if ended(world, parent) {
                            self.parks[index].stage = Stage::Done;
                        }
                    } else if row(world, parent).is_some_and(running) {
                        self.send_park_answer(world, index, parent, number)?;
                        return Ok(());
                    }
                }
                Stage::Answering => {
                    if self.park_answered(world, &park)? {
                        self.parks[index].stage = Stage::Done;
                    }
                }
                Stage::Done | Stage::Omitted => {}
            }
        }
        Ok(())
    }

    fn park_answered(&self, world: &World, park: &Park) -> Result<bool, String> {
        let replies: Vec<_> = world.answers.iter().filter(|(_, key, _)| Some(*key) == park.reply).collect();
        if replies.is_empty() {
            return Ok(false);
        }
        let valid = if park.kind == faults::ANSWERED_AFTER_ITS_END {
            match &replies[0].2 {
                core::CallPart::MessageRefused(problem) => problem.why == tasks::Refusal::Unknown,
                core::CallPart::ToolDenied { answer, findings } => {
                    *answer == jig_domain_authority::Answer::Refuse
                        && findings.as_ref() == [jig_domain_authority::Finding::Reference]
                }
                _ => false,
            }
        } else {
            matches!(replies[0].2, core::CallPart::Sent { .. })
        };
        if replies.len() != 1 || !valid {
            return Err(self.park_finding(park, &format!("answer's terminal: {replies:?}")));
        }
        Ok(true)
    }

    fn send_park_answer(&mut self, world: &mut World, index: usize, parent: u64, number: u64) -> Result<(), String> {
        let park = self.parks[index].clone();
        self.record_park(world, park.target, park.kind)?;
        self.parks[index].reply = Some(self.message(
            world,
            parent,
            number,
            tasks::MessageKind::Answer { question: park.question },
            b"proceed after park",
        ));
        self.parks[index].stage = Stage::Answering;
        Ok(())
    }

    fn park_finding(&self, park: &Park, why: &str) -> String {
        format!(
            "seed {}: park {:?}, task {} ({:?}), question {}: {why}",
            self.seed,
            park.kind,
            self.workload.tasks[park.target].name,
            self.progress[park.target].number,
            park.question
        )
    }

    pub(super) fn check_parks(&self, world: &World) -> Result<(), String> {
        for park in self.parks.iter().filter(|park| park.stage == Stage::Done) {
            let number = self.progress[park.target].number.expect("parked task");
            let answers: Vec<_> = world
                .store
                .rows
                .values()
                .filter_map(|record| match record {
                    root::Record::Core(core::Record::Tasks(tasks::Stored::Activity(line))) if line.task == number => {
                        match &line.line {
                            tasks::Line::Words { message, kind, .. }
                                if *kind == (tasks::MessageKind::Answer { question: park.question }) =>
                            {
                                Some(*message)
                            }
                            _ => None,
                        }
                    }
                    _ => None,
                })
                .collect();
            let expects_answer =
                park.kind == faults::ANSWERED_WHILE_PARKED || park.kind == faults::ANSWERED_TO_ITS_SUCCESSOR;
            if answers.len() != usize::from(expects_answer) {
                return Err(self.park_finding(park, &format!("{} answers heard", answers.len())));
            }
            if let Some(message) = answers.first() {
                let reads: Vec<_> =
                    self.reads.iter().filter(|(task, _, word)| *task == number && word == message).collect();
                if reads.len() != 1 || reads[0].1 <= park.attempt {
                    return Err(
                        self.park_finding(park, &format!("answer must be read once, only by the successor: {reads:?}"))
                    );
                }
                let recipients: Vec<_> =
                    world
                        .assignment_transcripts
                        .iter()
                        .filter(|assigned| {
                            assigned.task == number && assigned.messages.iter().any(|word| word.number == *message)
                        })
                        .map(|assigned| assigned.attempt)
                        .chain(world.inbound_runs.iter().filter_map(|(task, attempt, word)| {
                            (*task == number && word == message).then_some(*attempt)
                        }))
                        .collect();
                if recipients != [reads[0].1] {
                    return Err(
                        self.park_finding(park, &format!("answer offered to {recipients:?}, expected only its reader"))
                    );
                }
            }
            if park.kind == faults::COORDINATOR_ENDS_WHILE_A_CHILD_WAITS && !ended(world, number) {
                return Err(self.park_finding(park, "coordinator left its child waiting"));
            }
            if let Some(key) = park.reply {
                let count = world.answers.iter().filter(|(_, actual, _)| *actual == key).count();
                if count != 1 {
                    return Err(self.park_finding(park, "answer sender must be told exactly once"));
                }
            }
        }
        Ok(())
    }
}

pub(super) fn running(row: &tasks::TaskRecord) -> bool {
    matches!(row.phase, tasks::Phase::Active(tasks::Active::Running { .. }))
}

#[cfg(test)]
mod tests {
    use super::{Workload, answer_parks, draw, faults, stuck};
    use crate::workload::Party;
    use skein_world::streams::{Stream, Streams};

    #[test]
    fn stuck_draws_respect_the_planning_profile() {
        let configured = [
            faults::PARKS_WITH_A_QUESTION_OPEN,
            faults::ANSWERED_WHILE_PARKED,
            faults::ANSWERED_AFTER_ITS_END,
            faults::ANSWERED_TO_ITS_SUCCESSOR,
            faults::COORDINATOR_ENDS_WHILE_A_CHILD_WAITS,
            faults::STUCK_IN_A_CIRCLE,
            faults::PARKS_ON_NOTHING,
            faults::WATCHES_A_TASK_THAT_CAN_ANSWER,
        ];
        let (_, limits) = crate::faults::plan_fixture(0);
        let mut circle_sizes = std::collections::BTreeSet::new();
        let mut control = false;
        let mut nothing = false;
        for seed in 0..128 {
            let streams = Streams::new(seed);
            let workload = Workload::draw(streams);
            let (parks, shape) = draw(&workload, streams, &configured, &limits.core.tasks);
            let Some(shape) = shape else { continue };
            let _ = streams.rng(Stream::Parties, "owner").next_u64();
            assert_eq!(Some(&shape), draw(&workload, streams, &configured, &limits.core.tasks).1.as_ref());
            assert!(parks.iter().all(|park| shape.expect(park.target).is_none()), "seed {seed}");
            match shape {
                stuck::Draw::Circle(members) => {
                    circle_sizes.insert(members.len());
                }
                stuck::Draw::Nothing(_) => nothing = true,
                stuck::Draw::Watch { .. } => control = true,
            }
        }
        assert_eq!(circle_sizes.into_iter().collect::<Vec<_>>(), [1, 2]);
        assert!(control && nothing);
    }

    #[test]
    fn park_draws_use_only_the_faults_parks_stream() {
        let configured = [
            faults::PARKS_WITH_A_QUESTION_OPEN,
            faults::ANSWERED_WHILE_PARKED,
            faults::ANSWERED_AFTER_ITS_END,
            faults::ANSWERED_TO_ITS_SUCCESSOR,
            faults::COORDINATOR_ENDS_WHILE_A_CHILD_WAITS,
        ];
        let streams = Streams::new(49);
        let workload = Workload::draw(streams);
        let expected = answer_parks(&workload, &mut streams.rng(Stream::Faults, "parks"), &configured);
        let first = &expected[0];
        assert_eq!(workload.tasks[first.target].name, "t0.0");
        assert_eq!(first.kind, faults::COORDINATOR_ENDS_WHILE_A_CHILD_WAITS);
        assert_eq!((first.after, first.answer_after), (1, 5), "Faults/parks stream");
        assert_eq!(expected.len(), 2, "independent targets in different goals");
        let _ = streams.rng(Stream::Parties, "owner").next_u64();
        let _ = workload.acts(streams, Party::Member, &[crate::people::HOLDS_A_TASK]);
        assert_eq!(expected, answer_parks(&workload, &mut streams.rng(Stream::Faults, "parks"), &configured));
        assert_eq!(workload, Workload::draw(streams));
        assert!(expected.iter().all(|park| workload.tasks[park.target].asks_requester));
    }
}
