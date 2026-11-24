//! People's actions, drawn independently by party (`temper/docs/design/testing/workloads.md`, 3.3).

use std::collections::BTreeSet;

use skein_world::streams::{Stream, Streams};

use crate::draw::{Party, Workload, draw_party};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Act {
    CancelGoal,
    CancelChange,
    CancelLanding,
    HoldMember,
    ReleaseMember,
    AmendPassedDeadline,
    AnswerLate,
    NeverAnswer,
    AnswerTwice,
    AnswerEndedRun,
    WordsToEndedProducer,
}

/// The logical moment an act waits for in the world's durable records.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum When {
    Started,
    Opened,
    Asked,
    Landing,
    Held,
    DeadlinePassed,
    ProducerEnded,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Action {
    pub party: Party,
    pub act: Act,
    pub target: String,
    pub when: When,
    pub after: u32,
}

pub const AFTER_MOST: u32 = 50;

/// Each party draws from its own part (`first` or `second`) of `Stream::Parties`.
#[must_use]
pub fn draw(streams: &Streams, workload: &Workload, acts: &[Act]) -> Vec<Action> {
    [Party::First, Party::Second]
        .into_iter()
        .flat_map(|party| draw_for_party(*streams, workload, acts, party))
        .collect()
}

pub(crate) fn draw_for_party(streams: Streams, workload: &Workload, acts: &[Act], party: Party) -> Vec<Action> {
    let acts: BTreeSet<_> = acts.iter().copied().collect();
    let mut actions = Vec::new();
    if workload.goals.iter().any(|goal| goal.party == party) {
        let part = match party {
            Party::First => "first",
            Party::Second => "second",
        };
        let mut rng = streams.rng(Stream::Parties, part);
        // Reservations are party-local: one party's draw never reads the other's.
        // AmendPassedDeadline can target the other party's goal and overlap its acts;
        // excluding that overlap globally would couple the two independent draws.
        let mut taken = BTreeSet::new();
        let mut held = Vec::new();
        for &act in &acts {
            if rng.below(2) != 0 {
                continue;
            }
            let targets = if act == Act::ReleaseMember {
                held.clone()
            } else {
                targets(workload, party, act).into_iter().filter(|name| !taken.contains(name)).collect()
            };
            if targets.is_empty() {
                continue;
            }
            let target = targets[usize::try_from(rng.below(u64::try_from(targets.len()).expect("target count fits")))
                .expect("target index fits")]
            .clone();
            let actor = if act == Act::AmendPassedDeadline { draw_party(&mut rng) } else { party };
            let after = u32::try_from(rng.between(0, u64::from(AFTER_MOST))).expect("after fits in u32");
            taken.insert(target.clone());
            if act == Act::HoldMember {
                held.push(target.clone());
            }
            actions.push(Action { party: actor, act, target, when: when(act), after });
        }
    }
    actions
}

fn targets(workload: &Workload, party: Party, act: Act) -> Vec<String> {
    let mut names = Vec::new();
    for goal in &workload.goals {
        if goal.party != party && act != Act::AmendPassedDeadline {
            continue;
        }
        match act {
            Act::CancelGoal | Act::AmendPassedDeadline => names.push(goal.name.clone()),
            Act::CancelChange | Act::WordsToEndedProducer => {
                names.extend(goal.changes.iter().map(|change| change.name.clone()))
            }
            Act::CancelLanding | Act::HoldMember if goal.train => {
                names.extend(goal.changes.iter().map(|change| change.name.clone()));
            }
            Act::AnswerLate | Act::NeverAnswer | Act::AnswerTwice | Act::AnswerEndedRun => {
                names.extend(goal.changes.iter().filter(|change| change.asks).map(|change| change.name.clone()));
            }
            Act::CancelLanding | Act::HoldMember | Act::ReleaseMember => {}
        }
    }
    names
}

fn when(act: Act) -> When {
    match act {
        Act::CancelGoal => When::Started,
        Act::CancelChange => When::Opened,
        Act::CancelLanding | Act::HoldMember => When::Landing,
        Act::ReleaseMember => When::Held,
        Act::AmendPassedDeadline => When::DeadlinePassed,
        Act::AnswerLate | Act::NeverAnswer | Act::AnswerTwice => When::Asked,
        Act::AnswerEndedRun | Act::WordsToEndedProducer => When::ProducerEnded,
    }
}
