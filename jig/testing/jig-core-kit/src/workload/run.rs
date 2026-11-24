//! The workload's loop keeps scripts separate from independently paced party inputs.
use super::{Act, Bound, Expect, Moment, Party, TaskKind, Workload};
use crate::{
    fixtures, open, people,
    world::{Setup, World},
};
use jig_domain_core as core;
use jig_domain_people as requests;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{Duration, ReplyTo, Time, Token, Wall};
use skein_world::{
    faults::{FaultLedger, Kind, Struck},
    streams::{Stream, Streams},
    sweep::Sweep,
};

mod actions;

#[expect(clippy::struct_excessive_bools, reason = "independent script milestones survive task holds and new attempts")]
#[derive(Debug)]
struct Progress {
    number: Option<u64>,
    submitted: bool,
    start_epoch: Option<usize>,
    pruned: bool,
    parked: bool,
    words: bool,
    last: (Time, u64),
    expect: Expect,
}

#[derive(Debug)]
struct DrawnAct {
    act: Act,
    seen: Option<u64>,
    fired: bool,
}

struct Driver {
    workload: Workload,
    progress: Vec<Progress>,
    acts: Vec<DrawnAct>,
    ledger: FaultLedger,
    sign_ins: [u64; 2],
    persons: [u64; 2],
    key: u8,
    acknowledgements: usize,
    completion: u32,
    script_target: Option<usize>,
    seed: u64,
    receipts: Vec<actions::Receipt>,
}

/// Run a workload with the caller's configured kinds and exclusions.
/// Each person question is a person delegate, answered through `AnswerPerson`
/// (jig/docs/design/domain/people/README.md, 7); a parked requester receives its
/// result on its next run, just as it receives any delegate's result.
pub fn run(streams: Streams, configured: &[Kind], sweep: &Sweep) -> Result<FaultLedger, String> {
    let workload = Workload::draw(streams);
    let (configuration, limits) = fixture(streams.seed_of(Stream::Scheduling, "core"));
    let mut world = World::new(Setup {
        seed: streams.seed(),
        configuration: Some(configuration),
        limits,
        initial: Some(requests::Ask::MarkSeen { seen: requests::Seen::Results { through: 0 } }),
        require_assignment: false,
        ..Setup::default()
    });
    let owner = world.sign_in.expect("owner signed in");
    let member = fixtures::second_party(&mut world, b"workload-member", requests::Role::Member);
    let owner_person = person(&world, owner);
    let member_person = person(&world, member);
    let acts = [Party::Owner, Party::Member]
        .into_iter()
        .flat_map(|party| workload.acts(streams, party, configured))
        .map(|act| DrawnAct { act, seen: None, fired: false })
        .collect();
    let last = (world.clocks().0, world.iterations());
    let progress = workload
        .tasks
        .iter()
        .map(|_| Progress {
            number: None,
            submitted: false,
            start_epoch: None,
            pruned: false,
            parked: false,
            words: false,
            last,
            expect: Expect::Ends,
        })
        .collect();
    let mut driver = Driver {
        workload,
        progress,
        acts,
        ledger: FaultLedger::new(),
        sign_ins: [owner, member],
        persons: [owner_person, member_person],
        key: 1,
        acknowledgements: 0,
        completion: 1000,
        script_target: None,
        seed: streams.seed(),
        receipts: Vec::new(),
    };
    let bound = Bound::default();
    let mut script_turn = true;
    loop {
        driver.refresh(&world);
        driver.make_request_room(&mut world);
        if script_turn {
            driver.script(&mut world)?;
        } else {
            driver.script_target = None;
        }
        script_turn = !script_turn;
        world
            .try_drain_with(&mut |world| {
                driver.refresh(world);
                driver.act(world, sweep)
            })
            .map_err(|finding| {
                let finding = world.diagnostic(&finding);
                eprintln!("{finding}");
                finding
            })?;
        driver.refresh(&world);
        if driver.finished(&world, bound).inspect_err(|finding| eprintln!("{finding}"))? {
            driver.check_outcomes(&world)?;
            return Ok(driver.ledger);
        }
        // Waiting on a person's choice does not busy-spin the simulator.
        // Each pass still drains and checks every immediate transition first.
        world.advance(Duration::from_millis(10));
    }
}

