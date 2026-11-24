//! The draw's order and names are independent of every fault and actor stream.
use crate::{open, people};
use skein_lib::Rng;
use skein_world::{
    faults::Kind,
    streams::{Stream, Streams},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Party {
    Owner,
    Member,
}

impl Party {
    #[must_use]
    pub const fn part(self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Member => "member",
        }
    }

    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Owner => 0,
            Self::Member => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskKind {
    Run { words: bool },
    Question,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Task {
    pub name: String,
    pub parent: Option<usize>,
    pub party: Party,
    pub kind: TaskKind,
    /// A delegate's one-third question draw addresses its requester, not a person task.
    pub asks_requester: bool,
    pub budget: u64,
    /// Milliseconds from admission; the short quarter passes while work waits.
    pub deadline_ms: u64,
    pub short: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Workload {
    pub tasks: Vec<Task>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Moment {
    Exists,
    QuestionOpen,
    Held,
    DeadlineHeld,
    RunEnded,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Act {
    pub kind: Kind,
    pub party: Party,
    pub target: usize,
    pub moment: Moment,
    pub after: u64,
}

impl Workload {
    /// Draw within the plan profile's five-task tree, three delegates and two levels.
    /// A person question occupies a child slot too. The one-third question draw
    /// applies wherever it fits; no intent is drawn beyond a full tree or its depth.
    #[must_use]
    pub fn draw(streams: Streams) -> Self {
        let mut rng = streams.rng(Stream::Workload, "tasks");
        let goals = 2 + rng.below(3);
        let first = if rng.below(2) == 0 { Party::Owner } else { Party::Member };
        let mut tasks = Vec::new();
        for goal in 0..goals {
            let party = if goal == 0 {
                first
            } else if goal == 1 {
                match first {
                    Party::Owner => Party::Member,
                    Party::Member => Party::Owner,
                }
            } else if rng.below(2) == 0 {
                Party::Owner
            } else {
                Party::Member
            };
            let budget = 60 + rng.below(41);
            let short = rng.below(4) == 0;
            let deadline_ms = if short { 200 + rng.below(201) } else { 600_000 + rng.below(60_001) };
            let mut room = 5;
            draw_task(&mut tasks, &mut rng, &format!("t{goal}"), None, party, budget, 0, &mut room, deadline_ms, short);
        }
        // Keep the seed's draw order and tree slots. A question position under a
        // delegate is that run's own question, not another task to admit.
        let mut positions: Vec<usize> = Vec::new();
        let mut kept: Vec<Task> = Vec::new();
        for task in &tasks {
            let parent = task.parent.map(|parent| positions[parent]);
            if task.kind == TaskKind::Question && task.parent.is_some_and(|p| tasks[p].parent.is_some()) {
                kept[parent.expect("requester question has a parent")].asks_requester = true;
                positions.push(kept.len());
            } else {
                positions.push(kept.len());
                let mut task = task.clone();
                task.parent = parent;
                kept.push(task);
            }
        }
        Self { tasks: kept }
    }

    /// Each actor consumes only its own part, even if the other actor's configured acts change.
    #[must_use]
    pub fn acts(&self, streams: Streams, party: Party, configured: &[Kind]) -> Vec<Act> {
        let mut rng = streams.rng(Stream::Parties, party.part());
        let mut acts = Vec::new();
        for kind in configured
            .iter()
            .copied()
            .filter(|kind| kind.family == people::FAMILY)
            .collect::<std::collections::BTreeSet<_>>()
        {
            let chosen = rng.below(2) == 0;
            let (moment, candidates): (_, Vec<_>) = if kind == people::RELEASES_A_TASK {
                (
                    Moment::Held,
                    acts.iter()
                        .filter_map(|act: &Act| (act.kind == people::HOLDS_A_TASK).then_some(act.target))
                        .collect(),
                )
            } else {
                let moment = if [people::ANSWERS_LATE, people::NEVER_ANSWERS, people::ANSWERS_TWICE].contains(&kind) {
                    Moment::QuestionOpen
                } else if kind == people::ANSWERS_AN_ENDED_RUN || kind == people::WORDS_AFTER_ITS_RUN_ENDED {
                    Moment::RunEnded
                } else if kind == open::AMENDS_A_PASSED_DEADLINE {
                    Moment::DeadlineHeld
                } else {
                    Moment::Exists
                };
                let candidates = self
                    .tasks
                    .iter()
                    .enumerate()
                    .filter_map(|(index, task)| {
                        let owned =
                            task.party == party || kind == open::AMENDS_A_PASSED_DEADLINE && party == Party::Owner;
                        let shape = match moment {
                            Moment::QuestionOpen => task.kind == TaskKind::Question,
                            Moment::RunEnded if kind == people::ANSWERS_AN_ENDED_RUN => task.kind == TaskKind::Question,
                            Moment::DeadlineHeld => task.parent.is_none() && task.short,
                            Moment::Exists if kind == open::WIDENS_GOAL_AUTHORITY => task.parent.is_none(),
                            Moment::Exists | Moment::Held | Moment::RunEnded => {
                                matches!(task.kind, TaskKind::Run { .. })
                            }
                        };
                        (owned && shape).then_some(index)
                    })
                    .collect();
                (moment, candidates)
            };
            let target_draw = rng.next_u64();
            let after = rng.below(8);
            if chosen && !candidates.is_empty() {
                let index = usize::try_from(target_draw % u64::try_from(candidates.len()).expect("bounded tasks"))
                    .expect("bounded index");
                acts.push(Act { kind, party, target: candidates[index], moment, after });
            }
        }
        acts
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "recursive draw carries the bounded tree's place and inherited goal terms"
)]
fn draw_task(
    tasks: &mut Vec<Task>,
    rng: &mut Rng,
    name: &str,
    parent: Option<usize>,
    party: Party,
    budget: u64,
    depth: u32,
    room: &mut u32,
    deadline_ms: u64,
    short: bool,
) {
    *room -= 1;
    let index = tasks.len();
    let words = rng.below(4) == 0;
    tasks.push(Task {
        name: name.to_owned(),
        parent,
        party,
        kind: TaskKind::Run { words },
        asks_requester: false,
        budget,
        deadline_ms,
        short,
    });
    let asks = depth < 2 && *room > 0 && rng.below(3) == 0;
    let mut child_number = 0;
    if asks {
        *room -= 1;
        tasks.push(Task {
            name: format!("{name}.{child_number}"),
            parent: Some(index),
            party,
            kind: TaskKind::Question,
            asks_requester: false,
            budget: 0,
            deadline_ms,
            short,
        });
        child_number += 1;
    }
    if depth < 2 && *room > 0 {
        let children = if depth == 0 { rng.below(3) } else { rng.below(2) };
        for _ in 0..children {
            if *room == 0 {
                break;
            }
            draw_task(
                tasks,
                rng,
                &format!("{name}.{child_number}"),
                Some(index),
                party,
                budget / 3,
                depth + 1,
                room,
                deadline_ms,
                short,
            );
            child_number += 1;
        }
    }
}
