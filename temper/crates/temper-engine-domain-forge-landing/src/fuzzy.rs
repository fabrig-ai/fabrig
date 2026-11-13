//! Seeded snapshots of stacks and their changing rounds (domain/landing.md, 4–8).
use crate::tests::step::{decide, stack, stack_facts};
use crate::{Combine, Decision, Landing, Outcome, Phase, Preparation, Proof, Status};
use skein_lib::{List, Rng};

#[test]
fn seeded_stacks_keep_their_prefix_and_turn_invariants() {
    for seed in 0..256 {
        assert_eq!(run(seed), run(seed), "replay seed {seed}");
    }
}

fn run(seed: u64) -> (Landing, Box<[Decision]>) {
    let mut rng = Rng::new(seed);
    let count = usize::try_from(rng.below(6).checked_add(1).unwrap()).unwrap();
    let units = usize::try_from(rng.below(4).checked_add(1).unwrap()).unwrap().min(count);
    let mut landing = stack(count);
    let mut f = stack_facts(count);
    for (index, member) in landing.members.iter_mut().enumerate() {
        member.unit = u16::try_from(index.checked_mul(units).unwrap().checked_div(count).unwrap()).unwrap();
        f.members[index].judgement = Status::Pending;
        if index > 0 && rng.below(4) == 0 {
            f.members[index].dependencies = Box::new([u16::try_from(index.checked_sub(1).unwrap()).unwrap()]);
        }
    }
    let mut trace = List::with_capacity(160);
    for tick in 0_u32..160 {
        if tick < 24 && landing.round.ending.is_none() {
            let index = usize::try_from(rng.below(u64::try_from(count).unwrap())).unwrap();
            let member = &landing.members[index];
            if member.left.is_none() && member.landed.is_none() {
                match rng.below(12) {
                    0 => f.members[index].cancelled = true,
                    1 => f.members[index].head = [90; 32],
                    2 if landing.phase == Phase::Preparing && member.candidate.is_none() => {
                        f.members[index].outcome =
                            Some(if rng.below(2) == 0 { Outcome::Conflict } else { Outcome::EarlierConflict });
                    }
                    3 if landing.phase == Phase::Proving => f.members[index].gates[0].receipt = Status::Failed,
                    4 if crate::domain::matches_turn(&landing.phase) => {
                        for fresh in &mut f.members {
                            fresh.main = [91; 32];
                        }
                    }
                    _ => {}
                }
            }
        }
        let next = decide(&landing, &f);
        assert_eq!(next, decide(&landing, &f), "snapshot seed {seed} tick {tick}");
        if let Decision::Merge { member, candidate } = next.decision {
            let mut last = None;
            for (index, m) in landing.members.iter().enumerate() {
                if m.left.is_none() && m.landed.is_none() {
                    last = Some(index);
                    assert_eq!(f.members[index].judgement, Status::Passed, "judged seed {seed}");
                }
            }
            let last = last.unwrap();
            let top = &landing.members[last];
            assert_eq!(usize::from(member), last, "top seed {seed}");
            assert_eq!(Some(candidate), top.candidate, "prefix seed {seed}");
            for gate in &top.gates {
                match gate.proof {
                    Proof::Passed { .. } => {}
                    Proof::Unkeyed | Proof::Keyed { .. } | Proof::Dispatched { .. } | Proof::Failed { .. } => {
                        panic!("gates seed {seed}")
                    }
                }
            }
        }
        // The cleanup boundary, not a leave notification, retires the changed suffix.
        if landing.round.ending.is_some() && next.landing.round.ending.is_none() {
            let first = if landing.round.ending == Some(crate::RoundEnd::Stale) {
                0
            } else {
                usize::from(landing.round.changed.unwrap())
            };
            for (index, (old, new)) in landing.members.iter().zip(&next.landing.members).enumerate() {
                if index < first {
                    assert_eq!(old, new, "kept below leave seed {seed}");
                } else {
                    assert_eq!(new.candidate, None, "cleared suffix seed {seed}");
                    f.members[index].candidate = None;
                    f.members[index].deleted = Status::Unknown;
                    f.members[index].outcome = None;
                    f.members[index].judgement = Status::Pending;
                }
            }
            f.closed = Status::Unknown;
            f.opened = None;
            f.preparation = Preparation::Idle;
        }
        trace.push(next.decision).unwrap();
        match next.decision {
            Decision::Prepare { member, .. } => {
                f.preparation = Preparation::Failed;
                // Partial pushes, including repeated reads of the kept prefix.
                let take = if tick < 24 {
                    usize::try_from(rng.below(u64::try_from(count).unwrap()).checked_add(1).unwrap()).unwrap()
                } else {
                    count
                };
                for (index, m) in next.landing.members.iter().enumerate().skip(usize::from(member)).take(take) {
                    if m.left.is_none() && m.landed.is_none() && f.members[index].outcome.is_none() {
                        f.members[index].candidate = Some((
                            [u8::try_from(index.checked_add(30).unwrap()).unwrap(); 32],
                            if rng.below(2) == 0 { Combine::Merge } else { Combine::Squash },
                        ));
                    }
                }
            }
            Decision::Open => {
                f.opened = Some(99);
                f.opening = Status::Passed;
            }
            Decision::Close => f.closed = Status::Passed,
            Decision::Delete { member } => f.members[usize::from(member)].deleted = Status::Passed,
            Decision::Judge { member } => f.members[usize::from(member)].judgement = Status::Passed,
            Decision::Key { member, candidate, .. } => f.members[usize::from(member)].gates[0].key = Some(candidate),
            Decision::Dispatch { member, .. } => f.members[usize::from(member)].gates[0].receipt = Status::Passed,
            Decision::Merge { candidate, .. } => {
                for fresh in &mut f.members {
                    fresh.main = candidate;
                }
            }
            Decision::Record { member, .. } => f.members[usize::from(member)].record = Status::Passed,
            Decision::Hold(_) => f.released = true,
            Decision::Finish => {
                for member in &next.landing.members {
                    assert!(member.landed.is_some() || member.left.is_some(), "terminal seed {seed}");
                }
                return (next.landing, trace.into_boxed());
            }
            Decision::None
            | Decision::Wait { .. }
            | Decision::Post { .. }
            | Decision::Retry { .. }
            | Decision::Repair { .. }
            | Decision::Proved
            | Decision::Stale
            | Decision::Cancel
            | Decision::Landed { .. }
            | Decision::Resolve { .. }
            | Decision::Leave { .. } => {}
        }
        f.turn_free = true;
        landing = next.landing;
    }
    panic!("did not settle seed {seed}: {trace:?}");
}