/// The plan's task, worker and funds bounds, with permission to ask both parties.
fn fixture(seed: u64) -> (root::Config, root::Limits) {
    use jig_domain_authority::Executor;
    let (mut configuration, mut limits) = crate::faults::plan_fixture(seed);
    limits.core.authority.roles = 2;
    limits.core.authority.executors = 3;
    limits.core.documents.executors = 3;
    let kinds: Box<[Executor]> =
        Box::new([Executor::Charter(1), Executor::Role(0), Executor::Role(requests::Role::Member.number())]);
    configuration.core.defaults.rules.ceiling.delegation.kinds.clone_from(&kinds);
    let policy = &mut configuration.core.seed_projects[0].policy.policy;
    policy.ceiling.delegation.kinds.clone_from(&kinds);
    let mut member = policy.roles[0].clone();
    member.number = requests::Role::Member.number();
    member.decides = jig_domain_authority::Proposals(0);
    member.authority.delegation.kinds = Box::new([Executor::Charter(1), Executor::Role(member.number)]);
    policy.roles[0].authority.delegation.kinds = kinds;
    policy.roles = Box::new([policy.roles[0].clone(), member]);
    (configuration, limits)
}

fn person(world: &World, sign_in: u64) -> u64 {
    world
        .people_answers
        .iter()
        .find_map(|reply| match reply {
            requests::Reply::SignedIn { sign_in: actual, person, .. } if *actual == sign_in => Some(*person),
            _ => None,
        })
        .expect("signed-in person's number")
}

pub(super) fn row(world: &World, number: u64) -> Option<&tasks::TaskRecord> {
    [tasks::Key::Live(number), tasks::Key::Ended(number)].into_iter().find_map(|key| {
        match world.store.rows.get(&root::Key::Core(core::Key::Tasks(key))) {
            Some(root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row) | tasks::Stored::Ended(row)))) => {
                Some(row.as_ref())
            }
            _ => None,
        }
    })
}

fn live_count(world: &World) -> u32 {
    world
        .store
        .rows
        .values()
        .filter(|record| matches!(record, root::Record::Core(core::Record::Tasks(tasks::Stored::Live(_)))))
        .count()
        .try_into()
        .expect("bounded live rows")
}

fn hold_room(world: &World) -> bool {
    world
        .store
        .rows
        .values()
        .filter(|record| {
            matches!(record,
        root::Record::Core(core::Record::Tasks(tasks::Stored::PlacedHold(hold))) if !hold.lifted)
        })
        .count()
        < usize::try_from(world.env().limits.core.tasks.placed_holds).expect("bounded holds")
}

fn ended(world: &World, number: u64) -> bool {
    row(world, number).is_some_and(|row| matches!(row.phase, tasks::Phase::Ended(_)))
}

impl Driver {
    fn make_request_room(&mut self, world: &mut World) {
        let answers: Vec<_> = world
            .store
            .rows
            .values()
            .filter_map(|record| match record {
                root::Record::Core(core::Record::People(requests::Stored::Answer { at, .. })) => Some(*at),
                _ => None,
            })
            .collect();
        // Keep room for a duplicate answer and a script's request without bypassing admission.
        if answers.len() + 3 < usize::try_from(world.env().limits.core.people.requests).expect("bounded requests") {
            return;
        }
        let until = answers.into_iter().min().expect("full answer table").as_nanos()
            + world.env().limits.core.people.request_retention.as_nanos()
            + 1;
        world.advance(Duration::from_nanos(until.saturating_sub(world.clocks().1.as_nanos())));
        for (party, subject) in
            [(Party::Owner, 7_u64.to_be_bytes().to_vec()), (Party::Member, b"workload-member".to_vec())]
        {
            if let Some(root::Record::Core(core::Record::People(requests::Stored::SignIn { expires, .. }))) = world
                .store
                .rows
                .get(&root::Key::Core(core::Key::People(requests::Key::SignIn(self.sign_ins[party.index()]))))
                && *expires > world.clocks().1
            {
                continue;
            }
            world.send(fixtures::sign_in(
                Token::new(30_000 + u64::try_from(party.index()).expect("two parties")),
                0,
                &subject,
                party.part().as_bytes(),
                party.part().as_bytes(),
            ));
            let Some(requests::Reply::SignedIn { sign_in, .. }) = world.people_answers.last() else {
                panic!("workload party signs in again: {:?}", world.people_answers.last())
            };
            self.sign_ins[party.index()] = *sign_in;
        }
    }

