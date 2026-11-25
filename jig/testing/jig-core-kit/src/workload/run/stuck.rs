//! Stuck cases in the parks workload (`jig/docs/design/domain/testing-workloads.md`, 4).
use super::{Bound, Driver, Expect, Kind, Sweep, TaskKind, Workload, World, answer, core, ended, parks, row, tasks};
use crate::parks as faults;
use skein_lib::Rng;
use std::fmt::Write as _;

const QUESTION: &str = "Which next?";
const IDLE: tasks::Phase = tasks::Phase::Active(tasks::Active::Idle);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Pair {
    goal: usize,
    child: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Draw {
    Circle(Vec<Pair>),
    Nothing(Pair),
    Watch { pair: Pair, worker: usize, notice: Option<u64>, answer_notice: Option<u64> },
}

pub(super) fn draw(
    workload: &Workload,
    rng: &mut Rng,
    configured: &[Kind],
    limits: &tasks::Limits,
    parks: &[parks::Park],
) -> Option<Draw> {
    let kinds = [faults::STUCK_IN_A_CIRCLE, faults::PARKS_ON_NOTHING, faults::WATCHES_A_TASK_THAT_CAN_ANSWER];
    if !kinds.iter().any(|kind| configured.contains(kind)) {
        return None;
    }
    let (mut singles, mut doubles) = (Vec::new(), Vec::new());
    for (goal, _) in workload.tasks.iter().enumerate().filter(|(_, task)| task.parent.is_none() && !task.short) {
        let children: Vec<_> = workload.tasks[goal + 1..]
            .iter()
            .enumerate()
            .take_while(|(_, child)| child.parent.is_some())
            .map(|(offset, child)| (goal + 1 + offset, child))
            .collect();
        // One script per tree. The live-wake and circle cases need no extra edges from other delegates.
        if children.iter().any(|(index, child)| {
            child.parent != Some(goal)
                || !matches!(child.kind, TaskKind::Run { .. })
                || parks.iter().any(|park| park.target == *index)
        }) {
            continue;
        }
        match children.as_slice() {
            [(child, _)] => singles.push(Pair { goal, child: *child }),
            [(child, _), (worker, _)] => doubles.push((Pair { goal, child: *child }, *worker)),
            _ => {}
        }
    }
    // A seed sets up one of these cases without changing the other parks draws.
    let choices: Vec<_> = kinds
        .into_iter()
        .filter(|kind| {
            configured.contains(kind)
                && if *kind == faults::WATCHES_A_TASK_THAT_CAN_ANSWER {
                    !doubles.is_empty()
                } else {
                    !singles.is_empty()
                }
        })
        .collect();
    if choices.is_empty() {
        return None;
    }
    let kind = choices[pick(rng, choices.len())];
    Some(if kind == faults::STUCK_IN_A_CIRCLE {
        // Draw one to three, then cap at eligible goals and the profile
        // (jig/docs/design/domain/testing-workloads.md, 4). Each pair costs
        // two live tasks and two vertices of the walk.
        // At the planning fixture's five tasks this admits at most two pairs.
        let pairs = (limits.tasks.min(limits.project_tasks).min(limits.tree_tasks) / 2) as usize;
        let len = (1 + pick(rng, 3)).min(singles.len()).min(pairs);
        let members = (0..len)
            .map(|_| {
                let index = pick(rng, singles.len());
                singles.swap_remove(index)
            })
            .collect();
        Draw::Circle(members)
    } else if kind == faults::PARKS_ON_NOTHING {
        Draw::Nothing(singles[pick(rng, singles.len())])
    } else {
        let (pair, worker) = doubles[pick(rng, doubles.len())];
        Draw::Watch { pair, worker, notice: None, answer_notice: None }
    })
}

fn pick(rng: &mut Rng, len: usize) -> usize {
    usize::try_from(rng.below(len as u64)).expect("bounded draw")
}

impl Draw {
    pub(super) fn watches(&self) -> bool {
        !matches!(self, Self::Nothing(_))
    }

    pub(super) fn goal(&self, index: usize) -> bool {
        match self {
            Self::Circle(members) => members.iter().any(|pair| pair.goal == index),
            Self::Nothing(pair) | Self::Watch { pair, .. } => pair.goal == index,
        }
    }

    pub(super) fn expect(&self, index: usize) -> Option<Expect> {
        match self {
            Self::Circle(members) => {
                let at = members.iter().position(|pair| pair.goal == index || pair.child == index)?;
                Some(if at + 1 == members.len() && members[at].goal == index { Expect::Stuck } else { Expect::Waits })
            }
            Self::Nothing(pair) if pair.child == index => Some(Expect::Stuck),
            Self::Nothing(pair) if pair.goal == index => Some(Expect::Waits),
            Self::Watch { pair, worker, .. } if [pair.goal, pair.child, *worker].contains(&index) => Some(Expect::Ends),
            Self::Nothing(_) | Self::Watch { .. } => None,
        }
    }
}

impl Driver {
    pub(super) fn can_start_stuck_goal(&self, world: &World, index: usize) -> bool {
        let Some(draw) = &self.stuck else { return true };
        // Leave room in the same world for every arm of a circle to be live together.
        !draw.goal(index)
            || self.workload.tasks.iter().enumerate().all(|(other, task)| {
                task.parent.is_some()
                    || draw.goal(other)
                    || self.progress[other].number.is_some_and(|n| ended(world, n))
            })
    }

    pub(super) fn script_stuck(&mut self, world: &mut World, sweep: &Sweep) -> Result<bool, String> {
        let Some(mut draw) = self.stuck.take() else { return Ok(false) };
        let step = match &mut draw {
            Draw::Circle(members) => self.circle_step(world, sweep, members),
            Draw::Nothing(pair) => self.nothing_step(world, sweep, *pair),
            Draw::Watch { pair, worker, notice, answer_notice } => {
                self.watch_step(world, sweep, *pair, *worker, notice, answer_notice)
            }
        };
        self.stuck = Some(draw);
        step
    }

    fn nothing_step(&mut self, world: &mut World, sweep: &Sweep, pair: Pair) -> Result<bool, String> {
        if self.create(world, pair.child, sweep)? {
            return Ok(true);
        }
        if self.progress[pair.child].number.is_none() {
            return Ok(false);
        }
        let child = self.stuck_row(world, pair.child).expect("delegate admitted");
        // The delegate's hold can wake its requester to decide. Read that
        // notice and park again without deciding the hold.
        if !self.progress[pair.goal].parked || matches!(child.phase, tasks::Phase::Held { .. }) {
            if self.read_stuck_run(world, pair.goal)? {
                return Ok(true);
            }
            let goal = self.stuck_row(world, pair.goal).expect("child's goal");
            if parks::running(goal) {
                self.park_stuck(pair.goal, (goal.number, goal.attempt), world);
                return Ok(true);
            }
        }
        if self.read_stuck_run(world, pair.child)? {
            return Ok(true);
        }
        let child = self.stuck_row(world, pair.child).expect("delegate admitted");
        if parks::running(child) {
            let run = (child.number, child.attempt);
            if !self.progress[pair.child].parked {
                let limit = world.env().limits.core.tasks.retries.run.retries;
                self.progress[pair.child].stuck_words =
                    Some(format!("Its runs parked with nothing left to await, past its {limit} run tries."));
                self.record_park(world, pair.child, faults::PARKS_ON_NOTHING)?;
            }
            self.park_stuck(pair.child, run, world);
            return Ok(true);
        }
        Ok(false)
    }

    fn circle_step(&mut self, world: &mut World, sweep: &Sweep, members: &[Pair]) -> Result<bool, String> {
        if members.iter().any(|pair| self.progress[pair.goal].number.is_none()) {
            return Ok(false);
        }
        for pair in members {
            if self.create(world, pair.child, sweep)? {
                return Ok(true);
            }
        }
        // Consume the question before reserving a watch's inbox slot. The
        // read frees inbox room without answering the question.
        for pair in members {
            if self.read_stuck_run(world, pair.goal)? {
                return Ok(true);
            }
        }
        for &pair in members {
            if self.progress[pair.child].number.is_none() {
                return Ok(false);
            }
            if self.ask_and_park(world, pair)? {
                return Ok(true);
            }
            if !self.progress[pair.child].parked {
                return Ok(false);
            }
        }
        for (pair, next) in members.iter().zip(members.iter().cycle().skip(1)) {
            let target = self.progress[next.child].number.expect("asking delegate");
            if self.install_watch(world, pair.goal, target)? {
                return Ok(true);
            }
        }
        let (last, before) = members.split_last().expect("the circle has at least one pair");
        for pair in members {
            let goal = self.stuck_row(world, pair.goal).expect("circle goal");
            if !self.progress[pair.goal].parked || pair == last && parks::running(goal) {
                if !parks::running(goal) {
                    return Err(self.stuck_finding(world, pair.goal, "the goal did not run to read its question"));
                }
                let run = (goal.number, goal.attempt);
                if pair == last && !self.progress[pair.goal].parked {
                    self.progress[pair.goal].stuck_words = Some(self.circle_words(world, *last, before)?);
                    self.record_park(world, pair.goal, faults::STUCK_IN_A_CIRCLE)?;
                    eprintln!("parks circle: {} goals", members.len());
                }
                self.park_stuck(pair.goal, run, world);
                return Ok(true);
            }
            if pair != last {
                self.idle_without_tries(world, pair.goal, goal)?;
            }
        }
        Ok(false)
    }

    fn watch_step(
        &mut self,
        world: &mut World,
        sweep: &Sweep,
        pair: Pair,
        worker: usize,
        notice: &mut Option<u64>,
        received: &mut Option<u64>,
    ) -> Result<bool, String> {
        if self.create(world, pair.child, sweep)? || self.create(world, worker, sweep)? {
            return Ok(true);
        }
        let Some(worker_number) = self.progress[worker].number else { return Ok(false) };
        if !self.progress[pair.goal].parked {
            if self.read_stuck_run(world, pair.goal)? || self.ask_and_park(world, pair)? {
                return Ok(true);
            }
            if !self.progress[pair.child].parked {
                return Ok(false);
            }
            if self.install_watch(world, pair.goal, worker_number)? {
                return Ok(true);
            }
            let goal = self.stuck_row(world, pair.goal).expect("watching goal");
            let run = (goal.number, goal.attempt);
            self.record_park(world, pair.goal, faults::WATCHES_A_TASK_THAT_CAN_ANSWER)?;
            self.park_stuck(pair.goal, run, world);
            return Ok(true);
        }
        let goal = self.stuck_row(world, pair.goal).expect("watching goal");
        if !ended(world, worker_number) {
            self.idle_without_tries(world, pair.goal, goal)?;
            if self.read_run(world, worker_number)? {
                return Ok(true);
            }
            let work = self.stuck_row(world, worker).expect("watched worker");
            if parks::running(work) {
                Self::finish_stuck((work.number, work.attempt), world);
                return Ok(true);
            }
            return Ok(false);
        }
        if notice.is_none() {
            *notice = goal.inbox.iter().find_map(|message| match message.kind {
                tasks::MessageKind::Notice { target, state: tasks::NoticeState::Done, .. }
                    if target == worker_number =>
                {
                    Some(message.number)
                }
                _ => None,
            });
            if notice.is_none() || goal.tries.run != 0 || goal.hold != 0 {
                return Err(self.stuck_finding(
                    world,
                    pair.goal,
                    "the worker did not wake the goal by its watch without spending a try",
                ));
            }
        }
        self.answer_after_watch(world, pair, notice.expect("watch notice"), received)
    }

    fn answer_after_watch(
        &mut self,
        world: &mut World,
        pair: Pair,
        notice: u64,
        received: &mut Option<u64>,
    ) -> Result<bool, String> {
        if self.read_stuck_run(world, pair.goal)? {
            return Ok(true);
        }
        let goal = self.stuck_row(world, pair.goal).expect("watching goal");
        let child = self.stuck_row(world, pair.child).expect("asking delegate");
        if let Some(question) = child.questions.first() {
            if parks::running(goal) && self.reads.contains(&(goal.number, goal.attempt, notice)) {
                self.message(
                    world,
                    goal.number,
                    child.number,
                    tasks::MessageKind::Answer { question: question.number },
                    b"proceed",
                );
                return Ok(true);
            }
            return Ok(false);
        }
        if !ended(world, child.number) {
            if received.is_none() {
                *received = child.inbox.iter().find_map(|message| {
                    matches!(message.kind, tasks::MessageKind::Answer { .. }).then_some(message.number)
                });
            }
            let run = (child.number, child.attempt);
            let running = parks::running(child);
            if self.read_run(world, run.0)? {
                return Ok(true);
            }
            if running && received.is_some_and(|message| self.reads.contains(&(run.0, run.1, message))) {
                Self::finish_stuck(run, world);
                return Ok(true);
            }
        } else if parks::running(goal) {
            Self::finish_stuck((goal.number, goal.attempt), world);
            return Ok(true);
        }
        Ok(false)
    }

    fn ask_and_park(&mut self, world: &mut World, pair: Pair) -> Result<bool, String> {
        if self.progress[pair.child].parked {
            return Ok(false);
        }
        if self.read_stuck_run(world, pair.child)? {
            return Ok(true);
        }
        let child = self.stuck_row(world, pair.child).expect("asking delegate admitted");
        if !parks::running(child) {
            return Ok(false);
        }
        if self.progress[pair.child].asked {
            self.stuck_question(world, pair.child, child)?;
            self.park_stuck(pair.child, (child.number, child.attempt), world);
        } else {
            let goal = self.progress[pair.goal].number.expect("delegate has a goal");
            self.message(world, child.number, goal, tasks::MessageKind::Question, QUESTION.as_bytes());
            self.progress[pair.child].asked = true;
        }
        Ok(true)
    }

    fn install_watch(&mut self, world: &mut World, goal: usize, target: u64) -> Result<bool, String> {
        let current = self.stuck_row(world, goal).expect("watching goal");
        let kind = tasks::SubscriptionKind::Task { target, held: false, result: true };
        if current.subscriptions.iter().any(|subscription| subscription.kind == kind) {
            return Ok(false);
        }
        if let Some(key) = self.progress[goal].watch_call {
            let reply = world.answers.iter().find(|(_, received, _)| *received == key);
            return Err(self.stuck_finding(world, goal, &format!(
                "could not watch task {target}: {reply:?}; delegates {:?}, subscriptions {:?}, questions {:?}, inbox lengths ({}, {})",
                current.delegates, current.subscriptions, current.question_lines, current.inbox.len(), current.unrecorded.len()
            )));
        }
        let number = current.number;
        self.progress[goal].watch_call = Some(self.park_action(world, number, core::NamedAction::Subscribe { kind }));
        Ok(true)
    }

    fn circle_words(&self, world: &World, last: Pair, before: &[Pair]) -> Result<String, String> {
        let asker = self.stuck_row(world, last.child).expect("circle's asker");
        let question = self.stuck_question(world, last.child, asker)?;
        let mut words = format!("Task {}'s question {question} is still open", asker.number);
        for (at, pair) in before.iter().enumerate() {
            let child = self.stuck_row(world, pair.child).expect("other asker");
            let goal = self.progress[pair.goal].number.expect("other goal");
            let question = self.stuck_question(world, pair.child, child)?;
            let separator = if at == 0 { ", through tasks " } else { ", " };
            write!(words, "{separator}{} (question {question}), {goal}", child.number).expect("writing a String");
        }
        words.push_str(": ");
        words.push_str(QUESTION);
        words.truncate(words.len().min(tasks::REASON_BYTES as usize));
        Ok(words)
    }

    fn read_stuck_run(&mut self, world: &mut World, index: usize) -> Result<bool, String> {
        match self.progress[index].number {
            Some(number) => self.read_run(world, number),
            None => Ok(false),
        }
    }

    fn stuck_row<'w>(&self, world: &'w World, index: usize) -> Option<&'w tasks::TaskRecord> {
        row(world, self.progress[index].number?)
    }

    fn park_stuck(&mut self, index: usize, (number, attempt): (u64, u64), world: &mut World) {
        self.progress[index].parked = true;
        world.enqueue(answer(number, attempt, tasks::End::Parked));
    }

    fn finish_stuck((number, attempt): (u64, u64), world: &mut World) {
        world.enqueue(answer(
            number,
            attempt,
            tasks::End::Finished {
                result: tasks::TaskResult::Report { words: b"done".as_slice().into() },
                cancel_delegates: false,
            },
        ));
    }

    fn stuck_question(&self, world: &World, index: usize, child: &tasks::TaskRecord) -> Result<u64, String> {
        child.questions.first().map(|question| question.number).ok_or_else(|| {
            let replies: Vec<_> = world.answers.iter().filter(|(_, key, _)| key.task == child.number).collect();
            self.stuck_finding(world, index, &format!("the delegate's question was not left open: {replies:?}"))
        })
    }

    fn idle_without_tries(&self, world: &World, index: usize, current: &tasks::TaskRecord) -> Result<(), String> {
        if current.phase != IDLE || current.tries.run != 0 || current.hold != 0 {
            return Err(self.stuck_finding(world, index, "expected idle without spends or holds"));
        }
        Ok(())
    }

    pub(super) fn stuck_settled(&self, world: &World, bound: Bound) -> Result<bool, String> {
        let Some(Draw::Nothing(pair)) = &self.stuck else { return Ok(true) };
        let goal = self.stuck_row(world, pair.goal).expect("settling the held delegate's requester");
        if goal.phase == IDLE {
            return Ok(true);
        }
        if bound.spent((world.clocks().0, world.iterations()), self.progress[pair.goal].last) {
            return Err(self.stuck_finding(world, pair.goal, "requester did not settle after its delegate's hold"));
        }
        Ok(false)
    }

    pub(super) fn check_stuck(&self, world: &World) -> Result<(), String> {
        match &self.stuck {
            Some(Draw::Circle(members)) => {
                let (_, before) = members.split_last().expect("circle");
                for pair in members {
                    let asker = self.stuck_row(world, pair.child).expect("settled asker");
                    self.idle_without_tries(world, pair.child, asker)?;
                    self.stuck_question(world, pair.child, asker)?;
                }
                for pair in before {
                    let goal = self.stuck_row(world, pair.goal).expect("settled waiter");
                    self.idle_without_tries(world, pair.goal, goal)?;
                }
            }
            Some(Draw::Watch { pair, worker, .. }) => {
                for index in [pair.goal, pair.child, *worker] {
                    let current = self.stuck_row(world, index).expect("settled control");
                    if !matches!(current.phase, tasks::Phase::Ended(tasks::Ending::Done(_)))
                        || current.hold != 0
                        || current.tries.run != 0
                    {
                        return Err(self.stuck_finding(
                            world,
                            index,
                            "watch control did not finish successfully without holds or tries",
                        ));
                    }
                }
            }
            Some(Draw::Nothing(pair)) => {
                let goal = self.stuck_row(world, pair.goal).expect("settled waiter");
                self.idle_without_tries(world, pair.goal, goal)?;
            }
            None => {}
        }
        Ok(())
    }

    fn stuck_finding(&self, world: &World, index: usize, why: &str) -> String {
        format!(
            "seed {}: task {} ({:?}) at {:?}, iteration {}: {why}; {:?}",
            self.seed,
            self.workload.tasks[index].name,
            self.progress[index].number,
            world.clocks().0,
            world.iterations(),
            self.stuck_row(world, index).map(|row| (&row.phase, row.tries.run))
        )
    }
}
