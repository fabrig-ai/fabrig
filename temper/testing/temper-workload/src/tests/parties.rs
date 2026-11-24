use std::collections::BTreeSet;

use skein_world::streams::{Stream, Streams};

use crate::draw::{COMPOSED, Party, Workload};
use crate::parties::{self, AFTER_MOST, Act, When};

const ACTS: [Act; 11] = [
    Act::CancelGoal,
    Act::CancelChange,
    Act::CancelLanding,
    Act::HoldMember,
    Act::ReleaseMember,
    Act::AmendPassedDeadline,
    Act::AnswerLate,
    Act::NeverAnswer,
    Act::AnswerTwice,
    Act::AnswerEndedRun,
    Act::WordsToEndedProducer,
];

#[test]
fn parties_draw_only_allowed_acts_targets_and_moments() {
    let mut seen = BTreeSet::new();
    let mut amendment_parties = BTreeSet::new();
    let mut amendment_owners = BTreeSet::new();
    for seed in 0..64 {
        let streams = Streams::new(seed);
        let workload = Workload::draw(&streams, &COMPOSED);
        for act in ACTS {
            let allowed = if act == Act::ReleaseMember { vec![Act::HoldMember, act] } else { vec![act] };
            let actions = parties::draw(&streams, &workload, &allowed);
            assert_eq!(actions, parties::draw(&streams, &workload, &allowed));
            for (i, action) in actions.iter().enumerate() {
                seen.insert(action.act);
                assert!(allowed.contains(&action.act));
                assert!(action.after <= AFTER_MOST);
                let expected_when = match action.act {
                    Act::CancelGoal | Act::AmendPassedDeadline => {
                        let goal = workload.goal(&action.target).expect("goal target");
                        if action.act == Act::AmendPassedDeadline {
                            amendment_parties.insert(action.party);
                            amendment_owners.insert(goal.party);
                            When::DeadlinePassed
                        } else {
                            assert_eq!(goal.party, action.party);
                            When::Started
                        }
                    }
                    _ => {
                        let (goal, change) = workload.change(&action.target).expect("change target");
                        assert_eq!(goal.party, action.party);
                        match action.act {
                            Act::CancelChange => When::Opened,
                            Act::CancelLanding | Act::HoldMember => {
                                assert!(goal.train);
                                When::Landing
                            }
                            Act::ReleaseMember => {
                                assert!(goal.train);
                                assert!(actions[..i].iter().any(|earlier| earlier.act == Act::HoldMember
                                    && earlier.target == action.target
                                    && earlier.party == action.party));
                                When::Held
                            }
                            Act::AnswerLate | Act::NeverAnswer | Act::AnswerTwice => {
                                assert!(change.asks);
                                When::Asked
                            }
                            Act::AnswerEndedRun => {
                                assert!(change.asks);
                                When::ProducerEnded
                            }
                            Act::WordsToEndedProducer => When::ProducerEnded,
                            Act::CancelGoal | Act::AmendPassedDeadline => unreachable!("goal target handled above"),
                        }
                    }
                };
                assert_eq!(action.when, expected_when);
            }
        }
        assert_eq!(parties::draw(&streams, &workload, &[Act::ReleaseMember]), []);
        assert_eq!(parties::draw(&streams, &workload, &[]), []);
    }
    assert_eq!(seen, ACTS.into_iter().collect());
    assert_eq!(amendment_parties, [Party::First, Party::Second].into_iter().collect());
    assert_eq!(amendment_owners, amendment_parties);
}

#[test]
fn parties_reserve_targets_order_acts_and_keep_streams_independent() {
    let acts: Vec<_> = ACTS.into_iter().filter(|act| *act != Act::AmendPassedDeadline).collect();
    for seed in 0..64 {
        let streams = Streams::new(seed);
        let workload = Workload::draw(&streams, &COMPOSED);
        let actions = parties::draw(&streams, &workload, &acts);
        let mut reversed = acts.clone();
        reversed.reverse();
        reversed.extend_from_slice(&acts);
        assert_eq!(actions, parties::draw(&streams, &workload, &reversed));
        let mut taken = BTreeSet::new();
        for action in &actions {
            if action.act != Act::ReleaseMember {
                assert!(taken.insert(&action.target), "one act per target: {actions:?}");
            }
        }
        // Each part stays unchanged by a different set of acts for the other party.
        let first = parties::draw_for_party(streams, &workload, &acts, Party::First);
        for other_acts in [&[][..], &acts[..], &[Act::CancelGoal][..]] {
            let _ = parties::draw_for_party(streams, &workload, other_acts, Party::Second);
            assert_eq!(first, parties::draw_for_party(streams, &workload, &acts, Party::First));
        }
        let second = parties::draw_for_party(streams, &workload, &acts, Party::Second);
        for other_acts in [&[][..], &acts[..], &[Act::AnswerTwice][..]] {
            let _ = parties::draw_for_party(streams, &workload, other_acts, Party::First);
            assert_eq!(second, parties::draw_for_party(streams, &workload, &acts, Party::Second));
        }
        assert_eq!(actions, first.into_iter().chain(second).collect::<Vec<_>>());
        let _ = streams.rng(Stream::Workload, "goals").next_u64();
        let _ = streams.rng(Stream::Parties, "second").next_u64();
        assert_eq!(actions, parties::draw(&streams, &workload, &acts));
        for party in [Party::First, Party::Second] {
            let only = Workload {
                goals: workload.goals.iter().filter(|goal| goal.party == party).cloned().collect(),
                tier: workload.tier,
            };
            assert_eq!(
                parties::draw(&streams, &only, &acts),
                actions.iter().filter(|action| action.party == party).cloned().collect::<Vec<_>>()
            );
        }
    }
}