    fn ask(&mut self, world: &mut World, party: Party, ask: requests::Ask) -> Result<u8, String> {
        let key = self.key;
        self.key = self.key.checked_add(1).ok_or_else(|| "workload exhausted request keys".to_string())?;
        world.enqueue(fixtures::party_ask(8200, self.sign_ins[party.index()], key, ask));
        Ok(key)
    }

    fn refresh(&mut self, world: &World) {
        for (index, task) in self.workload.tasks.iter().enumerate() {
            if self.progress[index].number.is_none() {
                self.progress[index].number =
                    world.store.rows.values().find_map(|record| match record {
                        root::Record::Core(core::Record::Tasks(
                            tasks::Stored::Live(row) | tasks::Stored::Ended(row),
                        )) if row.spec.words.as_ref() == task.name.as_bytes() => Some(row.number),
                        _ => None,
                    });
            }
            if let Some(parent) = task.parent {
                let pruned =
                    self.progress[parent].pruned || self.progress[parent].number.is_some_and(|n| ended(world, n));
                if self.progress[index].number.is_none() && pruned {
                    self.progress[index].pruned = true;
                }
            }
        }
        for (index, progress) in self.progress.iter_mut().enumerate() {
            let never = self
                .acts
                .iter()
                .any(|act| act.act.target == index && act.fired && act.act.kind == people::NEVER_ANSWERS);
            let held = progress.number.and_then(|number| row(world, number)).is_some_and(|row| {
                row.placed.is_some() || matches!(row.phase, tasks::Phase::Held { why: tasks::Hold::Stopped, .. })
            });
            progress.expect = if held || never { Expect::Waits } else { Expect::Ends };
        }
        // A waiting child keeps each ancestor waiting, rather than giving it an independent timeout.
        for index in (0..self.progress.len()).rev() {
            if self.progress[index].expect == Expect::Waits
                && let Some(parent) = self.workload.tasks[index].parent
            {
                self.progress[parent].expect = Expect::Waits;
            }
        }
    }

    fn read_run(&mut self, world: &mut World, number: u64) -> bool {
        let current = row(world, number).expect("scripted live task");
        if matches!(current.phase, tasks::Phase::Active(tasks::Active::Running { .. })) {
            if let Some(last) = current.inbox.last() {
                let assigned = world
                    .assignment_transcripts
                    .iter()
                    .rev()
                    .find(|assigned| assigned.task == number && assigned.attempt == current.attempt)
                    .expect("running assignment");
                let offered =
                    world.inbound.iter().chain(assigned.messages.iter()).any(|word| word.number == last.number);
                world.enqueue(root::Event::Turn {
                    channel: Token::new(7),
                    task: number,
                    attempt: current.attempt,
                    turn: current.turn + 1,
                    cumulative: 0,
                    read: offered.then_some(last.number),
                    transcript: b"workload read".as_slice().into(),
                    opens_window: current.turn == 0 && assigned.turns.is_empty(),
                    refused: Box::new([]),
                });
                return true;
            }
            if !current.unrecorded.is_empty() {
                self.completion += 1;
                world.enqueue(root::Event::Core(core::Event::NamedAction {
                    to: ReplyTo::new(Token::new(20_000 + u64::from(self.completion))),
                    key: core::CallKey {
                        task: number,
                        attempt: current.attempt,
                        completion: self.completion,
                        position: 0,
                    },
                    action: core::NamedAction::MarkRead {
                        messages: current.unrecorded.iter().map(|word| word.number).collect(),
                    },
                }));
                return true;
            }
        }
        false
    }

