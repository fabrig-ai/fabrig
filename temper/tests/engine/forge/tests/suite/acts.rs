//! The train's party draw is independent of its workload (testing/workloads-kit.md, 1.3, testing/workloads.md, 4.2).
use skein_world::streams::{Stream, Streams};
use temper_workload::{
    draw::{Change, Goal, Party, Workload},
    parties::{self, Act, Action},
};

pub(super) fn draw(streams: Streams, count: u64) -> Vec<Action> {
    let workload = Workload {
        goals: vec![Goal {
            name: "train".into(),
            party: Party::First,
            after: None,
            train: true,
            changes: (0..count)
                .map(|index| Change {
                    name: format!("m{index}"),
                    after: vec![],
                    files: vec![],
                    asks: false,
                    red_once: false,
                })
                .collect(),
        }],
        tier: false,
    };
    parties::draw(&streams, &workload, &[Act::CancelLanding, Act::HoldMember, Act::ReleaseMember, Act::CancelChange])
}

#[test]
fn drawing_the_train_does_not_change_its_parties() {
    let streams = Streams::new(1);
    let before = draw(streams, 4);
    let _workload = super::fuzzy::landing_batches::Workload::new(streams.seed_of(Stream::Workload, "landing_batches"));
    assert_eq!(draw(streams, 4), before);
    assert_eq!(
        before.iter().map(|action| (action.act, action.after)).collect::<Vec<_>>(),
        [(Act::CancelChange, 7), (Act::HoldMember, 46), (Act::ReleaseMember, 49)],
        "the first party uses its own stream, not the workload's"
    );
}
