//! Workload data and its independent draw (`temper/docs/design/testing/workloads.md`, 3.1).

use skein_lib::Rng;
use skein_world::streams::{Stream, Streams};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Party {
    First,
    Second,
}

/// The least and the most of a count, inclusive.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Span {
    pub least: u8,
    pub most: u8,
}

/// What a world draws: each world names its own.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Shape {
    pub goals: Span,
    /// Changes of each goal that isn't the train.
    pub changes: Span,
    /// Goals in the milestone's chain; 0 and 1 mean none.
    pub milestone: Span,
    /// Members of the train goal's landing; 0 and 1 mean no train.
    pub train: Span,
    /// An after-landing tier following main.
    pub tier: bool,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Workload {
    pub goals: Vec<Goal>,
    pub tier: bool,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Goal {
    pub name: String,
    pub party: Party,
    /// The milestone's earlier goal it depends on, if any.
    pub after: Option<String>,
    pub train: bool,
    pub changes: Vec<Change>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Change {
    pub name: String,
    /// Earlier changes of this goal, or the preceding milestone goal's last change.
    pub after: Vec<String>,
    pub files: Vec<File>,
    pub asks: bool,
    pub red_once: bool,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct File {
    pub path: String,
    pub text: String,
}

pub const ENGINE: Shape = Shape {
    goals: Span { least: 2, most: 4 },
    changes: Span { least: 1, most: 3 },
    milestone: Span { least: 0, most: 2 },
    train: Span { least: 2, most: 3 },
    tier: true,
};

pub const COMPOSED: Shape = Shape {
    goals: Span { least: 3, most: 3 },
    changes: Span { least: 1, most: 2 },
    milestone: Span { least: 2, most: 2 },
    train: Span { least: 2, most: 3 },
    tier: true,
};

impl Workload {
    /// Draws from `Stream::Workload`, part `goals`, alone.
    #[must_use]
    pub fn draw(streams: &Streams, shape: &Shape) -> Workload {
        let mut rng = streams.rng(Stream::Workload, "goals");
        let count = draw_span(&mut rng, shape.goals);
        let mut parties: Vec<_> = (0..count).map(|_| draw_party(&mut rng)).collect();
        if count >= 2 && parties.iter().all(|party| *party == parties[0]) {
            let other = match parties[0] {
                Party::First => Party::Second,
                Party::Second => Party::First,
            };
            *parties.last_mut().expect("at least two parties") = other;
        }
        // A shape can request more milestone goals than it drew; only existing goals can chain.
        let milestone = draw_span(&mut rng, shape.milestone).min(count);
        let members = if count > 0 && shape.train.most >= 2 {
            draw_span(&mut rng, Span { least: shape.train.least.max(2), most: shape.train.most })
        } else {
            0
        };
        let mut goals: Vec<Goal> = Vec::new();
        for (i, party) in parties.into_iter().enumerate() {
            let name = format!("g{i}");
            let previous = if i > 0 && i < usize::from(milestone) { goals.last() } else { None };
            let after = previous.map(|goal| goal.name.clone());
            let previous_change = previous.and_then(|goal| goal.changes.last()).map(|change| change.name.as_str());
            let train = members >= 2 && i + 1 == usize::from(count);
            let count = if train { members } else { draw_span(&mut rng, shape.changes) };
            let changes = draw_changes(&mut rng, &name, count, previous_change, train);
            goals.push(Goal { name, party, after, train, changes });
        }
        Workload { goals, tier: shape.tier }
    }

    #[must_use]
    pub fn goal(&self, name: &str) -> Option<&Goal> {
        self.goals.iter().find(|goal| goal.name == name)
    }

    /// A change by its name, with its goal.
    #[must_use]
    pub fn change(&self, name: &str) -> Option<(&Goal, &Change)> {
        self.goals
            .iter()
            .find_map(|goal| goal.changes.iter().find(|change| change.name == name).map(|change| (goal, change)))
    }
}

fn draw_span(rng: &mut Rng, span: Span) -> u8 {
    u8::try_from(rng.between(u64::from(span.least), u64::from(span.most))).expect("span fits in u8")
}

fn draw_changes(rng: &mut Rng, goal: &str, count: u8, previous_goal: Option<&str>, train: bool) -> Vec<Change> {
    let mut changes = Vec::new();
    for j in 0..count {
        let name = format!("{goal}.c{j}");
        let candidate = if j > 0 { Some(format!("{goal}.c{}", j - 1)) } else { previous_goal.map(str::to_owned) };
        // Spend the same coin even when the train cannot keep a sibling edge:
        // its members must be ready together, rather than stacked landings.
        let after = candidate.filter(|_| rng.below(2) == 0).filter(|_| !train || j == 0).into_iter().collect();
        let files = (0..rng.between(1, 2))
            .map(|k| File {
                path: format!("{goal}/c{j}/f{k}.txt"),
                text: format!("{name} f{k} {:x}\n", rng.next_u64()),
            })
            .collect();
        let asks = rng.below(3) == 0;
        let red_once = rng.below(4) == 0;
        // A red member leaves its train; only non-train changes draw a repair.
        let red_once = red_once && !train;
        changes.push(Change { name, after, files, asks, red_once });
    }
    changes
}

pub(crate) fn draw_party(rng: &mut Rng) -> Party {
    if rng.below(2) == 0 { Party::First } else { Party::Second }
}