    fn create(&mut self, world: &mut World, index: usize) -> Result<bool, String> {
        let task = &self.workload.tasks[index];
        if self.progress[index].submitted {
            return Ok(false);
        }
        if let Some(parent) = task.parent {
            let Some(parent_number) = self.progress[parent].number else { return Ok(false) };
            let Some(parent_row) = row(world, parent_number) else { return Ok(false) };
            if !matches!(parent_row.phase, tasks::Phase::Active(tasks::Active::Running { .. })) {
                return Ok(false);
            }
            if live_count(world) >= world.env().limits.core.tasks.tasks {
                return Ok(false);
            }
            let mut member = fixtures::member(
                world,
                match task.kind {
                    TaskKind::Question => {
                        tasks::Executor::Person(tasks::PersonAddress::Person(self.persons[task.party.index()]))
                    }
                    TaskKind::Run { .. } => tasks::Executor::Agent { charter: 1 },
                },
                task.name.as_bytes(),
            );
            member.authority = parent_row.authority.clone();
            member.authority.budget.spend = task.budget;
            // Child authority is bounded by its remaining depth and subtree size.
            member.authority.delegation.tasks = self.descendants(index);
            member.authority.delegation.depth = 2_u32.saturating_sub(parent_row.depth + 1);
            let completion = u32::try_from(index + 1).expect("bounded workload");
            world.enqueue(root::Event::Core(core::Event::DelegateValidated {
                to: ReplyTo::new(Token::new(9000 + u64::from(completion))),
                key: core::CallKey { task: parent_number, attempt: parent_row.attempt, completion, position: 0 },
                batch: Box::new([member]),
                stubs: Box::new([]),
            }));
            self.progress[index].submitted = true;
            return Ok(true);
        }
        // A refused start consumes no place; retry only after a live goal ends.
        let epoch = self
            .progress
            .iter()
            .enumerate()
            .filter(|(index, p)| {
                self.workload.tasks[*index].parent.is_none() && p.number.is_some_and(|n| ended(world, n))
            })
            .count();
        if self.progress[index].start_epoch == Some(epoch) {
            return Ok(false);
        }
        if live_count(world) >= world.env().limits.core.tasks.tasks {
            return Ok(false);
        }
        let deadline = Wall::from_nanos(world.clocks().1.as_nanos() + task.deadline_ms * 1_000_000);
        let mut request = fixtures::goal(
            world,
            1,
            task.name.as_bytes().into(),
            requests::TaskExecutor::Agent { charter: 1 },
            jig_test_profile::fixtures::Goal { budget: task.budget, deadline: Some(deadline), priority: 1 },
        );
        if let requests::Ask::Create { batch, .. } = &mut request {
            batch[0].authority.delegation.kinds = Box::new([
                requests::Executor::Charter(1),
                requests::Executor::Role(match task.party {
                    Party::Owner => 0,
                    Party::Member => requests::Role::Member.number(),
                }),
            ]);
        }
        self.ask(world, task.party, request)?;
        self.progress[index].start_epoch = Some(epoch);
        Ok(true)
    }

    fn script(&mut self, world: &mut World) -> Result<(), String> {
        self.script_target = None;
        // Every cancellation is acknowledged by the worker, including a held run.
        if let Some(&(task, attempt)) = world.cancellations.get(self.acknowledgements) {
            self.acknowledgements += 1;
            world.enqueue(answer(task, attempt, tasks::End::Parked));
            return Ok(());
        }
        for index in 0..self.progress.len() {
            if self.workload.tasks[index].parent.is_none()
                && self.progress[index].number.is_none()
                && !self.progress[index].pruned
            {
                self.script_target = Some(index);
                if self.create(world, index)? {
                    return Ok(());
                }
            }
        }
        for index in 0..self.progress.len() {
            if self.progress[index].pruned || self.receipts.iter().any(|receipt| self.related(index, receipt.target)) {
                continue;
            }
            self.script_target = Some(index);
            if self.script_task(world, index)? {
                return Ok(());
            }
        }
        self.script_target = None;
        self.retire_waiting(world)
    }

    fn script_task(&mut self, world: &mut World, index: usize) -> Result<bool, String> {
        let Some(number) = self.progress[index].number else {
            if self.create(world, index)? {
                return Ok(true);
            }
            return Ok(false);
        };
        if self.read_run(world, number) {
            return Ok(true);
        }
        let task = &self.workload.tasks[index];
        let Some(current) = row(world, number) else { return Ok(false) };
        if matches!(current.phase, tasks::Phase::Ended(_)) {
            return Ok(false);
        }
        if matches!(current.phase, tasks::Phase::Held { why: tasks::Hold::Deadline, .. })
            && !self
                .acts
                .iter()
                .any(|act| !act.fired && act.act.target == index && act.act.kind == open::AMENDS_A_PASSED_DEADLINE)
        {
            self.ask(
                world,
                task.party,
                requests::Ask::Cancel {
                    project: 1,
                    task: number,
                    reason: b"deadline ended work".as_slice().into(),
                    grace: None,
                },
            )?;
            return Ok(true);
        }
        if let TaskKind::Question = task.kind {
            let reserved = self.acts.iter().any(|act| act.act.target == index && !act.fired);
            let never = self
                .acts
                .iter()
                .any(|act| act.act.target == index && act.fired && act.act.kind == people::NEVER_ANSWERS);
            if current.person_wait.is_some() && matches!(current.phase, tasks::Phase::Active(_)) && !reserved && !never
            {
                self.ask(
                    world,
                    task.party,
                    requests::Ask::AnswerPerson {
                        project: 1,
                        task: number,
                        revision: current.revisions.person,
                        result: requests::PersonResult::Report { words: b"answer".as_slice().into() },
                    },
                )?;
                return Ok(true);
            }
            return Ok(false);
        }
        self.run_step(world, index, number)
    }

    fn run_step(&mut self, world: &mut World, index: usize, number: u64) -> Result<bool, String> {
        let task = &self.workload.tasks[index];
        let current = row(world, number).expect("scripted running task");
        let before_deadline = task.short
            && current.authority.budget.deadline.is_some_and(|due| {
                due > world.clocks().1 && due.as_nanos() <= current.created_at.as_nanos() + task.deadline_ms * 1_000_000
            });
        if matches!(current.phase, tasks::Phase::Active(tasks::Active::Idle))
            && !self.workload.tasks.iter().enumerate().any(|(child, task)| {
                task.parent == Some(index) && self.progress[child].number.is_some_and(|n| !ended(world, n))
            })
            && !before_deadline
            && !self.acts.iter().any(|act| {
                !act.fired
                    && act.act.moment == Moment::RunEnded
                    && (act.act.target == index || self.workload.tasks[act.act.target].parent == Some(index))
            })
        {
            self.ask(
                world,
                task.party,
                requests::Ask::Say { project: 1, task: number, words: b"continue work".as_slice().into() },
            )?;
            return Ok(true);
        }
        if !matches!(current.phase, tasks::Phase::Active(tasks::Active::Running { .. })) {
            return Ok(false);
        }
        let children_pending = self.workload.tasks.iter().enumerate().any(|(child, task)| {
            task.parent == Some(index)
                && !self.progress[child].pruned
                && self.progress[child].number.is_none_or(|number| !ended(world, number))
        });
        let missing_children = self.workload.tasks.iter().enumerate().any(|(child, task)| {
            task.parent == Some(index) && !self.progress[child].pruned && self.progress[child].number.is_none()
        });
        if missing_children {
            if live_count(world) >= world.env().limits.core.tasks.tasks
                && self.workload.tasks.iter().enumerate().any(|(child, task)| {
                    task.parent == Some(index) && self.progress[child].number.is_some_and(|n| !ended(world, n))
                })
            {
                world.enqueue(park(current));
                return Ok(true);
            }
            return Ok(false);
        }
        let needs_park = self.acts.iter().any(|act| {
            !act.fired
                && act.act.moment == Moment::RunEnded
                && (act.act.target == index || self.workload.tasks[act.act.target].parent == Some(index))
        });
        if needs_park && !self.progress[index].parked {
            self.progress[index].parked = !current.subscriptions.is_empty();
            world.enqueue(park(current));
            return Ok(true);
        }
        if task.short
            && current
                .authority
                .budget
                .deadline
                .is_some_and(|due| due.as_nanos() <= current.created_at.as_nanos() + task.deadline_ms * 1_000_000)
        {
            world.enqueue(park(current));
            return Ok(true);
        }
        if children_pending {
            world.enqueue(park(current));
            return Ok(true);
        }
        if let TaskKind::Run { words: true } = task.kind
            && !self.progress[index].words
        {
            self.ask(
                world,
                task.party,
                requests::Ask::Say { project: 1, task: number, words: b"workload words".as_slice().into() },
            )?;
            self.progress[index].words = true;
            return Ok(true);
        }
        world.enqueue(answer(
            number,
            current.attempt,
            tasks::End::Finished {
                result: tasks::TaskResult::Report { words: b"done".as_slice().into() },
                cancel_delegates: false,
            },
        ));
        Ok(true)
    }

    fn retire_waiting(&mut self, world: &mut World) -> Result<(), String> {
        // A completed waiting tree can be retired to admit a later drawn goal.
        // Its expectation is checked before that cleanup cancellation.
        if self.progress.iter().any(|p| p.number.is_none() && !p.pruned) {
            for index in 0..self.progress.len() {
                if self.workload.tasks[index].parent.is_some() || self.progress[index].expect != Expect::Waits {
                    continue;
                }
                let Some(number) = self.progress[index].number else { continue };
                if ended(world, number) {
                    continue;
                }
                let prefix = self.workload.tasks[index].name.clone() + ".";
                if self.acts.iter().any(|a| {
                    !a.fired
                        && (a.act.target == index || self.workload.tasks[a.act.target].name.starts_with(&prefix))
                        && self.progress[a.act.target].number.is_some()
                        && matches!(
                            a.act.moment,
                            Moment::Exists | Moment::Held | Moment::DeadlineHeld | Moment::QuestionOpen
                        )
                }) {
                    continue;
                }
                self.progress[index].expect.check(
                    (&self.workload.tasks[index].name, number),
                    self.seed,
                    (world.clocks().0, world.iterations()),
                    self.progress[index].last,
                    Bound::default(),
                    false,
                )?;
                self.script_target = Some(index);
                self.ask(
                    world,
                    self.workload.tasks[index].party,
                    requests::Ask::Cancel {
                        project: 1,
                        task: number,
                        reason: b"retire waiting tree".as_slice().into(),
                        grace: None,
                    },
                )?;
                return Ok(());
            }
        }
        Ok(())
    }

    fn related(&self, left: usize, right: usize) -> bool {
        let a = &self.workload.tasks[left].name;
        let b = &self.workload.tasks[right].name;
        a == b || a.starts_with(&(b.clone() + ".")) || b.starts_with(&(a.clone() + "."))
    }

    fn descendants(&self, index: usize) -> u32 {
        self.workload
            .tasks
            .iter()
            .enumerate()
            .filter(|(child, _)| {
                let mut parent = self.workload.tasks[*child].parent;
                while let Some(next) = parent {
                    if next == index {
                        return true;
                    }
                    parent = self.workload.tasks[next].parent;
                }
                false
            })
            .count()
            .try_into()
            .expect("bounded subtree")
    }

    fn waiting_for_party(&self, mut index: usize, world: &World) -> bool {
        loop {
            if self.progress[index].number.and_then(|n| row(world, n)).is_some_and(|r| r.placed.is_some()) {
                return true;
            }
            if self.acts.iter().any(|a| a.fired && a.act.target == index && a.act.kind == people::NEVER_ANSWERS) {
                return true;
            }
            let Some(parent) = self.workload.tasks[index].parent else { return false };
            index = parent;
        }
    }

    fn finished(&mut self, world: &World, bound: Bound) -> Result<bool, String> {
        let moment = (world.clocks().0, world.iterations());
        let mut all = true;
        for (index, progress) in self.progress.iter().enumerate() {
            if progress.pruned {
                continue;
            }
            let Some(number) = progress.number else {
                if self.waiting_for_party(index, world) {
                    continue;
                }
                all = false;
                if moment.1.saturating_sub(progress.last.1) > bound.iterations {
                    return Err(format!(
                        "seed {}: task {} not admitted at {:?}",
                        self.seed, self.workload.tasks[index].name, moment
                    ));
                }
                continue;
            };
            progress
                .expect
                .check(
                    (&self.workload.tasks[index].name, number),
                    self.seed,
                    moment,
                    progress.last,
                    bound,
                    ended(world, number),
                )
                .map_err(|finding| {
                    format!(
                        "{finding}; phase {:?}; progress {:?}; acts {:?}",
                        row(world, number).map(|row| &row.phase),
                        self.progress,
                        self.acts
                    )
                })?;
            all &= ended(world, number) || progress.expect == Expect::Waits;
        }
        Ok(all
            && self.receipts.is_empty()
            && self.acts.iter().all(|act| {
                act.fired
                    || self.progress[act.act.target].pruned
                    || self.progress[act.act.target].number.is_some_and(|number| ended(world, number))
                    || self.waiting_for_party(act.act.target, world)
                        && !matches!(act.act.moment, Moment::Exists | Moment::Held)
                    || act.act.kind == people::HOLDS_A_TASK && !hold_room(world)
            }))
    }
}

fn park(row: &tasks::TaskRecord) -> root::Event {
    if row.subscriptions.is_empty() {
        let subscription = u64::MAX - row.number;
        root::Event::Core(core::Event::Tasks(tasks::Event::Subscribe {
            reply_to: ReplyTo::new(Token::new(subscription)),
            task: row.number,
            subscription: tasks::Subscription {
                number: subscription,
                kind: tasks::SubscriptionKind::Timer { at: Wall::from_nanos(u64::MAX), period: None },
            },
        }))
    } else {
        answer(row.number, row.attempt, tasks::End::Parked)
    }
}

fn answer(task: u64, attempt: u64, end: tasks::End) -> root::Event {
    root::Event::Answer { channel: Token::new(7), task, attempt, cumulative: 0, end, read: None, refused: Box::new([]) }
}
